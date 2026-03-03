import { useCallback, useMemo, useState } from "react";
import {
  FileCode,
  FilePlus,
  FolderPlus,
  FolderTree,
  Search,
  X,
} from "lucide-react";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { FileTreeItem } from "@/components/file-tree";
import type { FileNode } from "@/api/types";
import { cn } from "@/lib/utils";

function filterTree(nodes: FileNode[], query: string): FileNode[] {
  if (!query) return nodes;
  const lower = query.toLowerCase();
  return nodes.reduce<FileNode[]>((acc, node) => {
    if (node.type === "file") {
      if (node.name.toLowerCase().includes(lower)) acc.push(node);
    } else {
      const filteredChildren = filterTree(node.children ?? [], query);
      if (filteredChildren.length > 0 || node.name.toLowerCase().includes(lower)) {
        acc.push({ ...node, children: filteredChildren });
      }
    }
    return acc;
  }, []);
}

interface SidebarProps {
  tree: FileNode[];
  selectedFile: string | null;
  openFiles: { path: string; dirty: boolean }[];
  onSelectFile: (path: string) => void;
  onCloseFile: (path: string) => void;
  onCloseAllFiles: () => void;
  onContextMenu: (e: React.MouseEvent, node: FileNode) => void;
  onPrefetch: (path: string) => void;
  onNewFile: () => void;
  onNewFolder: () => void;
  collapsed?: boolean;
  onToggle?: () => void;
}

export function SidebarCustom({
  tree,
  selectedFile,
  openFiles,
  onSelectFile,
  onCloseFile,
  onCloseAllFiles,
  onContextMenu,
  onPrefetch,
  onNewFile,
  onNewFolder,
  collapsed,
  onToggle,
}: SidebarProps) {
  const [search, setSearch] = useState("");
  const filtered = useMemo(() => filterTree(tree, search), [tree, search]);

  const handleSearchChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => setSearch(e.target.value),
    []
  );

  if (collapsed) {
    return (
      <div className="w-10 border-r bg-sidebar-background flex flex-col items-center pt-2">
        <button
          className="p-2 rounded hover:bg-accent"
          onClick={onToggle}
          title="Expand sidebar"
        >
          <FolderTree className="h-4 w-4 text-muted-foreground" />
        </button>
      </div>
    );
  }

  return (
    <div className="w-64 border-r bg-sidebar-background flex flex-col h-full shrink-0">
      {/* Header */}
      <div className="border-b border-sidebar-border px-3 py-2">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-2">
            <FolderTree className="h-4 w-4 text-blue-500" />
            <span className="text-sm font-semibold">Explorer</span>
          </div>
          <div className="flex items-center gap-0.5">
            <Button variant="ghost" size="icon-xs" onClick={onNewFile} title="New File">
              <FilePlus className="h-3.5 w-3.5" />
            </Button>
            <Button variant="ghost" size="icon-xs" onClick={onNewFolder} title="New Folder">
              <FolderPlus className="h-3.5 w-3.5" />
            </Button>
            {onToggle && (
              <Button variant="ghost" size="icon-xs" onClick={onToggle} title="Collapse sidebar">
                <X className="h-3.5 w-3.5" />
              </Button>
            )}
          </div>
        </div>
        <div className="relative">
          <Search className="absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="Search files..."
            value={search}
            onChange={handleSearchChange}
            className="h-7 pl-7 text-xs"
          />
        </div>
      </div>

      <ScrollArea className="flex-1">
        {/* Open Files */}
        {openFiles.length > 0 && (
          <div className="px-1 py-1">
            <div className="flex items-center justify-between px-2 py-1">
              <span className="text-[10px] uppercase tracking-wider text-muted-foreground font-medium">
                Open Files
              </span>
              <button
                onClick={onCloseAllFiles}
                className="p-0.5 rounded hover:bg-muted"
                title="Close all"
              >
                <X className="h-3 w-3 text-muted-foreground" />
              </button>
            </div>
            {openFiles.map((file) => (
              <div
                key={file.path}
                className={cn(
                  "group flex items-center gap-1.5 px-2 py-1 text-xs rounded-sm cursor-pointer",
                  selectedFile === file.path
                    ? "bg-accent text-accent-foreground"
                    : "hover:bg-accent/50"
                )}
                onClick={() => onSelectFile(file.path)}
              >
                <FileCode className="h-3.5 w-3.5 shrink-0" />
                <span className="truncate flex-1">
                  {file.dirty && <span className="text-yellow-500 mr-1">●</span>}
                  {file.path.split("/").pop()}
                </span>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onCloseFile(file.path);
                  }}
                  className="p-0.5 rounded opacity-0 group-hover:opacity-100 hover:bg-muted-foreground/20"
                >
                  <X className="h-2.5 w-2.5" />
                </button>
              </div>
            ))}
          </div>
        )}

        {/* Project Files */}
        <div className="px-1 py-1">
          <span className="px-2 py-1 text-[10px] uppercase tracking-wider text-muted-foreground font-medium block">
            Project Files
          </span>
          {filtered
            .slice()
            .sort((a, b) => {
              if (a.type !== b.type) return a.type === "dir" ? -1 : 1;
              return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
            })
            .map((node) => (
              <FileTreeItem
                key={node.path}
                node={node}
                selectedFile={selectedFile}
                onSelect={onSelectFile}
                onContextMenu={onContextMenu}
                onPrefetch={onPrefetch}
              />
            ))}
        </div>
      </ScrollArea>
    </div>
  );
}
