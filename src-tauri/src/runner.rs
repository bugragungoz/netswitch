use crate::context::ExecutionContext;
use crate::errors::{AppError, AppResult};
use crate::wrapper::wrap_script;
use std::fs;
use std::process::Command;

/// Returns true if the current process is running with administrator privileges.
#[cfg(windows)]
fn current_process_is_elevated() -> bool {
    is_elevated::is_elevated()
}

#[cfg(not(windows))]
fn current_process_is_elevated() -> bool {
    false
}

/// Runs the script with elevation only if the current process is not already elevated.
/// When the app is run as Administrator, this avoids repeated UAC prompts.
pub fn run_script_with_elevation(ctx: &ExecutionContext, script_content: &str) -> AppResult<i32> {
    if current_process_is_elevated() {
        run_script(ctx, script_content)
    } else {
        run_elevated_script(ctx, script_content)
    }
}

/// Executes a PowerShell script with admin elevation via Start-Process -Verb RunAs.
/// Returns the exit code from the script.
/// Windows are completely hidden for a clean user experience.
pub fn run_elevated_script(ctx: &ExecutionContext, script_content: &str) -> AppResult<i32> {
    // Wrap the script with logging/progress infrastructure
    let wrapped_script = wrap_script(ctx, script_content);

    // Write the wrapped script to temp file
    fs::write(&ctx.script_path, &wrapped_script).map_err(|e| {
        AppError::FileError(format!(
            "Failed to write script to {}: {}",
            ctx.script_path.display(),
            e
        ))
    })?;

    // Create initial state file
    fs::write(&ctx.state_path, r#"{"progress":0,"status":"Starting"}"#)?;

    let script_path_str = ctx.script_path.to_string_lossy();

    // Use Start-Process with -Verb RunAs and -WindowStyle Hidden
    // The elevated process runs hidden, only UAC prompt is shown
    let elevation_script = format!(
        r#"Start-Process powershell -Verb RunAs -Wait -WindowStyle Hidden -ArgumentList '-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "{}"'"#,
        script_path_str
    );

    // Execute the elevation command with hidden window
    #[cfg(windows)]
    let output = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy", "Bypass",
                "-WindowStyle", "Hidden",
                "-Command", &elevation_script
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| AppError::ScriptError(format!("Failed to start PowerShell: {}", e)))?
    };

    #[cfg(not(windows))]
    let output = Command::new("powershell")
        .args(["-ExecutionPolicy", "Bypass", "-Command", &elevation_script])
        .output()
        .map_err(|e| AppError::ScriptError(format!("Failed to start PowerShell: {}", e)))?;

    // Check for UAC cancellation (exit code 1223 or stderr contains "cancel")
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.to_lowercase().contains("cancel")
        || stderr.contains("1223")
        || output.status.code() == Some(1223)
    {
        return Err(AppError::UacCancelled);
    }

    // Read exit code from file if available
    let exit_code = if ctx.exit_code_path.exists() {
        fs::read_to_string(&ctx.exit_code_path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(output.status.code().unwrap_or(1))
    } else {
        output.status.code().unwrap_or(1)
    };

    Ok(exit_code)
}

/// Runs a non-elevated PowerShell script with hidden window.
pub fn run_script(ctx: &ExecutionContext, script_content: &str) -> AppResult<i32> {
    let wrapped_script = wrap_script(ctx, script_content);

    fs::write(&ctx.script_path, &wrapped_script)?;
    fs::write(&ctx.state_path, r#"{"progress":0,"status":"Starting"}"#)?;

    let script_path_str = ctx.script_path.to_string_lossy();

    #[cfg(windows)]
    let output = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy", "Bypass",
                "-WindowStyle", "Hidden",
                "-File", &script_path_str.to_string(),
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| AppError::ScriptError(format!("Failed to start PowerShell: {}", e)))?
    };

    #[cfg(not(windows))]
    let output = Command::new("powershell")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_path_str.to_string(),
        ])
        .output()
        .map_err(|e| AppError::ScriptError(format!("Failed to start PowerShell: {}", e)))?;

    let exit_code = if ctx.exit_code_path.exists() {
        fs::read_to_string(&ctx.exit_code_path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(output.status.code().unwrap_or(1))
    } else {
        output.status.code().unwrap_or(1)
    };

    Ok(exit_code)
}
