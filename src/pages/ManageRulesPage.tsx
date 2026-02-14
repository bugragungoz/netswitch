import {
  Network, Globe, Shield, Terminal, Wifi, Settings2,
  ExternalLink, FileText, Cog
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

// Quick access shortcuts for network and system tools
const QUICK_ACCESS_SHORTCUTS = [
  {
    id: "wf",
    name: "Windows Firewall with Advanced Security",
    command: "wf.msc",
    desc: "Configure inbound/outbound firewall rules, connection security rules, and monitoring",
    icon: Shield
  },
  {
    id: "ncpa",
    name: "Network Connections",
    command: "ncpa.cpl",
    desc: "View and configure network adapters, disable/enable connections, view status",
    icon: Wifi
  },
  {
    id: "inetcpl",
    name: "Internet Properties",
    command: "inetcpl.cpl",
    desc: "Configure proxy settings, security zones, privacy options, and connections",
    icon: Globe
  },
  {
    id: "devmgmt",
    name: "Device Manager",
    command: "devmgmt.msc",
    desc: "Manage network adapters, update drivers, enable/disable devices",
    icon: Settings2
  },
  {
    id: "services",
    name: "Services",
    command: "services.msc",
    desc: "Manage Windows services including DHCP Client, DNS Client, etc.",
    icon: Cog
  },
  {
    id: "cmd",
    name: "Network Shell (netsh)",
    command: "cmd /k netsh",
    desc: "Command-line tool to configure network settings, firewall, and interface",
    icon: Terminal
  },
  {
    id: "eventvwr",
    name: "Event Viewer",
    command: "eventvwr.msc",
    desc: "View Windows Firewall with Advanced Security logs and events",
    icon: FileText
  },
  {
    id: "resmon",
    name: "Resource Monitor",
    command: "resmon",
    desc: "Monitor CPU, memory, disk, and network usage in real-time",
    icon: Network
  },
];

export function ManageRulesPage() {

  const handleOpenShortcut = async (command: string) => {
    try {
      // Use the backend command which handles paths and UAC better for these tools
      await invoke("run_system_tool", { command });
    } catch (error) {
      console.error("Failed to open shortcut:", error);
    }
  };

  return (
    <div className="space-y-4 pb-12">
      {/* Header */}
      <div className="flex items-center gap-2">
        <Network className="h-5 w-5 text-primary" />
        <h2 className="text-lg font-semibold">Network Tools</h2>
      </div>

      {/* Quick Access Shortcuts - Full Row Cards */}
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm flex items-center gap-2">
            <Settings2 className="h-4 w-4 text-primary" />
            Quick Access
          </CardTitle>
          <CardDescription className="text-xs">
            Windows network and system management tools
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {QUICK_ACCESS_SHORTCUTS.map((shortcut) => (
              <div
                key={shortcut.id}
                className="flex items-center justify-between rounded-lg border bg-card p-3 hover:bg-accent/50 transition-colors group"
              >
                <div className="flex items-center gap-3 min-w-0 flex-1">
                  <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-primary/10 group-hover:bg-primary/20 transition-colors flex-shrink-0">
                    <shortcut.icon className="h-4 w-4 text-primary" />
                  </div>
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium truncate">{shortcut.name}</span>
                      <code className="text-[10px] px-1.5 py-0.5 rounded bg-muted text-muted-foreground font-mono">
                        {shortcut.command}
                      </code>
                    </div>
                    <p className="text-xs text-muted-foreground mt-0.5 line-clamp-1">
                      {shortcut.desc}
                    </p>
                  </div>
                </div>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-8 w-8 flex-shrink-0 group-hover:bg-primary/10"
                      onClick={() => handleOpenShortcut(shortcut.command)}
                    >
                      <ExternalLink className="h-4 w-4" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>Open {shortcut.name}</p>
                  </TooltipContent>
                </Tooltip>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Info Card */}
      <Card className="border-blue-500/20 bg-blue-500/5">
        <CardContent className="pt-4">
          <div className="flex items-start gap-3">
            <Shield className="h-5 w-5 text-blue-500 flex-shrink-0 mt-0.5" />
            <div className="space-y-1">
              <p className="text-sm font-medium text-blue-600 dark:text-blue-400">
                Tip: Managing Firewall Rules
              </p>
              <p className="text-xs text-muted-foreground">
                Use the <strong>Firewall</strong> tab to view and manage rules created by NetSwitch.
                For advanced configuration, open <strong>wf.msc</strong> to access the full Windows
                Firewall with Advanced Security console.
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
