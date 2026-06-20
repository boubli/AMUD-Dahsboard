use amud_protocol::{
    agent_auth_proof, AgentTelemetry, AuthProofMessage, ChallengeMessage, ConfigRequest,
    LxcContainer, NetworkTelemetry,
};
use serde::Serialize;
use std::io::Write;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use sysinfo::System;

const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

async fn with_http_timeout<T, F>(future: F) -> Result<T, String>
where
    F: std::future::Future<Output = Result<T, String>>,
{
    tokio::time::timeout(HTTP_TIMEOUT, future)
        .await
        .map_err(|_| "Request timed out".to_string())?
}

#[derive(Clone, Default)]
struct GpuSnapshot {
    name: String,
    usage: i32,
    mem_usage: i32,
    mem_used_mb: f64,
    mem_total_mb: f64,
}

fn read_gpu_snapshot() -> GpuSnapshot {
    use std::process::Command;

    let output = match Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,utilization.gpu,utilization.memory,memory.used,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return GpuSnapshot::default(),
    };

    let line = String::from_utf8_lossy(&output.stdout);
    let line = line.lines().next().unwrap_or("").trim();
    if line.is_empty() {
        return GpuSnapshot::default();
    }

    // GPU name may contain commas; numeric fields are always last.
    let mut parts = line.rsplitn(5, ',');
    let mem_total = parts
        .next()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .unwrap_or(0.0);
    let mem_used = parts
        .next()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .unwrap_or(0.0);
    let mem_usage = parts
        .next()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .map(|v| v.round() as i32)
        .unwrap_or(-1);
    let usage = parts
        .next()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .map(|v| v.round() as i32)
        .unwrap_or(-1);
    let name = parts.next().unwrap_or("").trim().to_string();

    if name.is_empty() || usage < 0 {
        return GpuSnapshot::default();
    }

    GpuSnapshot {
        name,
        usage,
        mem_usage,
        mem_used_mb: mem_used,
        mem_total_mb: mem_total,
    }
}

#[derive(Clone, Default)]
struct NetworkSnapshot {
    internal_rx: u64,
    internal_tx: u64,
    external_rx: u64,
    external_tx: u64,
}

fn main() {
    println!("AMUD-Agent telemetry client starting up...");

    if std::env::var("AMUD_AGENT_SECRET")
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        eprintln!("FATAL: AMUD_AGENT_SECRET is not set.");
        std::process::exit(1);
    }

    loop {
        println!("Attempting to connect to AMUD dashboard daemon...");
        match establish_connection() {
            Ok(stream) => {
                println!("Connected successfully. Beginning telemetry stream.");
                if let Err(e) = run_telemetry_loop(stream) {
                    eprintln!("Telemetry stream error occurred: {}", e);
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to connect to dashboard daemon: {}. Retrying in 5 seconds...",
                    e
                );
            }
        }
        sleep(Duration::from_secs(5));
    }
}

#[cfg(unix)]
type StreamType = std::os::unix::net::UnixStream;

#[cfg(windows)]
type StreamType = std::net::TcpStream;

#[cfg(unix)]
fn establish_connection() -> Result<StreamType, std::io::Error> {
    let path =
        std::env::var("AMUD_SOCKET_PATH").unwrap_or_else(|_| "/opt/amud/run/amud.sock".to_string());

    println!("Connecting via UDS to {}", path);
    std::os::unix::net::UnixStream::connect(&path)
}

#[cfg(windows)]
fn establish_connection() -> Result<StreamType, std::io::Error> {
    let addr = std::env::var("AMUD_TCP_ADDR").unwrap_or_else(|_| "127.0.0.1:8050".to_string());
    println!("Connecting via TCP to {}", addr);
    std::net::TcpStream::connect(addr)
}

// Trust-all certificate verifier. Proxmox ships a self-signed cert on :8006 and
// the agent only ever talks to the loopback node, so standard chain validation
// cannot succeed and MITM on 127.0.0.1 is not a meaningful threat. This disables
// verification deliberately and MUST NOT be reused for remote/non-loopback hosts.
#[derive(Debug)]
struct NoVerifier;

impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

use std::sync::{OnceLock, RwLock};

static AGENT_RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn agent_runtime() -> &'static tokio::runtime::Runtime {
    AGENT_RT.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build agent tokio runtime")
    })
}

fn pve_token_from_env() -> Option<String> {
    std::env::var("PVE_API_TOKEN")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(unix)]
fn docker_enabled() -> bool {
    matches!(
        std::env::var("AMUD_DOCKER").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

fn pve_node_name() -> String {
    std::env::var("PVE_NODE")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .unwrap_or_else(|_| "localhost".to_string())
                .trim()
                .to_string()
        })
}

static PVE_API_TOKEN_CACHE: OnceLock<RwLock<String>> = OnceLock::new();

fn get_pve_api_token() -> String {
    if let Some(env) = pve_token_from_env() {
        return env;
    }
    if let Some(lock) = PVE_API_TOKEN_CACHE.get() {
        let val = lock.read().unwrap().clone();
        if !val.is_empty() {
            return val;
        }
    }
    String::new()
}

fn update_pve_api_token(token: &str) {
    if pve_token_from_env().is_some() {
        return;
    }
    let lock = PVE_API_TOKEN_CACHE.get_or_init(|| RwLock::new(String::new()));
    let mut w = lock.write().unwrap();
    *w = token.to_string();
}

#[derive(Serialize, serde::Deserialize)]
struct PveTestResult {
    success: bool,
    error: Option<String>,
}

fn perform_pve_test(token: &str) -> PveTestResult {
    if token.is_empty() {
        return PveTestResult {
            success: false,
            error: Some("Token is empty".to_string()),
        };
    }
    match fetch_lxc_containers_with_token(token) {
        Ok(_) => PveTestResult {
            success: true,
            error: None,
        },
        Err(e) => PveTestResult {
            success: false,
            error: Some(e),
        },
    }
}

// Native Proxmox LXC fetch over HTTPS (replaces the `pvesh` subprocess fork).
// Reads PVE_API_TOKEN and queries the local PVE API. Returns an empty vec on any
// failure so a missing token or unreachable node never crashes the telemetry loop.
fn fetch_lxc_containers() -> Vec<LxcContainer> {
    let token = get_pve_api_token();
    if token.is_empty() {
        eprintln!("[LXC] PVE_API_TOKEN not set or empty, skipping LXC fetch.");
        return Vec::new();
    }
    match fetch_lxc_containers_with_token(&token) {
        Ok(containers) => containers,
        Err(e) => {
            eprintln!("[LXC] Error fetching containers: {}", e);
            Vec::new()
        }
    }
}

