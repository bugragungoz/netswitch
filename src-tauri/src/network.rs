use serde::{Deserialize, Serialize};
use sysinfo::{Networks, System};
use std::collections::HashMap;

/// Represents a process with network activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkProcess {
    pub pid: u32,
    pub name: String,
    pub tcp_connections: u32,
    pub udp_connections: u32,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub description: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub status: String,
}

/// Gets list of processes with network connections using sysinfo (native Rust)
#[tauri::command]
pub async fn get_network_processes() -> Result<Vec<NetworkProcess>, String> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

    // Get processes that are likely using network (we can't get exact connection count from sysinfo,
    // but we can list running processes - for detailed connection info, we still need netstat)
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        // Use netstat for connection info but process names from sysinfo (faster)
        let output = std::process::Command::new("netstat")
            .args(["-ano"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to run netstat: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut process_map: HashMap<u32, NetworkProcess> = HashMap::new();

        for line in stdout.lines().skip(4) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let protocol = parts[0];
                if let Ok(pid) = parts.last().unwrap_or(&"0").parse::<u32>() {
                    if pid == 0 {
                        continue;
                    }

                    let entry = process_map.entry(pid).or_insert_with(|| {
                        let name = sys
                            .process(sysinfo::Pid::from_u32(pid))
                            .map(|p| p.name().to_string_lossy().to_string())
                            .unwrap_or_else(|| format!("PID {}", pid));

                        NetworkProcess {
                            pid,
                            name,
                            tcp_connections: 0,
                            udp_connections: 0,
                            bytes_sent: 0,
                            bytes_received: 0,
                        }
                    });

                    if protocol.starts_with("TCP") {
                        entry.tcp_connections += 1;
                    } else if protocol.starts_with("UDP") {
                        entry.udp_connections += 1;
                    }
                }
            }
        }

        let mut processes: Vec<NetworkProcess> = process_map.into_values().collect();
        processes.sort_by(|a, b| {
            (b.tcp_connections + b.udp_connections).cmp(&(a.tcp_connections + a.udp_connections))
        });

        Ok(processes)
    }

    #[cfg(not(windows))]
    {
        Ok(vec![])
    }
}

/// Gets network interface statistics using sysinfo (native Rust - much faster than PowerShell)
#[tauri::command]
pub async fn get_network_interfaces() -> Result<Vec<NetworkInterface>, String> {
    let networks = Networks::new_with_refreshed_list();

    let interfaces: Vec<NetworkInterface> = networks
        .iter()
        .map(|(name, data)| NetworkInterface {
            name: name.to_string(),
            description: name.to_string(), // sysinfo doesn't provide description
            bytes_sent: data.total_transmitted(),
            bytes_received: data.total_received(),
            status: "Up".to_string(),
        })
        .collect();

    Ok(interfaces)
}

/// System statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub ram_used_gb: f32,
    pub ram_total_gb: f32,
}

/// Gets system CPU and RAM usage using sysinfo (native Rust)
#[tauri::command]
pub async fn get_system_stats_native() -> Result<SystemStats, String> {
    let mut sys = System::new();
    
    // Need to refresh twice with delay for accurate CPU usage
    sys.refresh_cpu_usage();
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    let cpu_usage = sys.global_cpu_usage();
    let ram_used = sys.used_memory() as f64 / 1_073_741_824.0; // bytes to GB
    let ram_total = sys.total_memory() as f64 / 1_073_741_824.0;

    Ok(SystemStats {
        cpu_usage,
        ram_used_gb: ram_used as f32,
        ram_total_gb: ram_total as f32,
    })
}
