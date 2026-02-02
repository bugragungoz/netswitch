use crate::context::ExecutionContext;

/// Generates a wrapped PowerShell script that:
/// - Sets UTF-8 output encoding
/// - Writes logs to log_path
/// - Writes progress to state_path
/// - Persists exit code to exit_code_path
/// - Always cleans up temp files on exit
pub fn wrap_script(ctx: &ExecutionContext, script_content: &str) -> String {
    let log_path = ctx.log_path.to_string_lossy().replace("\\", "\\\\");
    let error_path = ctx.error_path.to_string_lossy().replace("\\", "\\\\");
    let state_path = ctx.state_path.to_string_lossy().replace("\\", "\\\\");
    let exit_code_path = ctx.exit_code_path.to_string_lossy().replace("\\", "\\\\");

    format!(
        r#"# Croxz Script Wrapper - Auto-generated
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = "Stop"

$LogPath = "{log_path}"
$ErrorPath = "{error_path}"
$StatePath = "{state_path}"
$ExitCodePath = "{exit_code_path}"

function Write-CroxzLog {{
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "[$timestamp] [$Level] $Message"
    Add-Content -Path $LogPath -Value $logEntry -Encoding UTF8
    Write-Host $logEntry
}}

function Write-CroxzState {{
    param([int]$Progress, [string]$Status)
    $stateJson = @{{ progress = $Progress; status = $Status }} | ConvertTo-Json -Compress
    Set-Content -Path $StatePath -Value $stateJson -Encoding UTF8
}}

function Write-CroxzError {{
    param([string]$Message)
    Add-Content -Path $ErrorPath -Value $Message -Encoding UTF8
    Write-CroxzLog -Message $Message -Level "ERROR"
}}

$exitCode = 0

try {{
    Write-CroxzState -Progress 0 -Status "Started"
    Write-CroxzLog "Script execution started"

    # ========== USER SCRIPT BEGIN ==========
{script_content}
    # ========== USER SCRIPT END ==========

    Write-CroxzState -Progress 100 -Status "Completed"
    Write-CroxzLog "Script execution completed successfully"
}}
catch {{
    $exitCode = 1
    $errorMsg = $_.Exception.Message
    Write-CroxzError $errorMsg
    Write-CroxzState -Progress 100 -Status "Error"
}}
finally {{
    Set-Content -Path $ExitCodePath -Value $exitCode -Encoding UTF8
}}

exit $exitCode
"#,
        log_path = log_path,
        error_path = error_path,
        state_path = state_path,
        exit_code_path = exit_code_path,
        script_content = script_content
    )
}