fn fetch_lxc_containers_with_token(token: &str) -> Result<Vec<LxcContainer>, String> {
    let token = token.to_string();
    agent_runtime().block_on(async move {
        with_http_timeout(async move {
            use http_body_util::{BodyExt, Empty};
            use hyper::body::Bytes;
            use hyper_util::client::legacy::Client;
            use hyper_util::rt::TokioExecutor;

            let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
                rustls::crypto::ring::default_provider(),
            ))
            .with_safe_default_protocol_versions();
            let tls = match tls {
                Ok(b) => b
                    .dangerous()
                    .with_custom_certificate_verifier(Arc::new(NoVerifier))
                    .with_no_client_auth(),
                Err(e) => {
                    return Err(format!("Failed to build TLS config: {}", e));
                }
            };

            let https = hyper_rustls::HttpsConnectorBuilder::new()
                .with_tls_config(tls)
                .https_or_http()
                .enable_http1()
                .build();

            let client: Client<_, Empty<Bytes>> =
                Client::builder(TokioExecutor::new()).build(https);

            let node_name = pve_node_name();

            let api_url = format!("https://localhost:8006/api2/json/nodes/{}/lxc", node_name);
            eprintln!("[LXC] Fetching containers from: {}...", api_url);

            let req = hyper::Request::builder()
                .method("GET")
                .uri(&api_url)
                .header("Authorization", token)
                .body(Empty::<Bytes>::new())
                .map_err(|e| format!("Failed to build HTTP request: {}", e))?;

            let resp = client
                .request(req)
                .await
                .map_err(|e| format!("HTTP request to PVE API failed: {}", e))?;

            let status = resp.status();
            let body = resp
                .into_body()
                .collect()
                .await
                .map_err(|e| format!("Failed to read response body: {}", e))?
                .to_bytes();

            if !status.is_success() {
                let body_str = String::from_utf8_lossy(&body);
                let snippet = body_str.chars().take(200).collect::<String>();
                return Err(format!("PVE API returned HTTP {}: {}", status, snippet));
            }

            // The PVE REST API wraps the array in a `{ "data": [...] }` envelope,
            // unlike the bare array that `pvesh --output-format json` returned.
            #[derive(serde::Deserialize)]
            struct PveResponse {
                data: Vec<LxcContainer>,
            }

            match serde_json::from_slice::<PveResponse>(&body) {
                Ok(parsed) => {
                    eprintln!(
                        "[LXC] Successfully fetched {} containers from PVE.",
                        parsed.data.len()
                    );
                    Ok(parsed.data)
                }
                Err(e) => {
                    let body_str = String::from_utf8_lossy(&body);
                    let snippet = body_str.chars().take(200).collect::<String>();
                    Err(format!(
                        "Failed to parse PVE response: {}. Body snippet: {}",
                        e, snippet
                    ))
                }
            }
        })
        .await
    })
}

fn format_rate(bytes_per_sec: u64) -> String {
    let bits = bytes_per_sec as f64 * 8.0;
    if bits >= 1_000_000.0 {
        format!("{:.1} Mbit/s", bits / 1_000_000.0)
    } else {
        format!("{:.0} kbit/s", bits / 1_000.0)
    }
}

fn read_network_snapshot() -> NetworkSnapshot {
    let Ok(content) = std::fs::read_to_string("/proc/net/dev") else {
        return NetworkSnapshot::default();
    };
    let mut snapshot = NetworkSnapshot::default();

    for line in content.lines().skip(2) {
        let Some((iface, stats)) = line.split_once(':') else {
            continue;
        };
        let iface = iface.trim();
        if iface == "lo" {
            continue;
        }
        let values: Vec<u64> = stats
            .split_whitespace()
            .filter_map(|v| v.parse::<u64>().ok())
            .collect();
        if values.len() < 16 {
            continue;
        }

        let rx = values[0];
        let tx = values[8];
        if iface.starts_with("vmbr") || iface.starts_with("br-") || iface.starts_with("docker") {
            snapshot.internal_rx = snapshot.internal_rx.saturating_add(rx);
            snapshot.internal_tx = snapshot.internal_tx.saturating_add(tx);
        } else {
            snapshot.external_rx = snapshot.external_rx.saturating_add(rx);
            snapshot.external_tx = snapshot.external_tx.saturating_add(tx);
        }
    }

    snapshot
}

fn network_rates(
    previous: &NetworkSnapshot,
    current: &NetworkSnapshot,
    elapsed: f64,
) -> NetworkTelemetry {
    let seconds = elapsed.max(1.0);
    let rate = |now: u64, before: u64| -> u64 {
        now.saturating_sub(before)
            .checked_div(seconds as u64)
            .unwrap_or(0)
    };

    NetworkTelemetry {
        internal_tx: format_rate(rate(current.internal_tx, previous.internal_tx)),
        internal_rx: format_rate(rate(current.internal_rx, previous.internal_rx)),
        external_tx: format_rate(rate(current.external_tx, previous.external_tx)),
        external_rx: format_rate(rate(current.external_rx, previous.external_rx)),
    }
}

