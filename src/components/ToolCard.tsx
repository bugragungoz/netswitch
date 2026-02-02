import { ReactNode } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { AdminBadge } from "./AdminBadge";
import { cn } from "@/lib/utils";

interface ToolCardProps {
  title: string;
  description: string;
  icon: ReactNode;
  requiresAdmin?: boolean;
  children: ReactNode;
  className?: string;
}

export function ToolCard({
  title,
  description,
  icon,
  requiresAdmin = false,
  children,
  className,
}: ToolCardProps) {
  return (
    <Card className={cn("w-full", className)}>
      <CardHeader className="pb-3 pt-4 px-4">
        <div className="flex items-start justify-between gap-2">
          <div className="flex items-center gap-2">
            <div className="flex h-8 w-8 items-center justify-center rounded-md bg-primary/10">
              {icon}
            </div>
            <div>
              <CardTitle className="text-sm">{title}</CardTitle>
              <CardDescription className="text-[11px]">
                {description}
              </CardDescription>
            </div>
          </div>
          {requiresAdmin && <AdminBadge />}
        </div>
      </CardHeader>
      <CardContent className="px-4 pb-4">{children}</CardContent>
    </Card>
  );
}
