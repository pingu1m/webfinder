import React, { memo, useCallback, useState } from "react";
import {
  ChevronRight,
  File,
  FileCode,
  FileJson,
  FileText,
  Folder,
  FolderOpen,
  Image,
  Settings,
  Shield,
} from "lucide-react";
import { cn } from "@/lib/utils";
import type { FileNode } from "@/api/types";

function getFileIcon(name: string) {
  const ext = name.split(".").pop()?.toLowerCase();
  switch (ext) {
    case "tsx":
    case "ts":
    case "jsx":
    case "js":
    case "mjs":
    case "rs":
    case "py":
    case "go":
    case "rb":
      return FileCode;
    case "json":
    case "toml":
    case "yaml":
    case "yml":
      return FileJson;
    case "md":
    case "txt":
    case "css":
    case "scss":
    case "html":
      return FileText;
    case "svg":
    case "png":
    case "jpg":
    case "ico":
    case "gif":
    case "webp":
      return Image;
    case "config":
      return Settings;
    default:
      if (name.startsWith(".env")) return Shield;
      if (name.startsWith(".git")) return Shield;
      return File;
  }
}

interface FileTreeItemProps {
  node: FileNode;
  selectedFile: string | null;
  onSelect: (path: string) => void;
  onContextMenu: (e: React.MouseEvent, node: FileNode) => void;
  onPrefetch: (path: string) => void;
  depth?: number;
}

export const FileTreeItem = memo(function FileTreeItem({
  node,
  selectedFile,
  onSelect,
  onContextMenu,
  onPrefetch,
  depth = 0,
}: FileTreeItemProps) {
  const [isOpen, setIsOpen] = useState(false);

  const handleContextMenu = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      onContextMenu(e, node);
    },
    [onContextMenu, node]
  );

  if (node.type === "dir") {
    return (
      <div>
        <button
          className={cn(
            "flex w-full items-center gap-1 px-2 py-1 text-sm hover:bg-accent/50 rounded-sm text-left",
            "transition-colors duration-75"
          )}
          style={{ paddingLeft: `${depth * 12 + 8}px` }}
          onClick={() => setIsOpen(!isOpen)}
          onContextMenu={handleContextMenu}
        >
          <ChevronRight
            className={cn(
              "h-3.5 w-3.5 shrink-0 transition-transform duration-150",
              isOpen && "rotate-90"
            )}
          />
          {isOpen ? (
            <FolderOpen className="h-4 w-4 shrink-0 text-blue-500" />
          ) : (
            <Folder className="h-4 w-4 shrink-0 text-blue-500" />
          )}
          <span className="truncate">{node.name}</span>
        </button>
        {isOpen && node.children && (
          <div>
            {node.children
              .slice()
              .sort((a, b) => {
                if (a.type !== b.type) return a.type === "dir" ? -1 : 1;
                return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
              })
              .map((child) => (
                <FileTreeItem
                  key={child.path}
                  node={child}
                  selectedFile={selectedFile}
                  onSelect={onSelect}
                  onContextMenu={onContextMenu}
                  onPrefetch={onPrefetch}
                  depth={depth + 1}
                />
              ))}
          </div>
        )}
      </div>
    );
  }

  const Icon = getFileIcon(node.name);
  const isSelected = selectedFile === node.path;

  return (
    <button
      className={cn(
        "flex w-full items-center gap-1.5 px-2 py-1 text-sm rounded-sm text-left",
        "transition-colors duration-75",
        isSelected
          ? "bg-accent text-accent-foreground"
          : "hover:bg-accent/50 text-foreground"
      )}
      style={{ paddingLeft: `${depth * 12 + 22}px` }}
      onClick={() => onSelect(node.path)}
      onContextMenu={handleContextMenu}
      onMouseEnter={() => {
        if (node.type === "file") {
          const timer = setTimeout(() => onPrefetch(node.path), 300);
          (globalThis as any).__prefetchTimer = timer;
        }
      }}
      onMouseLeave={() => {
        clearTimeout((globalThis as any).__prefetchTimer);
      }}
    >
      <Icon className="h-4 w-4 shrink-0 text-muted-foreground" />
      <span className="truncate">{node.name}</span>
    </button>
  );
});
