import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
    Settings,
    Save,
    RotateCcw,
    FolderOpen,
    Download,
    Upload,
    Info,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Separator } from "@/components/ui/separator";
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";

interface AppSettings {
    auto_create_restore_point: boolean;
    show_confirmation_dialogs: boolean;
    include_subdirectories: boolean;
    block_exe_by_default: boolean;
    block_dll_by_default: boolean;
    default_excluded_keywords: string[];
    default_excluded_files: string[];
    custom_rule_prefix: string;
    enable_detailed_logging: boolean;
    log_retention_days: number;
    cache_firewall_rules: boolean;
    debug_mode: boolean;
    check_for_updates: boolean;
    run_at_startup: boolean;
}

interface AppInfo {
    version: string;
    build_date: string;
    settings_path: string;
    logs_path: string;
}

const defaultSettings: AppSettings = {
    auto_create_restore_point: false,
    show_confirmation_dialogs: true,
    include_subdirectories: true,
    block_exe_by_default: true,
    block_dll_by_default: true,
    default_excluded_keywords: ["uninstall", "updater", "helper"],
    default_excluded_files: [],
    custom_rule_prefix: "AppBlocker Rule -",
    enable_detailed_logging: false,
    log_retention_days: 30,
    cache_firewall_rules: true,
    debug_mode: false,
    check_for_updates: true,
    run_at_startup: false,
};

