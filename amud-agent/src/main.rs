use std::io::Write;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use serde::Serialize;
use sysinfo::System;

#[derive(Serialize)]
struct Telemetry {
    cpu_usage: i32,
    ram_usage: i32,
    ram_used_gb: f64,
    ram_total_gb: f64,
    cpu_temp: f64,
    disk_usage: i32,
    disk_used_gb: f64,
    disk_total_gb: f64,
    lxc_containers: Vec<LxcContainer>,
}

#[derive(Serialize, serde::Deserialize, Clone, Default)]
struct LxcContainer {
    vmid: i64,
    status: String,
    name: String,
    cpu: Option<f64>,
    maxmem: Option<i64>,
    mem: Option<i64>,
    maxdisk: Option<i64>,
    disk: Option<i64>,
    uptime: Option<i64>,
}

fn main() {
    println!("AMUD-Agent telemetry client starting up...");
    
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
                eprintln!("Failed to connect to dashboard daemon: {}. Retrying in 5 seconds...", e);
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
    let path = std::env::var("AMUD_SOCKET_PATH")
        .unwrap_or_else(|_| "/opt/amud/run/amud.sock".to_string());
    
    println!("Connecting via UDS to {}", path);
    match std::os::unix::net::UnixStream::connect(&path) {
        Ok(s) => Ok(s),
        Err(e) => {
            let fallback = "/tmp/amud.sock";
            println!("Connection to {} failed ({}). Trying fallback: {}", path, e, fallback);
            std::os::unix::net::UnixStream::connect(fallback)
        }
    }
}

#[cfg(windows)]
fn establish_connection() -> Result<StreamType, std::io::Error> {
    let addr = std::env::var("AMUD_TCP_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8050".to_string());
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

// Native Proxmox LXC fetch over HTTPS (replaces the `pvesh` subprocess fork).
// Reads PVE_API_TOKEN and queries the local PVE API. Returns an empty vec on any
// failure so a missing token or unreachable node never crashes the telemetry loop.
fn fetch_lxc_containers() -> Vec<LxcContainer> {
    let token = match std::env::var("PVE_API_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => return Vec::new(),
    };

    // Drive the async hyper client on a lightweight current-thread runtime so the
    // rest of the agent stays synchronous.
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(_) => return Vec::new(),
    };

    rt.block_on(async move {
        use http_body_util::{BodyExt, Empty};
        use hyper::body::Bytes;
        use hyper_util::client::legacy::Client;
        use hyper_util::rt::TokioExecutor;

        // Build a rustls client config that trusts the Proxmox self-signed cert.
        let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions();
        let tls = match tls {
            Ok(b) => b
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoVerifier))
                .with_no_client_auth(),
            Err(_) => return Vec::new(),
        };

        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_tls_config(tls)
            .https_or_http()
            .enable_http1()
            .build();

        // legacy::Client handles connection setup/pooling, so no manual handshake.
        let client: Client<_, Empty<Bytes>> =
            Client::builder(TokioExecutor::new()).build(https);

        // PVE_API_TOKEN is expected to hold the full credential, including the
        // `PVEAPIToken=` scheme prefix, e.g. `PVEAPIToken=root@pam!amud=<secret>`.
        let req = match hyper::Request::builder()
            .method("GET")
            .uri("https://localhost:8006/api2/json/nodes/localhost/lxc")
            .header("Authorization", token)
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

        // The PVE REST API wraps the array in a `{ "data": [...] }` envelope,
        // unlike the bare array that `pvesh --output-format json` returned.
        #[derive(serde::Deserialize)]
        struct PveResponse {
            data: Vec<LxcContainer>,
        }

        serde_json::from_slice::<PveResponse>(&body)
            .map(|parsed| parsed.data)
            .unwrap_or_default()
    })
}

