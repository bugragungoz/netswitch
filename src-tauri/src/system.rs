use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub ram_used_gb: f32,
    pub ram_total_gb: f32,
}

/// Gets system CPU and RAM statistics
#[tauri::command]
pub async fn get_system_stats() -> Result<SystemStats, String> {
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
                
                # Get CPU usage
                $cpu = (Get-Counter '\Processor(_Total)\% Processor Time' -ErrorAction SilentlyContinue).CounterSamples[0].CookedValue
                if (-not $cpu) { $cpu = 0 }
                
                # Get RAM info
                $os = Get-CimInstance Win32_OperatingSystem
                $totalRam = [math]::Round($os.TotalVisibleMemorySize / 1MB, 1)
                $freeRam = [math]::Round($os.FreePhysicalMemory / 1MB, 1)
                $usedRam = $totalRam - $freeRam
                
                @{
                    cpu_usage = [math]::Round($cpu, 1)
                    ram_used_gb = $usedRam
                    ram_total_gb = $totalRam
                } | ConvertTo-Json -Compress
                "#,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_str = stdout.trim();

        if json_str.is_empty() {
            return Ok(SystemStats {
                cpu_usage: 0.0,
                ram_used_gb: 0.0,
                ram_total_gb: 0.0,
            });
        }

        #[derive(serde::Deserialize)]
        struct RawStats {
            cpu_usage: Option<f32>,
            ram_used_gb: Option<f32>,
            ram_total_gb: Option<f32>,
        }

        let raw: RawStats = serde_json::from_str(json_str).unwrap_or(RawStats {
            cpu_usage: Some(0.0),
            ram_used_gb: Some(0.0),
            ram_total_gb: Some(0.0),
        });

        Ok(SystemStats {
            cpu_usage: raw.cpu_usage.unwrap_or(0.0),
            ram_used_gb: raw.ram_used_gb.unwrap_or(0.0),
            ram_total_gb: raw.ram_total_gb.unwrap_or(0.0),
        })
    }

    #[cfg(not(windows))]
    {
        Ok(SystemStats {
            cpu_usage: 0.0,
            ram_used_gb: 0.0,
            ram_total_gb: 0.0,
        })
    }
}