// Native Docker fetch over the Engine API UNIX socket (replaces the `curl` fork).
// Returns Docker containers with real CPU and memory stats where available.
#[cfg(unix)]
fn fetch_docker_containers() -> Vec<LxcContainer> {
    if !docker_enabled() {
        return Vec::new();
    }
    if !std::path::Path::new("/var/run/docker.sock").exists() {
        return Vec::new();
    }

    let rt = agent_runtime();

    rt.block_on(async move {
        use http_body_util::{BodyExt, Empty};
        use hyper::body::Bytes;
        use hyper_util::client::legacy::Client;
        use hyperlocal::{UnixClientExt, UnixConnector, Uri as UnixUri};

        let client: Client<UnixConnector, Empty<Bytes>> = Client::unix();

        let uri: hyper::Uri = UnixUri::new("/var/run/docker.sock", "/containers/json").into();
        let req = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .header("Host", "localhost")
            .body(Empty::<Bytes>::new())
        {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let resp = match client.request(req).await {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let body = match resp.into_body().collect().await {
            Ok(b) => b.to_bytes(),
            Err(_) => return Vec::new(),
        };

        #[derive(serde::Deserialize)]
        struct DockerContainer {
            #[serde(rename = "Id")]
            id: String,
            #[serde(rename = "Names")]
            names: Vec<String>,
            #[serde(rename = "State")]
            state: String,
        }

        #[derive(serde::Deserialize, Default)]
        struct CpuUsage {
            total_usage: Option<u64>,
            percpu_usage: Option<Vec<u64>>,
        }

        #[derive(serde::Deserialize, Default)]
        struct CpuStats {
            cpu_usage: Option<CpuUsage>,
            system_cpu_usage: Option<u64>,
            online_cpus: Option<u64>,
        }

        #[derive(serde::Deserialize, Default)]
        struct MemoryStats {
            usage: Option<i64>,
            limit: Option<i64>,
        }

        #[derive(serde::Deserialize, Default)]
        struct DockerStats {
            cpu_stats: Option<CpuStats>,
            precpu_stats: Option<CpuStats>,
            memory_stats: Option<MemoryStats>,
        }

        async fn docker_stats(
            client: &Client<UnixConnector, Empty<Bytes>>,
            id: &str,
        ) -> (Option<f64>, Option<i64>, Option<i64>) {
            let path = format!("/containers/{}/stats?stream=false", id);
            let uri: hyper::Uri = UnixUri::new("/var/run/docker.sock", &path).into();
            let req = match hyper::Request::builder()
                .method("GET")
                .uri(uri)
                .header("Host", "localhost")
                .body(Empty::<Bytes>::new())
            {
                Ok(req) => req,
                Err(_) => return (None, None, None),
            };
            let Ok(resp) = client.request(req).await else {
                return (None, None, None);
            };
            let Ok(body) = resp.into_body().collect().await else {
                return (None, None, None);
            };
            let Ok(stats) = serde_json::from_slice::<DockerStats>(&body.to_bytes()) else {
                return (None, None, None);
            };

            let cpu = match (&stats.cpu_stats, &stats.precpu_stats) {
                (Some(cpu), Some(pre)) => {
                    let cpu_total = cpu
                        .cpu_usage
                        .as_ref()
                        .and_then(|u| u.total_usage)
                        .unwrap_or(0);
                    let pre_total = pre
                        .cpu_usage
                        .as_ref()
                        .and_then(|u| u.total_usage)
                        .unwrap_or(0);
                    let sys_total = cpu.system_cpu_usage.unwrap_or(0);
                    let pre_sys = pre.system_cpu_usage.unwrap_or(0);
                    let cpu_delta = cpu_total.saturating_sub(pre_total) as f64;
                    let sys_delta = sys_total.saturating_sub(pre_sys) as f64;
                    let cpus = cpu
                        .online_cpus
                        .or_else(|| {
                            cpu.cpu_usage
                                .as_ref()
                                .and_then(|u| u.percpu_usage.as_ref().map(|p| p.len() as u64))
                        })
                        .unwrap_or(1) as f64;
                    if sys_delta > 0.0 && cpu_delta > 0.0 {
                        Some((cpu_delta / sys_delta) * cpus)
                    } else {
                        Some(0.0)
                    }
                }
                _ => None,
            };
            let mem = stats.memory_stats.as_ref().and_then(|m| m.usage);
            let maxmem = stats.memory_stats.as_ref().and_then(|m| m.limit);
            (cpu, mem, maxmem)
        }

        match serde_json::from_slice::<Vec<DockerContainer>>(&body) {
            Ok(dockers) => {
                let mut out = Vec::new();
                for (i, d) in dockers.into_iter().enumerate() {
                    let name = d
                        .names
                        .into_iter()
                        .next()
                        .unwrap_or_default()
                        .replace('/', "");
                    let (cpu, mem, maxmem) = if d.state == "running" {
                        docker_stats(&client, &d.id).await
                    } else {
                        (Some(0.0), None, None)
                    };
                    out.push(LxcContainer {
                        vmid: -1000 - i as i64,
                        status: d.state,
                        name,
                        cpu,
                        maxmem,
                        mem,
                        maxdisk: None,
                        disk: None,
                        uptime: None,
                    });
                }
                out
            }
            Err(_) => Vec::new(),
        }
    })
}

#[cfg(not(unix))]
fn fetch_docker_containers() -> Vec<LxcContainer> {
    Vec::new()
}

fn run_telemetry_loop(mut stream: StreamType) -> Result<(), std::io::Error> {
    let mut sys = System::new_all();

    let agent_secret = std::env::var("AMUD_AGENT_SECRET").unwrap_or_default();
    if agent_secret.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "AMUD_AGENT_SECRET is not set",
        ));
    }

    // Server sends a challenge first; respond with SHA-256(secret || nonce) — secret never on the wire.
    {
        use std::io::{BufRead, BufReader, Write};
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut challenge_line = String::new();
        reader.read_line(&mut challenge_line)?;
        let nonce = serde_json::from_str::<ChallengeMessage>(&challenge_line)
            .ok()
            .and_then(|m| m.challenge)
            .filter(|n| !n.is_empty())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Missing IPC auth challenge from server",
                )
            })?;
        let proof = agent_auth_proof(&agent_secret, &nonce);
        let auth = AuthProofMessage { auth: proof };
        if let Ok(mut serialized) = serde_json::to_vec(&auth) {
            serialized.push(b'\n');
            stream.write_all(&serialized)?;
            stream.flush()?;
        }
    }

    let req = ConfigRequest {
        request: "get_config".to_string(),
        pve_token_configured: Some(pve_token_from_env().is_some()),
    };
    if let Ok(mut serialized) = serde_json::to_vec(&req) {
        serialized.push(b'\n');
        stream.write_all(&serialized)?;
        stream.flush()?;
    }

    let reader_stream = stream.try_clone()?;
    let mut response_stream = stream.try_clone()?;
    std::thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        let mut reader = BufReader::new(reader_stream);
        let mut line = String::new();
        while let Ok(n) = reader.read_line(&mut line) {
            if n == 0 {
                break;
            }
            execute_command_from_server(&line, &mut response_stream);
            line.clear();
        }
        println!("AMUD-Agent command reader thread exiting.");
    });

    // Proxmox LXC data is polled on its own slower 10s cadence to minimize overhead.
    // We cache the last result and reuse it between fetches.
    let mut cached_lxc: Vec<LxcContainer> = Vec::new();
    let mut last_lxc_fetch = Instant::now()
        .checked_sub(Duration::from_secs(10))
        .unwrap_or_else(Instant::now);
    let mut last_network_snapshot = read_network_snapshot();
    let mut last_network_sample = Instant::now();

    loop {
        sys.refresh_all();

        let cpus = sys.cpus();
        let cpu_usage = if !cpus.is_empty() {
            let sum: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
            (sum / cpus.len() as f32).round() as i32
        } else {
            0
        };
        let cpu_model = cpus
            .first()
            .map(|cpu| cpu.brand().trim().to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or_default();
        let cpu_cores = cpus.len() as u32;
        let gpu = read_gpu_snapshot();

        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let ram_usage = if total_mem > 0 {
            ((used_mem as f64 / total_mem as f64) * 100.0).round() as i32
        } else {
            0
        };
        let ram_total_gb = (total_mem as f64 / 1_073_741_824.0 * 100.0).round() / 100.0;
        let ram_used_gb = (used_mem as f64 / 1_073_741_824.0 * 100.0).round() / 100.0;

        let components = sysinfo::Components::new_with_refreshed_list();
        let mut cpu_temp = 0.0;
        for c in &components {
            let label = c.label().to_lowercase();
            if label.contains("cpu") || label.contains("core") || label.contains("tctl") {
                cpu_temp = c.temperature() as f64;
                break;
            }
        }
        if cpu_temp == 0.0 && !components.is_empty() {
            cpu_temp = components[0].temperature() as f64;
        }

        let disks = sysinfo::Disks::new_with_refreshed_list();
        let mut total_disk: u64 = 0;
        let mut avail_disk: u64 = 0;
        let mut seen_devices = std::collections::HashSet::new();
        for disk in &disks {
            let fs = disk.file_system().to_string_lossy().to_lowercase();
            let mount = disk.mount_point().to_string_lossy().to_string();
            if fs == "tmpfs"
                || fs == "overlay"
                || fs == "squashfs"
                || fs == "devtmpfs"
                || fs == "sysfs"
                || fs == "proc"
                || fs == "devpts"
                || fs == "cgroup"
                || fs == "cgroup2"
                || fs == "none"
                || fs == "ramfs"
            {
                continue;
            }
            if mount.starts_with("/snap") || mount.starts_with("/dev/loop") {
                continue;
            }
            let device_name = disk.name().to_string_lossy().to_string();
            if !seen_devices.insert(device_name) {
                continue;
            }
            total_disk += disk.total_space();
            avail_disk += disk.available_space();
        }
        let used_disk = total_disk - avail_disk;
        let disk_usage = if total_disk > 0 {
            ((used_disk as f64 / total_disk as f64) * 100.0).round() as i32
        } else {
            0
        };
        let disk_total_gb = (total_disk as f64 / 1_073_741_824.0 * 100.0).round() / 100.0;
        let disk_used_gb = (used_disk as f64 / 1_073_741_824.0 * 100.0).round() / 100.0;

        if last_lxc_fetch.elapsed() >= Duration::from_secs(10) {
            cached_lxc = fetch_lxc_containers();
            last_lxc_fetch = Instant::now();
        }
        let mut lxc_containers = cached_lxc.clone();

        lxc_containers.extend(fetch_docker_containers());

        let now_network_snapshot = read_network_snapshot();
        let network = network_rates(
            &last_network_snapshot,
            &now_network_snapshot,
            last_network_sample.elapsed().as_secs_f64(),
        );
        last_network_snapshot = now_network_snapshot;
        last_network_sample = Instant::now();

        let telemetry = AgentTelemetry {
            cpu_usage,
            ram_usage,
            ram_used_gb,
            ram_total_gb,
            cpu_temp,
            disk_usage,
            disk_used_gb,
            disk_total_gb,
            cpu_model,
            cpu_cores,
            gpu_name: gpu.name,
            gpu_usage: gpu.usage,
            gpu_mem_usage: gpu.mem_usage,
            gpu_mem_used_mb: gpu.mem_used_mb,
            gpu_mem_total_mb: gpu.mem_total_mb,
            lxc_containers,
            network: Some(network),
        };

        let mut serialized = serde_json::to_vec(&telemetry).unwrap_or_default();
        serialized.push(b'\n');

        stream.write_all(&serialized)?;
        stream.flush()?;

        sleep(Duration::from_secs(5));
    }
}

