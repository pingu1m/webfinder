import { AlertTriangle, HardDriveDownload, PenLine } from "lucide-react";

interface FileConflictDialogProps {
  path: string;
  onKeep: () => void;
  onLoadFromDisk: () => void;
}

export function FileConflictDialog({
  path,
  onKeep,
  onLoadFromDisk,
}: FileConflictDialogProps) {
  const filename = path.split("/").pop() ?? path;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="bg-popover border rounded-lg shadow-lg p-5 w-96">
        <div className="flex items-start gap-3 mb-4">
          <div className="rounded-full bg-yellow-500/10 p-2 shrink-0">
            <AlertTriangle className="h-5 w-5 text-yellow-600" />
          </div>
          <div>
            <h3 className="text-sm font-semibold mb-1">File changed on disk</h3>
            <p className="text-xs text-muted-foreground">
              <span className="font-mono font-medium text-foreground">{filename}</span>{" "}
              has been modified externally. You have unsaved changes that may conflict.
            </p>
          </div>
        </div>

        <div className="flex justify-end gap-2">
          <button
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-md border hover:bg-muted transition-colors"
            onClick={onKeep}
          >
            <PenLine className="h-3 w-3" />
            Keep my changes
          </button>
          <button
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
            onClick={onLoadFromDisk}
          >
            <HardDriveDownload className="h-3 w-3" />
            Load from disk
          </button>
        </div>
      </div>
    </div>
  );
}
