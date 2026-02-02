import { Badge } from "@/components/ui/badge";
import { Shield } from "lucide-react";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface AdminBadgeProps {
  className?: string;
}

const ADMIN_TOOLTIP = (
  <div className="space-y-2 text-left max-w-[260px]">
    <p className="font-medium text-xs">Administrator privileges are required for:</p>
    <ul className="list-disc list-inside text-[11px] text-muted-foreground space-y-0.5">
      <li>Creating or removing Windows Firewall rules</li>
      <li>Listing firewall rules created by this app</li>
      <li>Creating system restore points</li>
    </ul>
    <p className="text-[11px] text-muted-foreground pt-0.5">
      If you run this app as Administrator (right-click exe -&gt; Run as administrator), you will not be prompted for UAC on each operation.
    </p>
  </div>
);

export function AdminBadge({ className }: AdminBadgeProps) {
  return (
    <Tooltip delayDuration={0}>
      <TooltipTrigger asChild>
        <span className="inline-flex">
          <Badge
            variant="warning"
            className={`cursor-help select-none text-[10px] px-1.5 py-0 h-5 hover:bg-amber-500/40 transition-colors ${className}`}
          >
            <Shield className="mr-1 h-2.5 w-2.5" />
            Admin
          </Badge>
        </span>
      </TooltipTrigger>
      <TooltipContent side="left" className="p-3 max-w-[280px]">
        {ADMIN_TOOLTIP}
      </TooltipContent>
    </Tooltip>
  );
}
