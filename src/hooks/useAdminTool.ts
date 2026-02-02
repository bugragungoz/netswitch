import { useState, useCallback } from "react";

export type ExecutionStatus = "idle" | "pending" | "running" | "completed" | "error" | "cancelled";

export interface ExecutionResult {
  success: boolean;
  logs: string[];
  error?: string;
}

export interface UseAdminToolOptions {
  requiresAdmin: boolean;
  onComplete?: (result: ExecutionResult) => void;
  onError?: (error: string) => void;
}

export interface UseAdminToolReturn {
  status: ExecutionStatus;
  logs: string[];
  error: string | null;
  progress: number;
  showWarningDialog: boolean;
  execute: (action: () => Promise<ExecutionResult>) => void;
  confirmExecution: () => void;
  cancelExecution: () => void;
  reset: () => void;
}

export function useAdminTool(options: UseAdminToolOptions): UseAdminToolReturn {
  const { requiresAdmin, onComplete, onError } = options;

  const [status, setStatus] = useState<ExecutionStatus>("idle");
  const [logs, setLogs] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState(0);
  const [showWarningDialog, setShowWarningDialog] = useState(false);
  const [pendingAction, setPendingAction] = useState<(() => Promise<ExecutionResult>) | null>(null);

  const reset = useCallback(() => {
    setStatus("idle");
    setLogs([]);
    setError(null);
    setProgress(0);
    setShowWarningDialog(false);
    setPendingAction(null);
  }, []);

  const execute = useCallback((action: () => Promise<ExecutionResult>) => {
    if (requiresAdmin) {
      setPendingAction(() => action);
      setShowWarningDialog(true);
      setStatus("pending");
    } else {
      runAction(action);
    }
  }, [requiresAdmin]);

  const runAction = async (action: () => Promise<ExecutionResult>) => {
    setStatus("running");
    setLogs([]);
    setError(null);
    setProgress(0);

    try {
      const result = await action();
      
      setLogs(result.logs);
      setProgress(100);

      if (result.success) {
        setStatus("completed");
        onComplete?.(result);
      } else {
        setStatus("error");
        setError(result.error || "Operation failed");
        onError?.(result.error || "Operation failed");
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setStatus("error");
      setError(errorMessage);
      onError?.(errorMessage);
    }
  };

  const confirmExecution = useCallback(() => {
    setShowWarningDialog(false);
    if (pendingAction) {
      runAction(pendingAction);
      setPendingAction(null);
    }
  }, [pendingAction]);

  const cancelExecution = useCallback(() => {
    setShowWarningDialog(false);
    setPendingAction(null);
    setStatus("cancelled");
    setError("Operation cancelled by user");
  }, []);

  return {
    status,
    logs,
    error,
    progress,
    showWarningDialog,
    execute,
    confirmExecution,
    cancelExecution,
    reset,
  };
}
