use crate::context::ExecutionContext;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionState {
    pub progress: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

/// Reads the current execution state from the state file.
pub fn read_state(ctx: &ExecutionContext) -> Option<ExecutionState> {
    if !ctx.state_path.exists() {
        return None;
    }

    fs::read_to_string(&ctx.state_path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
}

/// Reads all log entries from the log file.
pub fn read_logs(ctx: &ExecutionContext) -> Vec<LogEntry> {
    if !ctx.log_path.exists() {
        return Vec::new();
    }

    fs::read_to_string(&ctx.log_path)
        .map(|content| {
            content
                .lines()
                .filter_map(|line| parse_log_line(line))
                .collect()
        })
        .unwrap_or_default()
}

/// Reads error content from the error file.
pub fn read_errors(ctx: &ExecutionContext) -> Option<String> {
    if !ctx.error_path.exists() {
        return None;
    }

    fs::read_to_string(&ctx.error_path).ok()
}

/// Reads the exit code from the exit code file.
pub fn read_exit_code(ctx: &ExecutionContext) -> Option<i32> {
    if !ctx.exit_code_path.exists() {
        return None;
    }

    fs::read_to_string(&ctx.exit_code_path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// Parses a log line in format: [timestamp] [LEVEL] message
fn parse_log_line(line: &str) -> Option<LogEntry> {
    // Format: [2024-01-01 12:00:00] [INFO] message
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // Simple parsing - find the brackets
    let mut parts = line.splitn(3, "] ");
    let timestamp = parts.next()?.trim_start_matches('[').to_string();
    let level = parts.next()?.trim_start_matches('[').to_string();
    let message = parts.next().unwrap_or("").to_string();

    Some(LogEntry {
        timestamp,
        level,
        message,
    })
}
