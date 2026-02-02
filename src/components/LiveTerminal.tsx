import { useEffect, useRef } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import { Terminal } from "lucide-react";

interface LiveTerminalProps {
  logs: string[];
  className?: string;
  autoScroll?: boolean;
  maxHeight?: string;
}

export function LiveTerminal({
  logs,
  className,
  autoScroll = true,
  maxHeight = "h-40",
}: LiveTerminalProps) {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (autoScroll && bottomRef.current) {
      bottomRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [logs, autoScroll]);

  const getLogClass = (log: string): string => {
    const upperLog = log.toUpperCase();
    if (upperLog.includes("[ERROR]")) return "log-error";
    if (upperLog.includes("[WARN]")) return "log-warn";
    if (upperLog.includes("[DEBUG]")) return "log-debug";
    if (upperLog.includes("SUCCESS") || upperLog.includes("COMPLETED"))
      return "log-success";
    return "log-info";
  };

  return (
    <div className={cn("rounded-md border bg-zinc-950 dark:bg-zinc-900", className)}>
      <div className="flex items-center gap-1.5 border-b border-zinc-800 px-2 py-1">
        <Terminal className="h-3 w-3 text-zinc-500" />
        <span className="text-[10px] text-zinc-500">Output</span>
      </div>
      <ScrollArea className={cn(maxHeight, "w-full p-2")}>
        <div className="terminal-output space-y-0.5">
          {logs.length === 0 ? (
            <div className="text-zinc-600 text-[11px]">Waiting for output...</div>
          ) : (
            logs.map((log, index) => (
              <div key={index} className={getLogClass(log)}>
                {log}
              </div>
            ))
          )}
          <div ref={bottomRef} />
        </div>
      </ScrollArea>
    </div>
  );
}
