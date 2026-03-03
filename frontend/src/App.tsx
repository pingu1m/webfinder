import { useCallback, useMemo, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { SidebarCustom } from "@/components/sidebar-custom";
import { EditorPanel } from "@/components/editor-panel";
import { OutputPanel } from "@/components/output-panel";
import { ContextMenu, NewItemDialog } from "@/components/context-menu";
import { useFileTree } from "@/hooks/use-file-tree";
import { useRunner } from "@/hooks/use-runner";
import { useEditorStore } from "@/stores/editor-store";
import {
  getInfo,
  getFile,
  createFile,
  createFolder,
  deleteFile,
  deleteFolder,
  renameFile,
  renameFolder,
  copyFile,
} from "@/api/client";
import type { FileNode } from "@/api/types";

type DialogMode =
  | { type: "new-file"; parentPath: string }
  | { type: "new-folder"; parentPath: string }
  | { type: "rename"; node: FileNode }
  | { type: "copy"; node: FileNode }
  | null;

export default function App() {
  const { data: tree = [] } = useFileTree();
  const { data: info } = useQuery({
    queryKey: ["info"],
    queryFn: getInfo,
    staleTime: Infinity,
  });
  const queryClient = useQueryClient();

  const { openFiles, activeFile, openFile, closeFile, closeAllFiles } =
    useEditorStore();
  const runner = useRunner();

  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    node: FileNode | null;
  } | null>(null);
  const [dialog, setDialog] = useState<DialogMode>(null);

  // Derive runnable extensions from info
  const runnableExtensions = useMemo(() => {
    if (!info) return [];
    // We'll fetch the full config from the runners - for now use common defaults
    return ["py", "js", "mjs", "ts", "tsx", "sh", "bash", "rs"];
  }, [info]);

  const handleSelectFile = useCallback(
    (path: string) => {
      openFile(path);
    },
    [openFile]
  );

  const handlePrefetch = useCallback(
    (path: string) => {
      queryClient.prefetchQuery({
        queryKey: ["file", path],
        queryFn: () => getFile(path).then((r) => r.data),
        staleTime: 30_000,
      });
    },
    [queryClient]
  );

  const handleContextMenu = useCallback(
    (e: React.MouseEvent, node: FileNode) => {
      e.preventDefault();
      setContextMenu({ x: e.clientX, y: e.clientY, node });
    },
    []
  );

  const handleRun = useCallback(
    (path: string) => {
      runner.run(path);
    },
    [runner]
  );

  // Dialog actions
  const handleDialogConfirm = useCallback(
    async (value: string) => {
      if (!dialog) return;

      try {
        switch (dialog.type) {
          case "new-file": {
            const path = dialog.parentPath ? `${dialog.parentPath}/${value}` : value;
            await createFile(path);
            queryClient.invalidateQueries({ queryKey: ["tree"] });
            openFile(path);
            break;
          }
          case "new-folder": {
            const path = dialog.parentPath ? `${dialog.parentPath}/${value}` : value;
            await createFolder(path);
            queryClient.invalidateQueries({ queryKey: ["tree"] });
            break;
          }
          case "rename": {
            const oldPath = dialog.node.path;
            const parts = oldPath.split("/");
            parts[parts.length - 1] = value;
            const newPath = parts.join("/");
            if (dialog.node.type === "dir") {
              await renameFolder(oldPath, newPath);
            } else {
              await renameFile(oldPath, newPath);
            }
            queryClient.invalidateQueries({ queryKey: ["tree"] });
            break;
          }
          case "copy": {
            const oldPath = dialog.node.path;
            const ext = oldPath.includes(".") ? oldPath.substring(oldPath.lastIndexOf(".")) : "";
            const base = oldPath.includes(".")
              ? oldPath.substring(0, oldPath.lastIndexOf("."))
              : oldPath;
            const newPath = value.includes("/") ? value : `${base}-copy${ext}`;
            await copyFile(oldPath, value.includes("/") ? value : newPath);
            queryClient.invalidateQueries({ queryKey: ["tree"] });
            break;
          }
        }
      } catch (err) {
        console.error("operation failed:", err);
      }

      setDialog(null);
    },
    [dialog, queryClient, openFile]
  );

  const handleDelete = useCallback(
    async (node: FileNode) => {
      const confirmed = window.confirm(
        `Delete ${node.type === "dir" ? "folder" : "file"} "${node.name}"?`
      );
      if (!confirmed) return;

      try {
        if (node.type === "dir") {
          await deleteFolder(node.path);
        } else {
          await deleteFile(node.path);
          closeFile(node.path);
        }
        queryClient.invalidateQueries({ queryKey: ["tree"] });
      } catch (err) {
        console.error("delete failed:", err);
      }
    },
    [queryClient, closeFile]
  );

  const dialogTitle = dialog
    ? dialog.type === "new-file"
      ? "New File"
      : dialog.type === "new-folder"
      ? "New Folder"
      : dialog.type === "rename"
      ? `Rename "${dialog.node.name}"`
      : `Copy "${dialog.node.name}" to`
    : "";

  const dialogDefault = dialog
    ? dialog.type === "rename"
      ? dialog.node.name
      : dialog.type === "copy"
      ? dialog.node.path
      : ""
    : "";

  return (
    <div className="flex h-screen w-screen bg-background text-foreground dark">
      <SidebarCustom
        tree={tree}
        selectedFile={activeFile}
        openFiles={openFiles}
        onSelectFile={handleSelectFile}
        onCloseFile={closeFile}
        onCloseAllFiles={closeAllFiles}
        onContextMenu={handleContextMenu}
        onPrefetch={handlePrefetch}
        onNewFile={() => setDialog({ type: "new-file", parentPath: "" })}
        onNewFolder={() => setDialog({ type: "new-folder", parentPath: "" })}
        collapsed={sidebarCollapsed}
        onToggle={() => setSidebarCollapsed(!sidebarCollapsed)}
      />

      <div className="flex-1 flex flex-col min-w-0">
        <EditorPanel
          runnableExtensions={runnableExtensions}
          onRun={handleRun}
          isRunning={runner.running}
        />
        <OutputPanel
          lines={runner.lines}
          running={runner.running}
          exitCode={runner.exitCode}
          onStop={runner.stop}
          onClear={runner.clear}
        />
      </div>

      <ContextMenu
        state={contextMenu}
        onClose={() => setContextMenu(null)}
        onNewFile={(parentPath) =>
          setDialog({ type: "new-file", parentPath })
        }
        onNewFolder={(parentPath) =>
          setDialog({ type: "new-folder", parentPath })
        }
        onRename={(node) => setDialog({ type: "rename", node })}
        onDelete={handleDelete}
        onCopy={(node) => setDialog({ type: "copy", node })}
      />

      <NewItemDialog
        open={!!dialog}
        title={dialogTitle}
        defaultValue={dialogDefault}
        onConfirm={handleDialogConfirm}
        onCancel={() => setDialog(null)}
      />
    </div>
  );
}
