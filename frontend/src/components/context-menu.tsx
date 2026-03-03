import { useCallback, useEffect, useState } from "react";
import {
  Copy,
  Edit3,
  FilePlus,
  FolderPlus,
  Trash2,
} from "lucide-react";
import { cn } from "@/lib/utils";
import type { FileNode } from "@/api/types";

interface ContextMenuState {
  x: number;
  y: number;
  node: FileNode | null;
}

interface ContextMenuProps {
  state: ContextMenuState | null;
  onClose: () => void;
  onNewFile: (parentPath: string) => void;
  onNewFolder: (parentPath: string) => void;
  onRename: (node: FileNode) => void;
  onDelete: (node: FileNode) => void;
  onCopy: (node: FileNode) => void;
}

export function ContextMenu({
  state,
  onClose,
  onNewFile,
  onNewFolder,
  onRename,
  onDelete,
  onCopy,
}: ContextMenuProps) {
  useEffect(() => {
    if (state) {
      const handler = () => onClose();
      document.addEventListener("click", handler);
      return () => document.removeEventListener("click", handler);
    }
  }, [state, onClose]);

  if (!state || !state.node) return null;

  const isDir = state.node.type === "dir";
  const parentPath = isDir
    ? state.node.path
    : state.node.path.split("/").slice(0, -1).join("/");

  return (
    <div
      className="fixed z-50 min-w-48 rounded-md border bg-popover p-1 text-popover-foreground shadow-md"
      style={{ left: state.x, top: state.y }}
    >
      {isDir && (
        <>
          <MenuItem
            icon={<FilePlus className="h-3.5 w-3.5" />}
            label="New File"
            onClick={() => {
              onNewFile(parentPath);
              onClose();
            }}
          />
          <MenuItem
            icon={<FolderPlus className="h-3.5 w-3.5" />}
            label="New Folder"
            onClick={() => {
              onNewFolder(parentPath);
              onClose();
            }}
          />
          <div className="my-1 h-px bg-border" />
        </>
      )}
      <MenuItem
        icon={<Edit3 className="h-3.5 w-3.5" />}
        label="Rename"
        onClick={() => {
          onRename(state.node!);
          onClose();
        }}
      />
      {state.node.type === "file" && (
        <MenuItem
          icon={<Copy className="h-3.5 w-3.5" />}
          label="Copy"
          onClick={() => {
            onCopy(state.node!);
            onClose();
          }}
        />
      )}
      <div className="my-1 h-px bg-border" />
      <MenuItem
        icon={<Trash2 className="h-3.5 w-3.5" />}
        label="Delete"
        destructive
        onClick={() => {
          onDelete(state.node!);
          onClose();
        }}
      />
    </div>
  );
}

function MenuItem({
  icon,
  label,
  onClick,
  destructive,
}: {
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
  destructive?: boolean;
}) {
  return (
    <button
      className={cn(
        "flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-xs",
        destructive
          ? "text-destructive hover:bg-destructive/10"
          : "hover:bg-accent hover:text-accent-foreground"
      )}
      onClick={onClick}
    >
      {icon}
      {label}
    </button>
  );
}

// Simple prompt dialog for new file/folder/rename
interface NewItemDialogProps {
  open: boolean;
  title: string;
  defaultValue?: string;
  onConfirm: (value: string) => void;
  onCancel: () => void;
}

export function NewItemDialog({
  open,
  title,
  defaultValue = "",
  onConfirm,
  onCancel,
}: NewItemDialogProps) {
  const [value, setValue] = useState(defaultValue);

  useEffect(() => {
    setValue(defaultValue);
  }, [defaultValue, open]);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="bg-popover border rounded-lg shadow-lg p-4 w-80">
        <h3 className="text-sm font-medium mb-3">{title}</h3>
        <input
          autoFocus
          className="w-full rounded border bg-background px-3 py-1.5 text-sm focus:outline-none focus:ring-1 focus:ring-ring"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && value.trim()) onConfirm(value.trim());
            if (e.key === "Escape") onCancel();
          }}
        />
        <div className="flex justify-end gap-2 mt-3">
          <button
            className="px-3 py-1 text-xs rounded border hover:bg-muted"
            onClick={onCancel}
          >
            Cancel
          </button>
          <button
            className="px-3 py-1 text-xs rounded bg-primary text-primary-foreground hover:bg-primary/90"
            onClick={() => value.trim() && onConfirm(value.trim())}
          >
            Confirm
          </button>
        </div>
      </div>
    </div>
  );
}
