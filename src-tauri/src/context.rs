use std::env;
use std::path::PathBuf;
use uuid::Uuid;

/// Execution context with temp file paths for a single script execution.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub id: String,
    pub log_path: PathBuf,
    pub error_path: PathBuf,
    pub state_path: PathBuf,
    pub script_path: PathBuf,
    pub exit_code_path: PathBuf,
}

impl ExecutionContext {
    /// Creates a new execution context with unique temp file paths.
    pub fn new() -> Self {
        let id = Uuid::new_v4().to_string();
        let temp_dir = env::temp_dir();

        Self {
            log_path: temp_dir.join(format!("croxz_{}.log", id)),
            error_path: temp_dir.join(format!("croxz_{}.err", id)),
            state_path: temp_dir.join(format!("croxz_{}.state", id)),
            script_path: temp_dir.join(format!("croxz_{}.ps1", id)),
            exit_code_path: temp_dir.join(format!("croxz_{}.exit", id)),
            id,
        }
    }

    /// Cleans up all temp files created by this context.
    pub fn cleanup(&self) {
        let _ = std::fs::remove_file(&self.log_path);
        let _ = std::fs::remove_file(&self.error_path);
        let _ = std::fs::remove_file(&self.state_path);
        let _ = std::fs::remove_file(&self.script_path);
        let _ = std::fs::remove_file(&self.exit_code_path);
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}
