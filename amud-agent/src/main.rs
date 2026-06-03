use std::io::Write;
use std::thread::sleep;
use std::time::Duration;
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
        .unwrap_or_else(|_| "/var/run/amud.sock".to_string());
    
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

fn run_telemetry_loop(mut stream: StreamType) -> Result<(), std::io::Error> {
    let mut sys = System::new_all();
    
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

        let telemetry = Telemetry {
            cpu_usage,
            ram_usage,
            ram_used_gb,
            ram_total_gb,
            cpu_temp,
            disk_usage,
            disk_used_gb,
            disk_total_gb,
        };

        // Serialize and push
        let mut serialized = serde_json::to_vec(&telemetry).unwrap_or_default();
        serialized.push(b'\n'); // newline delimited JSON

        stream.write_all(&serialized)?;
        stream.flush()?;

        sleep(Duration::from_secs(3));
    }
}
