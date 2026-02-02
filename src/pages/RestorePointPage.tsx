import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { History, Play, AlertCircle, Info, Shield, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ToolCard } from "@/components/ToolCard";
import { LiveTerminal } from "@/components/LiveTerminal";
import { AdminWarningDialog } from "@/components/AdminWarningDialog";
import { useAdminTool, ExecutionResult } from "@/hooks/useAdminTool";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";

export function RestorePointPage() {
  const [description, setDescription] = useState(
    `Before AppBlocker - ${new Date().toLocaleDateString()}`
  );

  const adminTool = useAdminTool({
    requiresAdmin: true,
  });

  const handleCreate = () => {
    if (!description.trim()) return;

    adminTool.execute(async (): Promise<ExecutionResult> => {
      const result = await invoke<ExecutionResult>("create_restore_point", {
        description: description.trim(),
      });
      return result;
    });
  };

  const isRunning = adminTool.status === "running";

  return (
    <div className="space-y-4 pb-12">
      {/* Information Card */}
      <Card className="border-blue-500/20 bg-blue-500/5">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm flex items-center gap-2">
            <Info className="h-4 w-4 text-blue-500" />
            What is System Restore?
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-2 text-xs text-muted-foreground">
          <p>
            System Restore is a Windows feature that allows you to roll back your computer's
            system files and settings to an earlier point in time without affecting personal files.
          </p>
          <Accordion type="single" collapsible className="w-full">
            <AccordionItem value="why" className="border-none">
              <AccordionTrigger className="text-xs py-2 hover:no-underline">
                <span className="flex items-center gap-2">
                  <Shield className="h-3 w-3" />
                  Why create a restore point?
                </span>
              </AccordionTrigger>
              <AccordionContent className="text-xs text-muted-foreground pb-2">
                <ul className="list-disc list-inside space-y-1">
                  <li>Before making significant changes to your system</li>
                  <li>Before modifying Windows Firewall rules</li>
                  <li>As a safety net in case something goes wrong</li>
                  <li>Before installing new software or drivers</li>
                </ul>
              </AccordionContent>
            </AccordionItem>
            <AccordionItem value="how" className="border-none">
              <AccordionTrigger className="text-xs py-2 hover:no-underline">
                <span className="flex items-center gap-2">
                  <RotateCcw className="h-3 w-3" />
                  How to restore?
                </span>
              </AccordionTrigger>
              <AccordionContent className="text-xs text-muted-foreground pb-2">
                <ol className="list-decimal list-inside space-y-1">
                  <li>Search "Create a restore point" in Start Menu</li>
                  <li>Click "System Restore" button</li>
                  <li>Select a restore point from the list</li>
                  <li>Follow the wizard to restore your system</li>
                </ol>
              </AccordionContent>
            </AccordionItem>
          </Accordion>
        </CardContent>
      </Card>

      <ToolCard
        title="System Restore Point"
        description="Create a restore point before making changes"
        icon={<History className="h-4 w-4 text-primary" />}
        requiresAdmin
      >
        <div className="space-y-3">
          <div className="space-y-1">
            <Label htmlFor="description" className="text-xs">
              Description
            </Label>
            <Input
              id="description"
              placeholder="Restore point description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              disabled={isRunning}
              className="h-8 text-sm"
            />
          </div>

          {/* Terminal Output */}
          {adminTool.logs.length > 0 && (
            <LiveTerminal logs={adminTool.logs} maxHeight="h-24" />
          )}

          {/* Error/Success */}
          {adminTool.error && (
            <div className="rounded-md border border-destructive/50 bg-destructive/10 p-2 text-xs text-destructive">
              {adminTool.error}
            </div>
          )}

          {adminTool.status === "completed" && !adminTool.error && (
            <div className="rounded-md border border-green-500/50 bg-green-500/10 p-2 text-xs text-green-600 dark:text-green-400">
              Restore point created successfully.
            </div>
          )}

          <Button
            onClick={handleCreate}
            disabled={!description.trim() || isRunning}
            className="w-full h-8 text-xs"
          >
            {isRunning ? (
              "Creating..."
            ) : (
              <>
                <Play className="mr-1.5 h-3 w-3" />
                Create Restore Point
              </>
            )}
          </Button>

          <div className="flex items-start gap-2 rounded-md bg-muted/50 p-2 text-[11px] text-muted-foreground">
            <AlertCircle className="h-3 w-3 mt-0.5 flex-shrink-0" />
            <span>
              System Restore must be enabled on your system drive. This may take
              a few moments.
            </span>
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
