import { useState, useEffect, createContext, useContext } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Shield, Settings, History, AlertTriangle, Github, Flame, Activity, Sliders, Cpu, MemoryStick, ExternalLink } from "lucide-react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { TooltipProvider } from "@/components/ui/tooltip";
import { BlockApplicationPage } from "@/pages/BlockApplicationPage";
import { ManageRulesPage } from "@/pages/ManageRulesPage";
import { RestorePointPage } from "@/pages/RestorePointPage";
import { WindowsFirewallPage } from "@/pages/WindowsFirewallPage";
import { NetworkMonitorPage } from "@/pages/NetworkMonitorPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { ThemeToggle } from "@/components/ThemeToggle";
import { TitleBar } from "@/components/TitleBar";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

const GITHUB_REPO = "https://github.com/bugragungoz/netswitch";

interface AdminContextType {
  isAdmin: boolean;
  setAdmin: (value: boolean) => void;
}

export const AdminContext = createContext<AdminContextType>({
  isAdmin: false,
  setAdmin: () => { },
});

export const useAdmin = () => useContext(AdminContext);

interface SystemStats {
  cpu_usage: number;
  ram_used_gb: number;
  ram_total_gb: number;
}

function App() {
  const [activeTab, setActiveTab] = useState("block");
  const [isAdmin, setAdmin] = useState(false);
  const [systemStats, setSystemStats] = useState<SystemStats | null>(null);

  useEffect(() => {
    // Check if pending admin status from previous session or restart
    const checkAdmin = async () => {
      try {
        const adminStatus = await invoke<boolean>("check_is_admin");
        if (adminStatus) {
          setAdmin(true);
        }
      } catch (err) {
        console.error("Failed to check admin status:", err);
      }
    };

    checkAdmin();
  }, []);

  useEffect(() => {
    if (!isAdmin) return;

    const loadStats = async () => {
      try {
        const stats = await invoke<SystemStats>("get_system_stats");
        setSystemStats(stats);
      } catch (error) {
        console.error("Failed to load system stats:", error);
      }
    };

    loadStats();
    const interval = setInterval(loadStats, 3000);
    return () => clearInterval(interval);
  }, [isAdmin]);

  const openGitHub = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-shell");
      await open(GITHUB_REPO);
    } catch {
      window.open(GITHUB_REPO, "_blank", "noopener,noreferrer");
    }
  };

  return (
    <AdminContext.Provider value={{ isAdmin, setAdmin }}>
      <TooltipProvider delayDuration={300}>
        <div className="min-h-screen bg-background transition-theme flex flex-col">
          <TitleBar />

          {/* Disclaimer Banner */}
          <div className="border-b bg-amber-500/10 dark:bg-amber-500/5 shrink-0">
            <div className="flex items-center gap-2 px-4 py-1.5 text-[11px] font-medium text-amber-900 dark:text-amber-200">
              <AlertTriangle className="h-3.5 w-3.5 flex-shrink-0" />
              <span>
                Please use responsibly. The user assumes all liability for administrative actions and compliance with software licenses.
              </span>
            </div>
          </div>

          {/* Toolbar: theme + github */}
          <div className="flex h-9 items-center justify-end gap-0 border-b px-2 shrink-0">
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8"
                  onClick={openGitHub}
                >
                  <Github className="h-4 w-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>
                <p>Open GitHub repository</p>
              </TooltipContent>
            </Tooltip>
            <ThemeToggle />
          </div>

          {/* Main Content */}
          <main className="flex-1 overflow-auto p-4">
            <Tabs
              value={activeTab}
              onValueChange={setActiveTab}
              className="space-y-4"
            >
              <TabsList className="h-9 w-full max-w-2xl grid grid-cols-6">
                <TabsTrigger value="block" className="gap-1.5 text-xs">
                  <Shield className="h-3.5 w-3.5" />
                  Block
                </TabsTrigger>
                <TabsTrigger value="manage" className="gap-1.5 text-xs">
                  <Sliders className="h-3.5 w-3.5" />
                  Manage
                </TabsTrigger>
                <TabsTrigger value="firewall" className="gap-1.5 text-xs">
                  <Flame className="h-3.5 w-3.5" />
                  Firewall
                </TabsTrigger>
                <TabsTrigger value="network" className="gap-1.5 text-xs">
                  <Activity className="h-3.5 w-3.5" />
                  Network
                </TabsTrigger>
                <TabsTrigger value="restore" className="gap-1.5 text-xs">
                  <History className="h-3.5 w-3.5" />
                  Restore
                </TabsTrigger>
                <TabsTrigger value="settings" className="gap-1.5 text-xs">
                  <Settings className="h-3.5 w-3.5" />
                  Settings
                </TabsTrigger>
              </TabsList>

              <TabsContent value="block" className="mt-4">
                <BlockApplicationPage />
              </TabsContent>

              <TabsContent value="manage" className="mt-4">
                <ManageRulesPage />
              </TabsContent>

              <TabsContent value="firewall" className="mt-4">
                <WindowsFirewallPage />
              </TabsContent>

              <TabsContent value="network" className="mt-4">
                <NetworkMonitorPage />
              </TabsContent>

              <TabsContent value="restore" className="mt-4">
                <RestorePointPage />
              </TabsContent>

              <TabsContent value="settings" className="mt-4">
                <SettingsPage />
              </TabsContent>
            </Tabs>
          </main>

          {/* Footer with conditional content */}
          <footer className="shrink-0 border-t bg-card/80 backdrop-blur-sm">
            <div className="flex items-center justify-between px-4 py-2 text-[10px] text-muted-foreground select-none">
              {isAdmin ? (
                <>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <div className="flex items-center gap-1.5 cursor-default transition-colors hover:text-foreground">
                        <Cpu className="h-3 w-3" />
                        <span className="font-mono">
                          {systemStats ? `${systemStats.cpu_usage.toFixed(0)}%` : "--"}
                        </span>
                      </div>
                    </TooltipTrigger>
                    <TooltipContent side="top">
                      <p>CPU Usage</p>
                    </TooltipContent>
                  </Tooltip>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <div className="flex items-center gap-1.5 cursor-default transition-colors hover:text-foreground">
                        <MemoryStick className="h-3 w-3" />
                        <span className="font-mono">
                          {systemStats
                            ? `${systemStats.ram_used_gb.toFixed(1)}/${systemStats.ram_total_gb.toFixed(0)}GB`
                            : "--"}
                        </span>
                      </div>
                    </TooltipTrigger>
                    <TooltipContent side="top">
                      <p>RAM Usage</p>
                    </TooltipContent>
                  </Tooltip>
                </>
              ) : (
                <>
                  <div className="flex items-center gap-2">
                    <span className="text-muted-foreground/50">v0.1.0</span>
                    <span className="text-border">|</span>
                    <a
                      href="mailto:gungozb@gmail.com"
                      className="hover:text-foreground transition-colors hover:underline decoration-border/50 underline-offset-2"
                    >
                      gungozb@gmail.com
                    </a>
                  </div>
                  <button
                    onClick={openGitHub}
                    className="flex items-center gap-1 text-muted-foreground hover:text-foreground transition-colors"
                  >
                    <Github className="h-3 w-3" />
                    <span className="hidden sm:inline">github.com/bugragungoz/netswitch</span>
                    <ExternalLink className="h-2.5 w-2.5" />
                  </button>
                </>
              )}
            </div>
          </footer>
        </div>
      </TooltipProvider>
    </AdminContext.Provider>
  );
}

export default App;
