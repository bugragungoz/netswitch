use crate::context::ExecutionContext;
use crate::errors::AppError;
use crate::logger::{read_errors, read_logs};
use crate::runner::run_script_with_elevation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRequest {
    pub app_name: String,
    pub app_path: String,
    pub file_extensions: Vec<String>,
    pub excluded_keywords: Vec<String>,
    pub excluded_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BlockResult {
    pub success: bool,
    pub logs: Vec<String>,
    pub error: Option<String>,
    pub rules_created: i32,
    pub files_processed: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub name: String,
    pub direction: String,
    pub program: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RulesResult {
    pub success: bool,
    pub rules: Vec<FirewallRule>,
    pub error: Option<String>,
}

/// Blocks internet access for an application by creating firewall rules.
#[tauri::command]
pub async fn block_application(request: BlockRequest) -> Result<BlockResult, String> {
    let ctx = ExecutionContext::new();

    // Build PowerShell script for blocking
    let extensions_str = request
        .file_extensions
        .iter()
        .map(|e| format!(r#""{}""#, e))
        .collect::<Vec<_>>()
        .join(", ");

    let excluded_keywords_str = request
        .excluded_keywords
        .iter()
        .map(|k| format!(r#""{}""#, k))
        .collect::<Vec<_>>()
        .join(", ");

    let excluded_files_str = request
        .excluded_files
        .iter()
        .map(|f| format!(r#""{}""#, f))
        .collect::<Vec<_>>()
        .join(", ");

    let script = format!(
        r#"
    # Admin check
    $isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if (-not $isAdmin) {{ throw "This script must run as administrator" }}

    $AppName = "{app_name}"
    $AppPath = "{app_path}"
    $FileExtensions = @({extensions})
    $ExcludedKeywords = @({excluded_keywords})
    $ExcludedFiles = @({excluded_files})
    $RuleNamePrefix = "AppBlocker Rule -"

    Write-CroxzState -Progress 5 -Status "Validating path"
    Write-CroxzLog "Starting blocking process for: $AppName"

    if (-not (Test-Path -Path $AppPath -PathType Container)) {{
        throw "The specified path '$AppPath' was not found or is not a valid directory."
    }}

    Write-CroxzState -Progress 15 -Status "Scanning files"
    Write-CroxzLog "Scanning directory: $AppPath"

    $filesToProcess = @()
    foreach ($ext in $FileExtensions) {{
        $filesToProcess += Get-ChildItem -Path $AppPath -Recurse -Filter $ext -File -ErrorAction SilentlyContinue
    }}

    Write-CroxzLog "Found $($filesToProcess.Count) files matching extensions"

    if ($filesToProcess.Count -eq 0) {{
        Write-CroxzLog "No files found matching extensions" -Level "WARN"
        Write-CroxzState -Progress 100 -Status "Completed"
        exit 0
    }}

    # Filter exclusions
    Write-CroxzState -Progress 25 -Status "Applying exclusions"
    $filesToBlock = @()

    foreach ($file in $filesToProcess) {{
        $shouldSkip = $false

        if ($ExcludedFiles -contains $file.Name) {{
            $shouldSkip = $true
            Write-CroxzLog "Skipping file (excluded): $($file.Name)" -Level "DEBUG"
        }}

        if (-not $shouldSkip -and $ExcludedKeywords.Count -gt 0) {{
            foreach ($keyword in $ExcludedKeywords) {{
                if ($file.Name -like "*$keyword*") {{
                    $shouldSkip = $true
                    Write-CroxzLog "Skipping file (keyword): $($file.Name)" -Level "DEBUG"
                    break
                }}
            }}
        }}

        if (-not $shouldSkip) {{
            $filesToBlock += $file
        }}
    }}

    Write-CroxzLog "Files to block after exclusions: $($filesToBlock.Count)"

    if ($filesToBlock.Count -eq 0) {{
        Write-CroxzLog "No files remaining after exclusions" -Level "WARN"
        Write-CroxzState -Progress 100 -Status "Completed"
        exit 0
    }}

    # Create firewall rules
    Write-CroxzState -Progress 35 -Status "Creating rules"
    $totalFiles = $filesToBlock.Count
    $currentIndex = 0
    $successCount = 0
    $errorCount = 0

    foreach ($file in $filesToBlock) {{
        $currentIndex++
        $progress = 35 + [int](($currentIndex / $totalFiles) * 60)
        Write-CroxzState -Progress $progress -Status "Processing $currentIndex of $totalFiles"

        $baseRuleName = "$($RuleNamePrefix)$($AppName) - $($file.Name)"
        if ($baseRuleName.Length -gt 220) {{
            $baseRuleName = $baseRuleName.Substring(0, 220) + "..."
        }}

        # Inbound rule
        $inboundName = "$baseRuleName (Inbound)"
        $existingInbound = Get-NetFirewallRule -DisplayName $inboundName -ErrorAction SilentlyContinue
        if (-not $existingInbound) {{
            try {{
                New-NetFirewallRule -DisplayName $inboundName -Direction Inbound -Program $file.FullName -Action Block -Profile Any -Enabled True -ErrorAction Stop | Out-Null
                Write-CroxzLog "Created inbound rule: $inboundName"
            }} catch {{
                Write-CroxzLog "Failed to create inbound rule: $($_.Exception.Message)" -Level "ERROR"
                $errorCount++
            }}
        }} else {{
            Write-CroxzLog "Rule already exists: $inboundName" -Level "DEBUG"
        }}

        # Outbound rule
        $outboundName = "$baseRuleName (Outbound)"
        $existingOutbound = Get-NetFirewallRule -DisplayName $outboundName -ErrorAction SilentlyContinue
        if (-not $existingOutbound) {{
            try {{
                New-NetFirewallRule -DisplayName $outboundName -Direction Outbound -Program $file.FullName -Action Block -Profile Any -Enabled True -ErrorAction Stop | Out-Null
                Write-CroxzLog "Created outbound rule: $outboundName"
                $successCount++
            }} catch {{
                Write-CroxzLog "Failed to create outbound rule: $($_.Exception.Message)" -Level "ERROR"
                $errorCount++
            }}
        }} else {{
            Write-CroxzLog "Rule already exists: $outboundName" -Level "DEBUG"
            $successCount++
        }}
    }}

    Write-CroxzState -Progress 95 -Status "Finalizing"
    Write-CroxzLog "Blocking complete. Success: $successCount, Errors: $errorCount"
"#,
        app_name = request.app_name.replace("\"", "\\\""),
        app_path = request.app_path.replace("\\", "\\\\").replace("\"", "\\\""),
        extensions = extensions_str,
        excluded_keywords = excluded_keywords_str,
        excluded_files = excluded_files_str
    );

    // Run the script with elevation
    let result = run_script_with_elevation(&ctx, &script);

    // Collect logs
    let logs: Vec<String> = read_logs(&ctx)
        .iter()
        .map(|l| format!("[{}] {}", l.level, l.message))
        .collect();

    let error = read_errors(&ctx);

    // Clean up temp files
    ctx.cleanup();

    match result {
        Ok(exit_code) => {
            if exit_code == 0 {
                Ok(BlockResult {
                    success: true,
                    logs,
                    error: None,
                    rules_created: 0, // Would need to parse from logs
                    files_processed: 0,
                })
            } else {
                Ok(BlockResult {
                    success: false,
                    logs,
                    error: error.or(Some("Script execution failed".to_string())),
                    rules_created: 0,
                    files_processed: 0,
                })
            }
        }
        Err(AppError::UacCancelled) => Ok(BlockResult {
            success: false,
            logs,
            error: Some("Operation cancelled - UAC prompt was declined".to_string()),
            rules_created: 0,
            files_processed: 0,
        }),
        Err(e) => Err(e.to_string()),
    }
}

/// Gets firewall rules created by this application.
#[tauri::command]
pub async fn get_firewall_rules() -> Result<RulesResult, String> {
    let ctx = ExecutionContext::new();

    let script = r#"
    $isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if (-not $isAdmin) { throw "This script must run as administrator" }

    Write-CroxzState -Progress 25 -Status "Fetching rules"
    Write-CroxzLog "Searching for AppBlocker rules"

    $rules = Get-NetFirewallRule -DisplayName "AppBlocker Rule -*" -ErrorAction SilentlyContinue

    if ($rules.Count -eq 0) {
        Write-CroxzLog "No AppBlocker rules found"
        Write-CroxzState -Progress 100 -Status "Completed"
        Write-Host "RULES_JSON:[]"
        exit 0
    }

    Write-CroxzState -Progress 50 -Status "Processing rules"
    Write-CroxzLog "Found $($rules.Count) rules"

    $rulesData = @()
    foreach ($rule in $rules) {
        $filter = Get-NetFirewallApplicationFilter -AssociatedNetFirewallRule $rule -ErrorAction SilentlyContinue
        $program = if ($filter) { $filter.Program } else { "Unknown" }

        $rulesData += @{
            name = $rule.DisplayName
            direction = $rule.Direction.ToString()
            program = $program
            enabled = $rule.Enabled -eq "True"
        }
    }

    $json = $rulesData | ConvertTo-Json -Compress
    Write-Host "RULES_JSON:$json"
    Write-CroxzState -Progress 100 -Status "Completed"
"#;

    let result = run_script_with_elevation(&ctx, script);

    let logs: Vec<String> = read_logs(&ctx)
        .iter()
        .map(|l| l.message.clone())
        .collect();

    ctx.cleanup();

    match result {
        Ok(_) => {
            // Parse rules from logs
            let rules_json = logs
                .iter()
                .find(|l| l.starts_with("RULES_JSON:"))
                .map(|l| l.trim_start_matches("RULES_JSON:"))
                .unwrap_or("[]");

            let rules: Vec<FirewallRule> = serde_json::from_str(rules_json).unwrap_or_default();

            Ok(RulesResult {
                success: true,
                rules,
                error: None,
            })
        }
        Err(AppError::UacCancelled) => Ok(RulesResult {
            success: false,
            rules: vec![],
            error: Some("Operation cancelled".to_string()),
        }),
        Err(e) => Err(e.to_string()),
    }
}

/// Removes firewall rules by name pattern.
#[tauri::command]
pub async fn remove_firewall_rules(app_name: Option<String>) -> Result<BlockResult, String> {
    let ctx = ExecutionContext::new();

    let pattern = match &app_name {
        Some(name) => format!("AppBlocker Rule -{} -*", name),
        None => "AppBlocker Rule -*".to_string(),
    };

    let script = format!(
        r#"
    $isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if (-not $isAdmin) {{ throw "This script must run as administrator" }}

    Write-CroxzState -Progress 25 -Status "Finding rules"
    Write-CroxzLog "Searching for rules matching: {pattern}"

    $rules = Get-NetFirewallRule -DisplayName "{pattern}" -ErrorAction SilentlyContinue

    if ($rules.Count -eq 0) {{
        Write-CroxzLog "No rules found to remove"
        Write-CroxzState -Progress 100 -Status "Completed"
        exit 0
    }}

    Write-CroxzState -Progress 50 -Status "Removing rules"
    Write-CroxzLog "Removing $($rules.Count) rules"

    $rules | Remove-NetFirewallRule -ErrorAction Stop
    Write-CroxzLog "Successfully removed $($rules.Count) rules"
    Write-CroxzState -Progress 100 -Status "Completed"
"#,
        pattern = pattern
    );

    let result = run_script_with_elevation(&ctx, &script);

    let logs: Vec<String> = read_logs(&ctx)
        .iter()
        .map(|l| format!("[{}] {}", l.level, l.message))
        .collect();

    let error = read_errors(&ctx);
    ctx.cleanup();

    match result {
        Ok(exit_code) => Ok(BlockResult {
            success: exit_code == 0,
            logs,
            error: if exit_code == 0 { None } else { error },
            rules_created: 0,
            files_processed: 0,
        }),
        Err(AppError::UacCancelled) => Ok(BlockResult {
            success: false,
            logs,
            error: Some("Operation cancelled - UAC prompt was declined".to_string()),
            rules_created: 0,
            files_processed: 0,
        }),
        Err(e) => Err(e.to_string()),
    }
}

/// Opens Windows Firewall with Advanced Security (wf.msc).
#[tauri::command]
pub async fn open_windows_firewall() -> Result<bool, String> {
    std::process::Command::new("mmc")
        .arg("wf.msc")
        .spawn()
        .map_err(|e| format!("Failed to open Windows Firewall: {}", e))?;

    Ok(true)
}

/// Creates a system restore point.
#[tauri::command]
pub async fn create_restore_point(description: String) -> Result<BlockResult, String> {
    let ctx = ExecutionContext::new();

    let script = format!(
        r#"
    $isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if (-not $isAdmin) {{ throw "This script must run as administrator" }}

    Write-CroxzState -Progress 25 -Status "Creating restore point"
    Write-CroxzLog "Creating system restore point: {description}"

    try {{
        Checkpoint-Computer -Description "{description}" -ErrorAction Stop
        Write-CroxzLog "System restore point created successfully"
        Write-CroxzState -Progress 100 -Status "Completed"
    }}
    catch {{
        throw "Failed to create restore point: $($_.Exception.Message)"
    }}
"#,
        description = description.replace("\"", "\\\"")
    );

    let result = run_script_with_elevation(&ctx, &script);

    let logs: Vec<String> = read_logs(&ctx)
        .iter()
        .map(|l| format!("[{}] {}", l.level, l.message))
        .collect();

    let error = read_errors(&ctx);
    ctx.cleanup();

    match result {
        Ok(exit_code) => Ok(BlockResult {
            success: exit_code == 0,
            logs,
            error: if exit_code == 0 { None } else { error },
            rules_created: 0,
            files_processed: 0,
        }),
        Err(AppError::UacCancelled) => Ok(BlockResult {
            success: false,
            logs,
            error: Some("Operation cancelled - UAC prompt was declined".to_string()),
            rules_created: 0,
            files_processed: 0,
        }),
        Err(e) => Err(e.to_string()),
    }
}

/// Opens a directory selection dialog.
/// Note: In Tauri v2, the dialog is handled directly from the frontend using @tauri-apps/plugin-dialog
#[tauri::command]
pub async fn select_directory() -> Result<Option<String>, String> {
    // The frontend uses the dialog plugin directly
    // This command is kept for compatibility but not used
    Ok(None)
}

#[derive(Debug, Clone, Serialize)]
pub struct FirewallStats {
    pub total_rules: i32,
    pub inbound_rules: i32,
    pub outbound_rules: i32,
    pub app_blocker_rules: i32,
    pub enabled_rules: i32,
    pub disabled_rules: i32,
}

/// Gets firewall rule statistics without requiring elevation.
#[tauri::command]
pub async fn get_firewall_stats() -> Result<FirewallStats, String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        // Run PowerShell without elevation and hidden
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-WindowStyle", "Hidden",
                "-NonInteractive",
                "-Command",
                r#"
                $ErrorActionPreference = 'SilentlyContinue'
                $allRules = Get-NetFirewallRule
                $appBlockerRules = Get-NetFirewallRule -DisplayName "AppBlocker Rule -*"
                
                $stats = @{
                    total = @($allRules).Count
                    inbound = @($allRules | Where-Object { $_.Direction -eq 'Inbound' }).Count
                    outbound = @($allRules | Where-Object { $_.Direction -eq 'Outbound' }).Count
                    appBlocker = @($appBlockerRules).Count
                    enabled = @($allRules | Where-Object { $_.Enabled -eq 'True' }).Count
                    disabled = @($allRules | Where-Object { $_.Enabled -eq 'False' }).Count
                }
                
                $stats | ConvertTo-Json -Compress
                "#,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_str = stdout.trim();

        if json_str.is_empty() {
            return Ok(FirewallStats {
                total_rules: 0,
                inbound_rules: 0,
                outbound_rules: 0,
                app_blocker_rules: 0,
                enabled_rules: 0,
                disabled_rules: 0,
            });
        }

        #[derive(Deserialize)]
        struct RawStats {
            total: Option<i32>,
            inbound: Option<i32>,
            outbound: Option<i32>,
            appBlocker: Option<i32>,
            enabled: Option<i32>,
            disabled: Option<i32>,
        }

        let raw: RawStats = serde_json::from_str(json_str).unwrap_or(RawStats {
            total: Some(0),
            inbound: Some(0),
            outbound: Some(0),
            appBlocker: Some(0),
            enabled: Some(0),
            disabled: Some(0),
        });

        Ok(FirewallStats {
            total_rules: raw.total.unwrap_or(0),
            inbound_rules: raw.inbound.unwrap_or(0),
            outbound_rules: raw.outbound.unwrap_or(0),
            app_blocker_rules: raw.appBlocker.unwrap_or(0),
            enabled_rules: raw.enabled.unwrap_or(0),
            disabled_rules: raw.disabled.unwrap_or(0),
        })
    }

    #[cfg(not(windows))]
    {
        Ok(FirewallStats {
            total_rules: 0,
            inbound_rules: 0,
            outbound_rules: 0,
            app_blocker_rules: 0,
            enabled_rules: 0,
            disabled_rules: 0,
        })
    }
}