export function SettingsPage() {
    const [settings, setSettings] = useState<AppSettings>(defaultSettings);
    const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
    const [isSaving, setIsSaving] = useState(false);
    const [saveMessage, setSaveMessage] = useState<string | null>(null);
    const [excludedKeywordsText, setExcludedKeywordsText] = useState("");
    const [excludedFilesText, setExcludedFilesText] = useState("");

    useEffect(() => {
        loadSettings();
        loadAppInfo();
    }, []);

    useEffect(() => {
        setExcludedKeywordsText(settings.default_excluded_keywords.join(", "));
        setExcludedFilesText(settings.default_excluded_files.join(", "));
    }, [settings.default_excluded_keywords, settings.default_excluded_files]);

    const loadSettings = async () => {
        try {
            const loaded = await invoke<AppSettings>("load_settings");
            setSettings(loaded);
        } catch (error) {
            console.error("Failed to load settings:", error);
        }
    };

    const loadAppInfo = async () => {
        try {
            const info = await invoke<AppInfo>("get_app_info");
            setAppInfo(info);
        } catch (error) {
            console.error("Failed to load app info:", error);
        }
    };

    const handleSave = async () => {
        setIsSaving(true);
        setSaveMessage(null);

        const updatedSettings = {
            ...settings,
            default_excluded_keywords: excludedKeywordsText
                .split(",")
                .map((s) => s.trim())
                .filter((s) => s),
            default_excluded_files: excludedFilesText
                .split(",")
                .map((s) => s.trim())
                .filter((s) => s),
        };

        try {
            await invoke("save_settings", { settings: updatedSettings });
            setSettings(updatedSettings);
            setSaveMessage("Settings saved successfully");
            setTimeout(() => setSaveMessage(null), 3000);
        } catch (error) {
            setSaveMessage(`Failed to save: ${error}`);
        } finally {
            setIsSaving(false);
        }
    };

    const handleReset = async () => {
        try {
            const resetted = await invoke<AppSettings>("reset_settings");
            setSettings(resetted);
            setSaveMessage("Settings reset to defaults");
            setTimeout(() => setSaveMessage(null), 3000);
        } catch (error) {
            setSaveMessage(`Failed to reset: ${error}`);
        }
    };

    const handleOpenLogsFolder = async () => {
        try {
            await invoke("open_logs_folder");
        } catch (error) {
            console.error("Failed to open logs folder:", error);
        }
    };

    const handleExport = async () => {
        try {
            const json = await invoke<string>("export_settings");
            const blob = new Blob([json], { type: "application/json" });
            const url = URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = url;
            a.download = "app-blocker-settings.json";
            a.click();
            URL.revokeObjectURL(url);
        } catch (error) {
            console.error("Failed to export:", error);
        }
    };

    const handleImport = () => {
        const input = document.createElement("input");
        input.type = "file";
        input.accept = ".json";
        input.onchange = async (e) => {
            const file = (e.target as HTMLInputElement).files?.[0];
            if (!file) return;

            const text = await file.text();
            try {
                const imported = await invoke<AppSettings>("import_settings", {
                    jsonContent: text,
                });
                setSettings(imported);
                setSaveMessage("Settings imported successfully");
                setTimeout(() => setSaveMessage(null), 3000);
            } catch (error) {
                setSaveMessage(`Import failed: ${error}`);
            }
        };
        input.click();
    };

    const updateSetting = <K extends keyof AppSettings>(
        key: K,
        value: AppSettings[K]
    ) => {
        setSettings((prev) => ({ ...prev, [key]: value }));
    };

    return (
        <div className="space-y-4 pb-12">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <Settings className="h-5 w-5 text-primary" />
                    <h2 className="text-lg font-semibold">Settings</h2>
                </div>
                <div className="flex gap-2">
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={handleReset}
                        disabled={isSaving}
                    >
                        <RotateCcw className="mr-1.5 h-3.5 w-3.5" />
                        Reset
                    </Button>
                    <Button size="sm" onClick={handleSave} disabled={isSaving}>
                        <Save className="mr-1.5 h-3.5 w-3.5" />
                        {isSaving ? "Saving..." : "Save"}
                    </Button>
                </div>
            </div>

            {saveMessage && (
                <div
                    className={`rounded-md p-2 text-xs ${saveMessage.includes("Failed") || saveMessage.includes("failed")
                        ? "border border-destructive/50 bg-destructive/10 text-destructive"
                        : "border border-green-500/50 bg-green-500/10 text-green-600 dark:text-green-400"
                        }`}
                >
                    {saveMessage}
                </div>
            )}

            {/* General Settings */}
            <Card>
                <CardHeader className="pb-3">
                    <CardTitle className="text-sm">General</CardTitle>
                    <CardDescription className="text-xs">
                        Basic application behavior
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                    <div className="flex items-center justify-between">
                        <Label htmlFor="confirmDialogs" className="text-xs">
                            Show confirmation dialogs
                        </Label>
                        <Switch
                            id="confirmDialogs"
                            checked={settings.show_confirmation_dialogs}
                            onCheckedChange={(v) =>
                                updateSetting("show_confirmation_dialogs", v)
                            }
                        />
                    </div>
                    <div className="flex items-center justify-between">
                        <Label htmlFor="restorePoint" className="text-xs">
                            Auto-create restore point before blocking
                        </Label>
                        <Switch
                            id="restorePoint"
                            checked={settings.auto_create_restore_point}
                            onCheckedChange={(v) =>
                                updateSetting("auto_create_restore_point", v)
                            }
                        />
                    </div>
                    <div className="flex items-center justify-between">
                        <Label htmlFor="subdirs" className="text-xs">
                            Include subdirectories by default
                        </Label>
                        <Switch
                            id="subdirs"
                            checked={settings.include_subdirectories}
                            onCheckedChange={(v) => updateSetting("include_subdirectories", v)}
                        />
                    </div>
                    <div className="flex items-center justify-between">
                        <Label htmlFor="startup" className="text-xs">
                            Run at Windows startup
                        </Label>
                        <Switch
                            id="startup"
                            checked={settings.run_at_startup}
                            onCheckedChange={(v) => updateSetting("run_at_startup", v)}
                        />
                    </div>
                </CardContent>
            </Card>

            {/* Blocking Defaults */}
            <Card>
                <CardHeader className="pb-3">
                    <CardTitle className="text-sm">Blocking Defaults</CardTitle>
                    <CardDescription className="text-xs">
                        Default settings for blocking operations
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                    <div className="flex items-center justify-between">
                        <Label htmlFor="blockExe" className="text-xs">
                            Block .exe files by default
                        </Label>
                        <Switch
                            id="blockExe"
                            checked={settings.block_exe_by_default}
                            onCheckedChange={(v) => updateSetting("block_exe_by_default", v)}
                        />
                    </div>
                    <div className="flex items-center justify-between">
                        <Label htmlFor="blockDll" className="text-xs">
                            Block .dll files by default
                        </Label>
                        <Switch
                            id="blockDll"
                            checked={settings.block_dll_by_default}
                            onCheckedChange={(v) => updateSetting("block_dll_by_default", v)}
                        />
                    </div>

                    <Separator className="my-2" />

                    <div className="space-y-1.5">
                        <Label htmlFor="rulePrefix" className="text-xs">
                            Custom rule name prefix
                        </Label>
                        <Input
                            id="rulePrefix"
                            value={settings.custom_rule_prefix}
                            onChange={(e) =>
                                updateSetting("custom_rule_prefix", e.target.value)
                            }
                            className="h-8 text-xs"
                        />
                    </div>

                    <div className="space-y-1.5">
                        <Label htmlFor="excludedKeywords" className="text-xs">
                            Default excluded keywords (comma-separated)
                        </Label>
                        <Input
                            id="excludedKeywords"
                            value={excludedKeywordsText}
                            onChange={(e) => setExcludedKeywordsText(e.target.value)}
                            placeholder="uninstall, updater, helper"
                            className="h-8 text-xs"
                        />
                    </div>

                    <div className="space-y-1.5">
                        <Label htmlFor="excludedFiles" className="text-xs">
                            Default excluded files (comma-separated)
                        </Label>
                        <Input
                            id="excludedFiles"
                            value={excludedFilesText}
                            onChange={(e) => setExcludedFilesText(e.target.value)}
                            placeholder="setup.exe, config.dll"
                            className="h-8 text-xs"
                        />
                    </div>
                </CardContent>
            </Card>

            {/* Performance */}
            <Card>
                <CardHeader className="pb-3">
                    <CardTitle className="text-sm">Performance</CardTitle>
                    <CardDescription className="text-xs">
                        Logging and caching options
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                    <div className="flex items-center justify-between">
                        <Label htmlFor="detailedLog" className="text-xs">
                            Enable detailed logging
                        </Label>
                        <Switch
                            id="detailedLog"
                            checked={settings.enable_detailed_logging}
                            onCheckedChange={(v) =>
                                updateSetting("enable_detailed_logging", v)
                            }
                        />
                    </div>
                    <div className="flex items-center justify-between">
                        <Label htmlFor="cacheRules" className="text-xs">
                            Cache firewall rules
                        </Label>
                        <Switch
                            id="cacheRules"
                            checked={settings.cache_firewall_rules}
                            onCheckedChange={(v) => updateSetting("cache_firewall_rules", v)}
                        />
                    </div>

                    <div className="space-y-1.5">
                        <Label htmlFor="logRetention" className="text-xs">
                            Log retention (days)
                        </Label>
                        <Input
                            id="logRetention"
                            type="number"
                            min={1}
                            max={365}
                            value={settings.log_retention_days}
                            onChange={(e) =>
                                updateSetting("log_retention_days", parseInt(e.target.value) || 30)
                            }
                            className="h-8 w-24 text-xs"
                        />
                    </div>
                </CardContent>
            </Card>

            {/* Advanced */}
            <Card>
                <CardHeader className="pb-3">
                    <CardTitle className="text-sm">Advanced</CardTitle>
                    <CardDescription className="text-xs">
                        Developer and debug options
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                    <div className="flex items-center justify-between">
                        <Label htmlFor="debugMode" className="text-xs">
                            Debug mode (verbose logging)
                        </Label>
                        <Switch
                            id="debugMode"
                            checked={settings.debug_mode}
                            onCheckedChange={(v) => updateSetting("debug_mode", v)}
                        />
                    </div>
                    <div className="flex items-center justify-between">
                        <Label htmlFor="checkUpdates" className="text-xs">
                            Check for updates on startup
                        </Label>
                        <Switch
                            id="checkUpdates"
                            checked={settings.check_for_updates}
                            onCheckedChange={(v) => updateSetting("check_for_updates", v)}
                        />
                    </div>
                </CardContent>
            </Card>

            {/* Application Info */}
            <Card>
                <CardHeader className="pb-3">
                    <CardTitle className="text-sm flex items-center gap-1.5">
                        <Info className="h-3.5 w-3.5" />
                        Application Info
                    </CardTitle>
                </CardHeader>
                <CardContent className="space-y-3">
                    {appInfo && (
                        <div className="space-y-1 text-xs text-muted-foreground">
                            <p>
                                <span className="font-medium text-foreground">Version:</span>{" "}
                                {appInfo.version}
                            </p>
                            <p>
                                <span className="font-medium text-foreground">Build Date:</span>{" "}
                                {appInfo.build_date}
                            </p>
                            <p className="break-all">
                                <span className="font-medium text-foreground">
                                    Settings Path:
                                </span>{" "}
                                {appInfo.settings_path}
                            </p>
                        </div>
                    )}

                    <Separator className="my-2" />

                    <div className="flex flex-wrap gap-2">
                        <Button variant="outline" size="sm" onClick={handleOpenLogsFolder}>
                            <FolderOpen className="mr-1.5 h-3.5 w-3.5" />
                            Open Logs
                        </Button>
                        <Button variant="outline" size="sm" onClick={handleExport}>
                            <Download className="mr-1.5 h-3.5 w-3.5" />
                            Export
                        </Button>
                        <Button variant="outline" size="sm" onClick={handleImport}>
                            <Upload className="mr-1.5 h-3.5 w-3.5" />
                            Import
                        </Button>
                    </div>
                </CardContent>
            </Card>
        </div>
    );
}