fn send_action_result(
    response_stream: &mut StreamType,
    request_id: &str,
    success: bool,
    error: Option<String>,
) {
    let response = serde_json::json!({
        "action_result": {
            "request_id": request_id,
            "success": success,
            "error": error
        }
    });
    if let Ok(mut serialized) = serde_json::to_vec(&response) {
        serialized.push(b'\n');
        use std::io::Write;
        if let Err(e) = response_stream.write_all(&serialized) {
            eprintln!("Failed to write action result back to server: {}", e);
        }
        let _ = response_stream.flush();
    }
}

fn execute_command_from_server(line: &str, response_stream: &mut StreamType) {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
        if let Some(config) = val.get("config") {
            if pve_token_from_env().is_none() {
                if let Some(token) = config.get("pve_api_token").and_then(|t| t.as_str()) {
                    if !token.is_empty() {
                        println!("Received PVE API Token update from server.");
                        update_pve_api_token(token);
                    }
                }
            }
        } else if let Some(action) = val.get("action").and_then(|a| a.as_str()) {
            if action == "test_pve" {
                let token = val
                    .get("id")
                    .and_then(|i| i.as_str())
                    .filter(|t| !t.is_empty())
                    .map(|s| s.to_string())
                    .unwrap_or_else(get_pve_api_token);
                println!("Received test_pve action from server.");

                let result = perform_pve_test(&token);

                let response = serde_json::json!({
                    "test_pve_result": result
                });
                if let Ok(mut serialized) = serde_json::to_vec(&response) {
                    serialized.push(b'\n');
                    use std::io::Write;
                    if let Err(e) = response_stream.write_all(&serialized) {
                        eprintln!("Failed to write test result back to server: {}", e);
                    }
                    let _ = response_stream.flush();
                }
            } else {
                #[derive(serde::Deserialize)]
                struct ServerCommand {
                    provider: String,
                    id: String,
                    action: String,
                    request_id: Option<String>,
                }
                if let Ok(cmd) = serde_json::from_value::<ServerCommand>(val) {
                    println!(
                        "Received control action: action='{}' provider='{}' id='{}'",
                        cmd.action, cmd.provider, cmd.id
                    );
                    let (success, error) = match cmd.provider.as_str() {
                        "lxc" => {
                            if let Ok(vmid) = cmd.id.parse::<i64>() {
                                execute_lxc_action(vmid, &cmd.action)
                            } else {
                                (false, Some(format!("Invalid LXC VMID: {}", cmd.id)))
                            }
                        }
                        "docker" => execute_docker_action(&cmd.id, &cmd.action),
                        _ => (false, Some(format!("Unknown provider: {}", cmd.provider))),
                    };
                    if let Some(request_id) = cmd.request_id {
                        send_action_result(response_stream, &request_id, success, error);
                    }
                }
            }
        }
    } else {
        eprintln!("Failed to parse incoming command payload: {}", line);
    }
}