// Native Docker fetch over the Engine API UNIX socket (replaces the `curl` fork).
// Returns (name, state) pairs. Empty vec on any failure or if the socket is absent.
fn fetch_docker_containers() -> Vec<(String, String)> {
    if !std::path::Path::new("/var/run/docker.sock").exists() {
        return Vec::new();
    }

    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(_) => return Vec::new(),
    };

    rt.block_on(async move {
        use http_body_util::{BodyExt, Empty};
        use hyper::body::Bytes;
        use hyper_util::client::legacy::Client;
        use hyperlocal::{UnixClientExt, UnixConnector, Uri as UnixUri};

        // hyperlocal's UnixConnector speaks HTTP/1 directly over the socket.
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
            #[serde(rename = "Names")]
            names: Vec<String>,
            #[serde(rename = "State")]
            state: String,
        }

        match serde_json::from_slice::<Vec<DockerContainer>>(&body) {
            Ok(dockers) => dockers
                .into_iter()
                .map(|d| {
                    let name = d
                        .names
                        .into_iter()
                        .next()
                        .unwrap_or_default()
                        .replace('/', "");
                    (name, d.state)
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    })
}

fn run_telemetry_loop(mut stream: StreamType) -> Result<(), std::io::Error> {
    let mut sys = System::new_all();

    // Proxmox LXC data is polled on its own slower 10s cadence to minimize overhead.
    // We cache the last result and reuse it between fetches.
    let mut cached_lxc: Vec<LxcContainer> = Vec::new();
    let mut last_lxc_fetch = Instant::now()
        .checked_sub(Duration::from_secs(10))
        .unwrap_or_else(Instant::now);

    loop {
        // Refresh telemetry
        sys.refresh_all();
        
        // Let CPU settle and average usage across cores
        let cpus = sys.cpus();
        let cpu_usage = if !cpus.is_empty() {
            let sum: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
            (sum / cpus.len() as f32).round() as i32
        } else {
            0
        };

        // RAM Usage in GB
        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let ram_usage = if total_mem > 0 {
            ((used_mem as f64 / total_mem as f64) * 100.0).round() as i32
        } else {
            0
        };
        let ram_total_gb = (total_mem as f64 / 1_073_741_824.0 * 100.0).round() / 100.0;
        let ram_used_gb = (used_mem as f64 / 1_073_741_824.0 * 100.0).round() / 100.0;

        // CPU Temperatures
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

        // Disk metrics
        let disks = sysinfo::Disks::new_with_refreshed_list();
        let mut total_disk = 0;
        let mut avail_disk = 0;
        for disk in &disks {
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

        // Fetch LXC info from Proxmox natively (no subprocess fork), throttled to every 10s.
        if last_lxc_fetch.elapsed() >= Duration::from_secs(10) {
            cached_lxc = fetch_lxc_containers();
            last_lxc_fetch = Instant::now();
        }
        let mut lxc_containers = cached_lxc.clone();

        // Fetch Docker containers natively over the UNIX socket (no curl fork) and merge.
        for (i, (name, state)) in fetch_docker_containers().into_iter().enumerate() {
            // Negative vmid sequence so Docker entries never collide with Proxmox LXC IDs (100+).
            lxc_containers.push(LxcContainer {
                vmid: -1000 - i as i64,
                status: state,
                name,
                cpu: None,
                maxmem: None,
                mem: None,
                maxdisk: None,
                disk: None,
                uptime: None,
            });
        }

        let telemetry = Telemetry {
            cpu_usage,
            ram_usage,
            ram_used_gb,
            ram_total_gb,
            cpu_temp,
            disk_usage,
            disk_used_gb,
            disk_total_gb,
            lxc_containers,
        };

        // Serialize and push
        let mut serialized = serde_json::to_vec(&telemetry).unwrap_or_default();
        serialized.push(b'\n'); // newline delimited JSON

        stream.write_all(&serialized)?;
        stream.flush()?;

        sleep(Duration::from_secs(5));
    }
}
