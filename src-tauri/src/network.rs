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
    let mut seen_descriptions = HashSet::new();

    // Keywords to filter out unless they are the only option
    // "LightWeight Filter" often appears as duplicates of the real adapter
    let filter_keywords = ["LightWeight", "Filter", "Virtual", "Pseudo", "Loopback", "Teredo", "6to4", "Isatap"];

    for (name, data) in networks.iter() {
        let name_lower = name.to_lowercase();
        
        // Skip obvious loopback or non-physical/virtual generic adapters that are usually noise
        if name_lower.contains("loopback") {
            continue;
        }

        // Identify if this is likely a filter driver (duplicate)
        let is_filter = filter_keywords.iter().any(|k| name_lower.contains(&k.to_lowercase()));
        
        // Use MAC address or just name for deduplication if possible, but sysinfo 0.30+ exposes mac_address()
        // For now, we'll try to dedup by name/description but prioritize "clean" names.
        // If we have "Wi-Fi" and "Wi-Fi-Realtek LightWeight Filter", we want "Wi-Fi".
        
        let interface = NetworkInterface {
            name: name.to_string(),
            description: name.to_string(),
            bytes_sent: data.total_transmitted(),
            bytes_received: data.total_received(),
            status: "Up".to_string(), // sysinfo doesn't give Up/Down efficiently per interface without refresh, assuming Up if listed usually
        };

        if is_filter {
            // Only add filters if we haven't seen a "real" adapter with a similar name prefix? 
            // Or just skip them aggressively as the user requested "one of them".
            // "Wi-Fi-Realtek..." vs "Wi-Fi".
            // We'll skip them entirely for now as they are usually redundant stats.
            continue;
        }

        if seen_descriptions.insert(name.to_string()) {
            unique_interfaces.push(interface);
        }
    }

    // Sort by name
    unique_interfaces.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(unique_interfaces)
}