fn execute_lxc_action(vmid: i64, action: &str) -> (bool, Option<String>) {
    let token = get_pve_api_token();
    if token.is_empty() {
        return (false, Some("PVE API token is not configured".to_string()));
    }

    let action_str = match action {
        "start" | "stop" | "reboot" | "shutdown" => action,
        "restart" => "reboot",
        _ => return (false, Some(format!("Unsupported LXC action: {}", action))),
    };

    agent_runtime().block_on(async move {
        match tokio::time::timeout(HTTP_TIMEOUT, async move {
            use http_body_util::Empty;
            use hyper::body::Bytes;
            use hyper_util::client::legacy::Client;
            use hyper_util::rt::TokioExecutor;

            let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
                rustls::crypto::ring::default_provider(),
            ))
            .with_safe_default_protocol_versions();
            let tls = match tls {
                Ok(b) => b
                    .dangerous()
                    .with_custom_certificate_verifier(Arc::new(NoVerifier))
                    .with_no_client_auth(),
                Err(e) => {
                    return (false, Some(format!("Failed to build TLS config: {}", e)));
                }
            };

            let https = hyper_rustls::HttpsConnectorBuilder::new()
                .with_tls_config(tls)
                .https_or_http()
                .enable_http1()
                .build();

            let client: Client<_, Empty<Bytes>> =
                Client::builder(TokioExecutor::new()).build(https);

            let node_name = pve_node_name();

            let api_url = format!(
                "https://localhost:8006/api2/json/nodes/{}/lxc/{}/status/{}",
                node_name, vmid, action_str
            );
            println!("[LXC Action] POSTing: {}", api_url);

            let req = match hyper::Request::builder()
                .method("POST")
                .uri(&api_url)
                .header("Authorization", &token)
                .body(Empty::<Bytes>::new())
            {
                Ok(r) => r,
                Err(e) => {
                    return (false, Some(format!("Failed to build request: {}", e)));
                }
            };

            match client.request(req).await {
                Ok(resp) => {
                    let status = resp.status();
                    println!("[LXC Action] PVE API response status: {}", status);
                    if status.is_success() {
                        (true, None)
                    } else {
                        (false, Some(format!("PVE API returned HTTP {}", status)))
                    }
                }
                Err(e) => (false, Some(format!("HTTP request failed: {}", e))),
            }
        })
        .await
        {
            Ok(result) => result,
            Err(_) => (false, Some("LXC action timed out".to_string())),
        }
    })
}

