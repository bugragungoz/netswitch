use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::SystemState;

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

/// Gets list of processes with network connections using sysinfo (native Rust) + netstat
#[tauri::command]
pub async fn get_network_processes(
    state: tauri::State<'_, SystemState>,
) -> Result<Vec<NetworkProcess>, String> {
    // Lock the shared System state (tokio async mutex)
    let mut sys = state.0.lock().await;
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

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
    use sysinfo::Networks;
    use std::collections::HashSet;
    
    let networks = Networks::new_with_refreshed_list();
    let mut unique_interfaces: Vec<NetworkInterface> = Vec::new();
    let mut seen_names = HashSet::new();

    for (name, data) in networks.iter() {
        // Skip loopback and if we've already seen this interface name
        if name.to_lowercase().contains("loopback") || !seen_names.insert(name.to_string()) {
            continue;
        }

        // Filter out some common virtual/duplicate adaptors if needed, 
        // but primarily rely on exact name deduplication for now.
        
        unique_interfaces.push(NetworkInterface {
            name: name.to_string(),
            description: name.to_string(), // sysinfo still limits us here, but we can improve later
            bytes_sent: data.total_transmitted(),
            bytes_received: data.total_received(),
            status: "Up".to_string(),
        });
    }

    // Sort by name for consistent display
    unique_interfaces.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(unique_interfaces)
}
