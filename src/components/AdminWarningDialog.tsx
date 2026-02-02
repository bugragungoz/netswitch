import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Shield, Check } from "lucide-react";

interface AdminWarningDialogProps {
  open: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export function AdminWarningDialog({
  open,
  onConfirm,
  onCancel,
}: AdminWarningDialogProps) {
  return (
    <AlertDialog open={open}>
      <AlertDialogContent className="max-w-sm">
        <AlertDialogHeader>
          <div className="flex items-center gap-2">
            <div className="flex h-8 w-8 items-center justify-center rounded-full bg-amber-100 dark:bg-amber-900/50">
              <Shield className="h-4 w-4 text-amber-600 dark:text-amber-400" />
            </div>
            <AlertDialogTitle className="text-sm">
              Administrator Approval Required
            </AlertDialogTitle>
          </div>
          <AlertDialogDescription className="pt-2 text-xs">
            This operation requires administrator privileges.
          </AlertDialogDescription>
        </AlertDialogHeader>

        <div className="space-y-1.5 rounded-md bg-muted/50 p-3 text-xs">
          <div className="flex items-center gap-2">
            <Check className="h-3 w-3 text-muted-foreground" />
            <span>A UAC prompt will appear</span>
          </div>
          <div className="flex items-center gap-2">
            <Check className="h-3 w-3 text-muted-foreground" />
            <span>Click "Yes" to allow the operation</span>
          </div>
          <div className="flex items-center gap-2">
            <Check className="h-3 w-3 text-muted-foreground" />
            <span>Cancel will abort the operation</span>
          </div>
        </div>

        <AlertDialogFooter className="gap-2 sm:gap-0">
          <AlertDialogCancel onClick={onCancel} className="h-8 text-xs">
            Cancel
          </AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm} className="h-8 text-xs">
            Continue
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
