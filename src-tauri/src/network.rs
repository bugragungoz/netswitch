use serde::{Deserialize, Serialize};

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

/// Gets list of processes with network connections using netstat-like approach
/// Simplified version without Get-Counter which can be slow
#[tauri::command]
pub async fn get_network_processes() -> Result<Vec<NetworkProcess>, String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        // Simplified PowerShell - just get connections without performance counters
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy", "Bypass",
                "-WindowStyle", "Hidden",
                "-Command",
                r#"
                $ErrorActionPreference = 'SilentlyContinue'
                
                # Get TCP connections grouped by process
                $tcpConns = Get-NetTCPConnection -State Established, Listen | 
                    Group-Object OwningProcess
                
                # Get UDP endpoints grouped by process  
                $udpConns = Get-NetUDPEndpoint | Group-Object OwningProcess
                
                # Build process list
                $processMap = @{}
                
                foreach ($group in $tcpConns) {
                    $pid = [int]$group.Name
                    if (-not $processMap.ContainsKey($pid)) {
                        $proc = Get-Process -Id $pid -ErrorAction SilentlyContinue
                        if ($proc) {
                            $processMap[$pid] = @{
                                pid = $pid
                                name = $proc.ProcessName
                                tcp_connections = 0
                                udp_connections = 0
                                bytes_sent = 0
                                bytes_received = 0
                            }
                        }
                    }
                    if ($processMap.ContainsKey($pid)) {
                        $processMap[$pid].tcp_connections = $group.Count
                    }
                }
                
                foreach ($group in $udpConns) {
                    $pid = [int]$group.Name
                    if (-not $processMap.ContainsKey($pid)) {
                        $proc = Get-Process -Id $pid -ErrorAction SilentlyContinue
                        if ($proc) {
                            $processMap[$pid] = @{
                                pid = $pid
                                name = $proc.ProcessName
                                tcp_connections = 0
                                udp_connections = 0
                                bytes_sent = 0
                                bytes_received = 0
                            }
                        }
                    }
                    if ($processMap.ContainsKey($pid)) {
                        $processMap[$pid].udp_connections = $group.Count
                    }
                }
                
                @($processMap.Values) | ConvertTo-Json -Compress
                "#,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_str = stdout.trim();

        if json_str.is_empty() || json_str == "null" || json_str == "@()" {
            return Ok(vec![]);
        }

        // PowerShell returns single object without array brackets if only one result
        let processes: Vec<NetworkProcess> = if json_str.starts_with('[') {
            serde_json::from_str(json_str).unwrap_or_default()
        } else if json_str.starts_with('{') {
            serde_json::from_str::<NetworkProcess>(json_str)
                .map(|p| vec![p])
                .unwrap_or_default()
        } else {
            vec![]
        };

        Ok(processes)
    }

    #[cfg(not(windows))]
    {
        Ok(vec![])
    }
}

/// Gets network interface statistics
#[tauri::command]
pub async fn get_network_interfaces() -> Result<Vec<NetworkInterface>, String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy", "Bypass",
                "-WindowStyle", "Hidden",
                "-Command",
                r#"
                $ErrorActionPreference = 'SilentlyContinue'
                $adapters = Get-NetAdapter | Where-Object { $_.Status -eq 'Up' }
                $result = @()
                
                foreach ($adapter in $adapters) {
                    $stats = Get-NetAdapterStatistics -Name $adapter.Name -ErrorAction SilentlyContinue
                    $result += @{
                        name = $adapter.Name
                        description = $adapter.InterfaceDescription
                        bytes_sent = if ($stats) { [long]$stats.SentBytes } else { 0 }
                        bytes_received = if ($stats) { [long]$stats.ReceivedBytes } else { 0 }
                        status = $adapter.Status
                    }
                }
                
                $result | ConvertTo-Json -Compress
                "#,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_str = stdout.trim();

        if json_str.is_empty() || json_str == "null" {
            return Ok(vec![]);
        }

        let interfaces: Vec<NetworkInterface> = if json_str.starts_with('[') {
            serde_json::from_str(json_str).unwrap_or_default()
        } else if json_str.starts_with('{') {
            serde_json::from_str::<NetworkInterface>(json_str)
                .map(|i| vec![i])
                .unwrap_or_default()
        } else {
            vec![]
        };

        Ok(interfaces)
    }

    #[cfg(not(windows))]
    {
        Ok(vec![])
    }
}
