import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Shield } from "lucide-react";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

interface AdminBadgeProps {
  className?: string;
}

const ADMIN_TOOLTIP = (
  <div className="space-y-2 text-left max-w-[260px]">
    <p className="font-semibold text-xs text-foreground">Administrator Access Required</p>
    <p className="text-[11px] text-muted-foreground">
      Certain advanced features need elevated privileges to function:
    </p>
    <ul className="list-disc list-inside text-[11px] text-muted-foreground space-y-0.5 ml-1">
      <li>Managing Windows Firewall rules</li>
      <li>Creating system restore points</li>
      <li>Detailed network process analysis</li>
    </ul>
    <div className="pt-1.5 border-t mt-1.5">
      <p className="text-[10px] text-muted-foreground italic">
        Tip: Rerunning the app as Administrator removes UAC prompts.
      </p>
    </div>
  </div>
);

export function AdminBadge({ className }: AdminBadgeProps) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <Popover open={isOpen} onOpenChange={setIsOpen}>
      <PopoverTrigger asChild>
        <span
          className="inline-flex"
          onMouseEnter={() => setIsOpen(true)}
          onMouseLeave={() => setIsOpen(false)}
        >
          <Badge
            variant="outline"
            className={`
              cursor-help select-none text-[10px] px-2 py-0 h-5 transition-all duration-200
              ${isOpen
                ? "bg-amber-500/15 text-amber-600 border-amber-500/30"
                : "bg-transparent text-muted-foreground/40 border-transparent grayscale opacity-50 hover:grayscale-0 hover:opacity-100 hover:bg-amber-500/10 hover:text-amber-600 hover:border-amber-500/20"
              }
              ${className}
            `}
          >
            <Shield className="mr-1 h-3 w-3" />
            Admin
          </Badge>
        </span>
      </PopoverTrigger>
      <PopoverContent
        side="left"
        className="p-3 max-w-[280px]"
        onMouseEnter={() => setIsOpen(true)}
        onMouseLeave={() => setIsOpen(false)}
      >
        {ADMIN_TOOLTIP}
      </PopoverContent>
    </Popover>
  );
}
