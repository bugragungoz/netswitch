import { getCurrentWindow } from "@tauri-apps/api/window";
import { Shield, Minus, Square, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export function TitleBar() {
  const win = getCurrentWindow();

  return (
    <header
      data-tauri-drag-region
      className={cn(
        "flex h-10 items-center justify-between border-b bg-card/80 backdrop-blur-sm select-none",
        "pl-3 pr-1"
      )}
    >
      <div data-tauri-drag-region className="flex items-center gap-2">
        <div className="flex h-6 w-6 items-center justify-center rounded bg-primary/10">
          <Shield className="h-4 w-4 text-primary" />
        </div>
        <div data-tauri-drag-region>
          <h1 className="text-xs font-semibold leading-none">
            netSwitch
          </h1>
          <p className="text-[10px] text-muted-foreground leading-none mt-0.5">
            Network Control & Firewall Manager
          </p>
        </div>
      </div>

      <div className="flex items-center gap-0">
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 rounded-none hover:bg-muted"
          onClick={() => win.minimize()}
        >
          <Minus className="h-3.5 w-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 rounded-none hover:bg-muted"
          onClick={() => win.toggleMaximize()}
        >
          <Square className="h-3.5 w-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 rounded-none hover:bg-destructive hover:text-destructive-foreground"
          onClick={() => win.close()}
        >
          <X className="h-3.5 w-3.5" />
        </Button>
      </div>
    </header>
  );
}