#[cfg(unix)]
fn execute_docker_action(container_name: &str, action: &str) -> (bool, Option<String>) {
    if !docker_enabled() {
        return (
            false,
            Some("Docker integration disabled (set AMUD_DOCKER=1 to enable)".to_string()),
        );
    }
    if !std::path::Path::new("/var/run/docker.sock").exists() {
        return (false, Some("Docker Unix socket missing".to_string()));
    }

    let action_str = match action {
        "start" | "stop" | "restart" => action,
        _ => {
            return (
                false,
                Some(format!("Unsupported Docker action: {}", action)),
            )
        }
    };

    let c_name = container_name.to_string();

    agent_runtime().block_on(async move {
        match tokio::time::timeout(HTTP_TIMEOUT, async move {
            use http_body_util::Empty;
            use hyper::body::Bytes;
            use hyper_util::client::legacy::Client;
            use hyperlocal::{UnixClientExt, UnixConnector, Uri as UnixUri};

            let client: Client<UnixConnector, Empty<Bytes>> = Client::unix();
            let api_path = format!("/containers/{}/{}", c_name, action_str);
            let uri: hyper::Uri = UnixUri::new("/var/run/docker.sock", &api_path).into();

            println!("[Docker Action] POSTing: {}", api_path);

            let req = match hyper::Request::builder()
                .method("POST")
                .uri(uri)
                .header("Host", "localhost")
                .body(Empty::<Bytes>::new())
            {
                Ok(r) => r,
                Err(e) => return (false, Some(format!("Failed to build request: {}", e))),
            };

            match client.request(req).await {
                Ok(resp) => {
                    let status = resp.status();
                    println!("[Docker Action] Docker API response status: {}", status);
                    if status.is_success() || status.as_u16() == 304 {
                        (true, None)
                    } else {
                        (false, Some(format!("Docker API returned HTTP {}", status)))
                    }
                }
                Err(e) => (false, Some(format!("HTTP request failed: {}", e))),
            }
        })
        .await
        {
            Ok(result) => result,
            Err(_) => (false, Some("Docker action timed out".to_string())),
        }
    })
}

#[cfg(not(unix))]
fn execute_docker_action(_container_name: &str, _action: &str) -> (bool, Option<String>) {
    (
        false,
        Some("Docker actions are unavailable on this platform".to_string()),
    )
}
