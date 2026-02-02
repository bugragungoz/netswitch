import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Shield, FolderOpen, Play, X, Settings2, ChevronDown, ChevronRight, FolderTree, FileType2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { Progress } from "@/components/ui/progress";
import { Switch } from "@/components/ui/switch";
import { ToolCard } from "@/components/ToolCard";
import { LiveTerminal } from "@/components/LiveTerminal";
import { AdminWarningDialog } from "@/components/AdminWarningDialog";
import { useAdminTool, ExecutionResult } from "@/hooks/useAdminTool";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";

interface BlockRequest {
  app_name: string;
  app_path: string;
  file_extensions: string[];
  excluded_keywords: string[];
  excluded_files: string[];
  include_subdirectories: boolean;
  block_both_directions: boolean;
}

export function BlockApplicationPage() {
  const [appName, setAppName] = useState("");
  const [appPath, setAppPath] = useState("");
  const [blockExe, setBlockExe] = useState(true);
  const [blockDll, setBlockDll] = useState(true);
  const [excludedKeywords, setExcludedKeywords] = useState("");
  const [excludedFiles, setExcludedFiles] = useState("");
  const [includeSubdirectories, setIncludeSubdirectories] = useState(true);
  const [blockBothDirections, setBlockBothDirections] = useState(true);
  const [advancedOpen, setAdvancedOpen] = useState(false);

  const adminTool = useAdminTool({
    requiresAdmin: true,
    onComplete: (result) => {
      console.log("Blocking completed:", result);
    },
    onError: (error) => {
      console.error("Blocking failed:", error);
    },
  });

  const handleSelectDirectory = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Application Directory",
      });

      if (selected && typeof selected === "string") {
        setAppPath(selected);

        if (!appName) {
          const parts = selected.split(/[\\/]/);
          const folderName = parts[parts.length - 1] || parts[parts.length - 2];
          setAppName(folderName);
        }
      }
    } catch (error) {
      console.error("Failed to open directory picker:", error);
    }
  };

  const handleBlock = () => {
    if (!appName.trim() || !appPath.trim()) {
      return;
    }

    const extensions: string[] = [];
    if (blockExe) extensions.push("*.exe");
    if (blockDll) extensions.push("*.dll");

    if (extensions.length === 0) {
      return;
    }

    const request: BlockRequest = {
      app_name: appName.trim(),
      app_path: appPath.trim(),
      file_extensions: extensions,
      excluded_keywords: excludedKeywords
        .split(",")
        .map((k) => k.trim())
        .filter((k) => k),
      excluded_files: excludedFiles
        .split(",")
        .map((f) => f.trim())
        .filter((f) => f),
      include_subdirectories: includeSubdirectories,
      block_both_directions: blockBothDirections,
    };

    adminTool.execute(async (): Promise<ExecutionResult> => {
      const result = await invoke<ExecutionResult>("block_application", {
        request,
      });
      return result;
    });
  };

  const isFormValid =
    appName.trim() && appPath.trim() && (blockExe || blockDll);
  const isRunning = adminTool.status === "running";

  return (
    <div className="space-y-4 pb-12">
      <ToolCard
        title="Block Application"
        description="Block internet access for executables in a folder"
        icon={<Shield className="h-4 w-4 text-primary" />}
        requiresAdmin
      >
        <div className="space-y-3">
          {/* Application Name */}
          <div className="space-y-1">
            <Label htmlFor="appName" className="text-xs">
              Application Name
            </Label>
            <Input
              id="appName"
              placeholder="e.g., My Drawing App"
              value={appName}
              onChange={(e) => setAppName(e.target.value)}
              disabled={isRunning}
              className="h-8 text-sm"
            />
          </div>

          {/* Application Path */}
          <div className="space-y-1">
            <Label htmlFor="appPath" className="text-xs">
              Application Directory
            </Label>
            <div className="flex gap-1.5">
              <Input
                id="appPath"
                placeholder="C:\Program Files\MyApp"
                value={appPath}
                onChange={(e) => setAppPath(e.target.value)}
                disabled={isRunning}
                className="flex-1 h-8 text-sm"
              />
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="outline"
                    size="icon"
                    onClick={handleSelectDirectory}
                    disabled={isRunning}
                    className="h-8 w-8"
                  >
                    <FolderOpen className="h-3.5 w-3.5" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  <p>Browse folder</p>
                </TooltipContent>
              </Tooltip>
            </div>
          </div>

          {/* File Extensions */}
          <div className="space-y-1.5">
            <Label className="text-xs">File Extensions</Label>
            <div className="flex gap-4">
              <div className="flex items-center space-x-1.5">
                <Checkbox
                  id="blockExe"
                  checked={blockExe}
                  onCheckedChange={(checked) => setBlockExe(checked === true)}
                  disabled={isRunning}
                  className="h-3.5 w-3.5"
                />
                <Label htmlFor="blockExe" className="text-xs font-normal">
                  *.exe
                </Label>
              </div>
              <div className="flex items-center space-x-1.5">
                <Checkbox
                  id="blockDll"
                  checked={blockDll}
                  onCheckedChange={(checked) => setBlockDll(checked === true)}
                  disabled={isRunning}
                  className="h-3.5 w-3.5"
                />
                <Label htmlFor="blockDll" className="text-xs font-normal">
                  *.dll
                </Label>
              </div>
            </div>
          </div>

          {/* Advanced Settings - Collapsible */}
          <Collapsible open={advancedOpen} onOpenChange={setAdvancedOpen}>
            <CollapsibleTrigger asChild>
              <Button
                variant="ghost"
                size="sm"
                className="w-full justify-between h-8 px-2 text-xs text-muted-foreground hover:text-foreground"
              >
                <span className="flex items-center gap-1.5">
                  <Settings2 className="h-3.5 w-3.5" />
                  Advanced Settings
                </span>
                {advancedOpen ? (
                  <ChevronDown className="h-3.5 w-3.5" />
                ) : (
                  <ChevronRight className="h-3.5 w-3.5" />
                )}
              </Button>
            </CollapsibleTrigger>
            <CollapsibleContent className="space-y-3 pt-2">
              <div className="rounded-lg border bg-muted/30 p-3 space-y-3">
                {/* Subdirectories Toggle */}
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <FolderTree className="h-4 w-4 text-muted-foreground" />
                    <div>
                      <Label htmlFor="includeSubdirs" className="text-xs font-medium">
                        Include Subdirectories
                      </Label>
                      <p className="text-[10px] text-muted-foreground">
                        Scan and block files in all subfolders
                      </p>
                    </div>
                  </div>
                  <Switch
                    id="includeSubdirs"
                    checked={includeSubdirectories}
                    onCheckedChange={setIncludeSubdirectories}
                    disabled={isRunning}
                  />
                </div>

                {/* Both Directions Toggle */}
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <FileType2 className="h-4 w-4 text-muted-foreground" />
                    <div>
                      <Label htmlFor="bothDirections" className="text-xs font-medium">
                        Block Both Directions
                      </Label>
                      <p className="text-[10px] text-muted-foreground">
                        Create both inbound and outbound rules
                      </p>
                    </div>
                  </div>
                  <Switch
                    id="bothDirections"
                    checked={blockBothDirections}
                    onCheckedChange={setBlockBothDirections}
                    disabled={isRunning}
                  />
                </div>

                {/* Exclusions */}
                <div className="pt-2 border-t space-y-2">
                  <p className="text-xs font-medium flex items-center gap-1.5">
                    Exclusions
                  </p>
                  <div className="space-y-1">
                    <Label htmlFor="excludedKeywords" className="text-[11px] text-muted-foreground">
                      Excluded Keywords (comma-separated)
                    </Label>
                    <Input
                      id="excludedKeywords"
                      placeholder="uninstall, updater, helper"
                      value={excludedKeywords}
                      onChange={(e) => setExcludedKeywords(e.target.value)}
                      disabled={isRunning}
                      className="h-7 text-xs"
                    />
                  </div>
                  <div className="space-y-1">
                    <Label htmlFor="excludedFiles" className="text-[11px] text-muted-foreground">
                      Excluded Files (comma-separated)
                    </Label>
                    <Input
                      id="excludedFiles"
                      placeholder="setup.exe, config.dll"
                      value={excludedFiles}
                      onChange={(e) => setExcludedFiles(e.target.value)}
                      disabled={isRunning}
                      className="h-7 text-xs"
                    />
                  </div>
                </div>
              </div>
            </CollapsibleContent>
          </Collapsible>

          {/* Progress */}
          {isRunning && (
            <div className="space-y-1">
              <div className="flex items-center justify-between text-xs">
                <span className="text-muted-foreground">Processing...</span>
                <span>{adminTool.progress}%</span>
              </div>
              <Progress value={adminTool.progress} className="h-1.5" />
            </div>
          )}

          {/* Terminal Output */}
          {adminTool.logs.length > 0 && (
            <LiveTerminal logs={adminTool.logs} maxHeight="h-32" />
          )}

          {/* Error Message */}
          {adminTool.error && (
            <div className="rounded-md border border-destructive/50 bg-destructive/10 p-2 text-xs text-destructive">
              {adminTool.error}
            </div>
          )}

          {/* Success Message */}
          {adminTool.status === "completed" && (
            <div className="rounded-md border border-green-500/50 bg-green-500/10 p-2 text-xs text-green-600 dark:text-green-400">
              Application blocked successfully.
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-2 pt-1">
            <Button
              onClick={handleBlock}
              disabled={!isFormValid || isRunning}
              className="flex-1 h-8 text-xs"
            >
              {isRunning ? (
                "Processing..."
              ) : (
                <>
                  <Play className="mr-1.5 h-3 w-3" />
                  Block Application
                </>
              )}
            </Button>
            {(adminTool.status === "completed" ||
              adminTool.status === "error") && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="outline"
                      size="icon"
                      onClick={adminTool.reset}
                      className="h-8 w-8"
                    >
                      <X className="h-3.5 w-3.5" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>Reset form</p>
                  </TooltipContent>
                </Tooltip>
              )}
          </div>
        </div>
      </ToolCard>

      <AdminWarningDialog
        open={adminTool.showWarningDialog}
        onConfirm={adminTool.confirmExecution}
        onCancel={adminTool.cancelExecution}
      />
    </div>
  );
}
