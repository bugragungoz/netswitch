use serde::Serialize;
use crate::SystemState;

#[derive(Debug, Clone, Serialize)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub ram_used_gb: f32,
    pub ram_total_gb: f32,
}

/// Gets system CPU and RAM statistics using sysinfo (native Rust - much faster than PowerShell)
#[tauri::command]
pub async fn get_system_stats(
    state: tauri::State<'_, SystemState>,
) -> Result<SystemStats, String> {
    // Lock the shared System state (tokio async mutex)
    let mut sys = state.0.lock().await;
    
    // Refresh CPU usage twice with delay for accurate reading
    sys.refresh_cpu_usage();
    
    // Drop lock, wait, then reacquire to avoid holding lock during sleep
    drop(sys);
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let mut sys = state.0.lock().await;
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
