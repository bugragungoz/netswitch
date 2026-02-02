import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
    Shield,
    RefreshCw,
    ExternalLink,
    Trash2,
    ShieldCheck,
    ShieldAlert,
    Lock,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from "@/components/ui/dialog";
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from "@/components/ui/tooltip";
import { useAdmin } from "@/App";

interface FirewallStats {
    total_rules: number;
    inbound_rules: number;
    outbound_rules: number;
    app_blocker_rules: number;
    enabled_rules: number;
    disabled_rules: number;
}

interface FirewallRule {
    name: string;
    direction: string;
    program: string;
    enabled: boolean;
}

export function WindowsFirewallPage() {
    const { isAdmin, setAdmin } = useAdmin();
    const [stats, setStats] = useState<FirewallStats | null>(null);
    const [rules, setRules] = useState<FirewallRule[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [isRunning, setIsRunning] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [selectedRule, setSelectedRule] = useState<string | null>(null);
    const [showAdminDialog, setShowAdminDialog] = useState(false);
    const [pendingAction, setPendingAction] = useState<"loadRules" | "deleteAll" | null>(null);

    // Load stats on mount (non-admin, silent)
    useEffect(() => {
        loadStats();
    }, []);

    // Load rules when admin mode is enabled
    useEffect(() => {
        if (isAdmin) {
            loadRulesWithAdmin();
        }
    }, [isAdmin]);

    const loadStats = async () => {
        setIsLoading(true);
        setError(null);
        try {
            const fetchedStats = await invoke<FirewallStats>("get_firewall_stats");
            setStats(fetchedStats);
        } catch (err) {
            setError(`Failed to load stats: ${err}`);
        } finally {
            setIsLoading(false);
        }
    };

    const loadRulesWithAdmin = async () => {
        try {
            const result = await invoke<{ success: boolean; rules: FirewallRule[] }>(
                "get_firewall_rules"
            );
            if (result.success) {
                setRules(result.rules);
            }
        } catch (err) {
            console.error("Failed to load rules:", err);
        }
    };

    const handleRefresh = () => {
        loadStats();
        if (isAdmin) {
            loadRulesWithAdmin();
        }
    };

    const handleOpenFirewall = async () => {
        try {
            await invoke("open_windows_firewall");
        } catch (err) {
            setError(`Failed to open Windows Firewall: ${err}`);
        }
    };

    const handleRequestAdmin = (action: "loadRules" | "deleteAll") => {
        setPendingAction(action);
        setShowAdminDialog(true);
    };

    const handleConfirmAdmin = async () => {
        setShowAdminDialog(false);
        setIsRunning(true);
        setError(null);

        try {
            if (pendingAction === "loadRules") {
                // Run an admin command to get rules (triggers UAC via run_script_with_elevation)
                const result = await invoke<{ success: boolean; rules: FirewallRule[] }>(
                    "get_firewall_rules"
                );
                if (result.success) {
                    setRules(result.rules);
                    setAdmin(true);
                } else {
                    setError("Failed to load rules. UAC may have been cancelled.");
                }
            } else if (pendingAction === "deleteAll") {
                const result = await invoke<{ success: boolean; error?: string }>("remove_firewall_rules", {
                    appName: null,
                });
                if (result.success) {
                    await loadRulesWithAdmin();
                    await loadStats();
                } else {
                    setError(result.error || "Failed to delete rules");
                }
            }
        } catch (err) {
            setError(`Operation failed: ${err}`);
        } finally {
            setIsRunning(false);
            setPendingAction(null);
        }
    };

    const handleCancelAdmin = () => {
        setShowAdminDialog(false);
        setPendingAction(null);
    };

    const StatCard = ({
        label,
        value,
        variant = "default",
    }: {
        label: string;
        value: number | string;
        variant?: "default" | "primary" | "success" | "warning";
    }) => (
        <div className="rounded-lg border bg-card p-3 text-center">
            <p className="text-xs text-muted-foreground">{label}</p>
            <p
                className={`text-xl font-bold ${variant === "primary"
                    ? "text-primary"
                    : variant === "success"
                        ? "text-green-600 dark:text-green-400"
                        : variant === "warning"
                            ? "text-amber-600 dark:text-amber-400"
                            : ""
                    }`}
            >
                {value}
            </p>
        </div>
    );

    return (
        <div className="space-y-4 pb-12">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <Shield className="h-5 w-5 text-primary" />
                    <h2 className="text-lg font-semibold">Windows Firewall</h2>
                </div>
                <div className="flex items-center gap-2">
                    {/* Admin Badge - clickable to enable admin mode */}
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <span className="inline-flex">
                                <Button
                                    variant={isAdmin ? "default" : "outline"}
                                    size="sm"
                                    className={`gap-1.5 ${isAdmin ? "bg-green-600 hover:bg-green-700" : "border-amber-500/50 text-amber-600 dark:text-amber-400 hover:bg-amber-500/10"}`}
                                    onClick={() => !isAdmin && handleRequestAdmin("loadRules")}
                                    disabled={isRunning}
                                >
                                    {isAdmin ? (
                                        <>
                                            <ShieldCheck className="h-3.5 w-3.5" />
                                            Admin
                                        </>
                                    ) : (
                                        <>
                                            <Lock className="h-3.5 w-3.5" />
                                            Enable Admin
                                        </>
                                    )}
                                </Button>
                            </span>
                        </TooltipTrigger>
                        <TooltipContent>
                            {isAdmin
                                ? "Admin mode active - full access enabled"
                                : "Click to enable admin mode for rule management"}
                        </TooltipContent>
                    </Tooltip>
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={handleRefresh}
                        disabled={isLoading}
                    >
                        <RefreshCw
                            className={`mr-1.5 h-3.5 w-3.5 ${isLoading ? "animate-spin" : ""}`}
                        />
                        Refresh
                    </Button>
                    <Button variant="outline" size="sm" onClick={handleOpenFirewall}>
                        <ExternalLink className="mr-1.5 h-3.5 w-3.5" />
                        Open wf.msc
                    </Button>
                </div>
            </div>

            {error && (
                <div className="rounded-md border border-destructive/50 bg-destructive/10 p-2 text-xs text-destructive">
                    {error}
                </div>
            )}

            {/* Statistics */}
            <Card>
                <CardHeader className="pb-3">
                    <CardTitle className="text-sm">Firewall Statistics</CardTitle>
                    <CardDescription className="text-xs">
                        Overview of Windows Firewall rules
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    {stats ? (
                        <div className="grid grid-cols-3 gap-3 md:grid-cols-6">
                            <StatCard label="Total Rules" value={stats.total_rules} />
                            <StatCard label="Inbound" value={stats.inbound_rules} />
                            <StatCard label="Outbound" value={stats.outbound_rules} />
                            <StatCard
                                label="NetSwitch"
                                value={stats.app_blocker_rules}
                                variant="primary"
                            />
                            <StatCard
                                label="Enabled"
                                value={stats.enabled_rules}
                                variant="success"
                            />
                            <StatCard
                                label="Disabled"
                                value={stats.disabled_rules}
                                variant="warning"
                            />
                        </div>
                    ) : (
                        <div className="flex h-20 items-center justify-center text-xs text-muted-foreground">
                            {isLoading ? "Loading statistics..." : "No data available"}
                        </div>
                    )}
                </CardContent>
            </Card>

            {/* NetSwitch Rules - requires admin */}
            <Card>
                <CardHeader className="pb-3">
                    <div className="flex items-center justify-between">
                        <div>
                            <CardTitle className="text-sm flex items-center gap-2">
                                NetSwitch Rules
                                {!isAdmin && (
                                    <Badge variant="outline" className="text-[10px] text-amber-600 dark:text-amber-400 border-amber-500/50">
                                        <Lock className="h-2.5 w-2.5 mr-1" />
                                        Admin Required
                                    </Badge>
                                )}
                            </CardTitle>
                            <CardDescription className="text-xs">
                                Firewall rules created by this application
                            </CardDescription>
                        </div>
                        {isAdmin && rules.length > 0 && (
                            <Button
                                variant="destructive"
                                size="sm"
                                onClick={() => handleRequestAdmin("deleteAll")}
                                disabled={isRunning}
                            >
                                <Trash2 className="mr-1.5 h-3.5 w-3.5" />
                                Delete All
                            </Button>
                        )}
                    </div>
                </CardHeader>
                <CardContent>
                    {!isAdmin ? (
                        <div className="flex h-32 flex-col items-center justify-center gap-3 text-center">
                            <ShieldAlert className="h-8 w-8 text-amber-500/50" />
                            <div className="space-y-1">
                                <p className="text-sm font-medium">Admin Access Required</p>
                                <p className="text-xs text-muted-foreground max-w-sm">
                                    Click "Enable Admin" button above to view and manage NetSwitch firewall rules.
                                    This requires Administrator privileges.
                                </p>
                            </div>
                        </div>
                    ) : rules.length > 0 ? (
                        <ScrollArea className="h-64">
                            <div className="space-y-1">
                                {rules.map((rule, index) => (
                                    <div
                                        key={index}
                                        className={`flex items-center justify-between rounded-md border p-2 text-xs ${selectedRule === rule.name
                                            ? "border-primary bg-primary/5"
                                            : "hover:bg-muted/50"
                                            }`}
                                        onClick={() => setSelectedRule(rule.name)}
                                    >
                                        <div className="flex-1 min-w-0">
                                            <p className="font-medium truncate">{rule.name}</p>
                                            <p className="text-muted-foreground truncate">
                                                {rule.program}
                                            </p>
                                        </div>
                                        <div className="flex items-center gap-2 ml-2">
                                            <span
                                                className={`px-1.5 py-0.5 rounded text-[10px] font-medium ${rule.direction === "Inbound"
                                                    ? "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400"
                                                    : "bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400"
                                                    }`}
                                            >
                                                {rule.direction}
                                            </span>
                                            <span
                                                className={`px-1.5 py-0.5 rounded text-[10px] font-medium ${rule.enabled
                                                    ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400"
                                                    : "bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-400"
                                                    }`}
                                            >
                                                {rule.enabled ? "Enabled" : "Disabled"}
                                            </span>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </ScrollArea>
                    ) : (
                        <div className="flex h-32 items-center justify-center text-xs text-muted-foreground">
                            No NetSwitch rules found. Block an application to create rules.
                        </div>
                    )}
                </CardContent>
            </Card>

            {/* Admin Confirmation Dialog */}
            <Dialog open={showAdminDialog} onOpenChange={setShowAdminDialog}>
                <DialogContent className="sm:max-w-md">
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            <ShieldAlert className="h-5 w-5 text-amber-500" />
                            Administrator Access Required
                        </DialogTitle>
                        <DialogDescription className="text-left space-y-2 pt-2">
                            <p>
                                This action requires Administrator privileges to access Windows Firewall settings.
                            </p>
                            <div className="rounded-md bg-muted p-3 text-xs space-y-1">
                                <p className="font-medium">Why UAC is needed:</p>
                                <ul className="list-disc list-inside text-muted-foreground space-y-0.5">
                                    {pendingAction === "loadRules" ? (
                                        <>
                                            <li>Reading detailed firewall rule information</li>
                                            <li>Accessing protected Windows Firewall API</li>
                                        </>
                                    ) : (
                                        <>
                                            <li>Removing Windows Firewall rules</li>
                                            <li>Modifying system security settings</li>
                                        </>
                                    )}
                                </ul>
                            </div>
                            <p className="text-xs text-muted-foreground">
                                Click OK to proceed with the UAC prompt.
                            </p>
                        </DialogDescription>
                    </DialogHeader>
                    <DialogFooter className="gap-2 sm:gap-0">
                        <Button variant="outline" onClick={handleCancelAdmin}>
                            Cancel
                        </Button>
                        <Button onClick={handleConfirmAdmin}>
                            OK, Proceed
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    );
}
