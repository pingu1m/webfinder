import { useCallback, useEffect, useMemo, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { SidebarCustom } from "@/components/sidebar-custom";
import { EditorPanel } from "@/components/editor-panel";
import { OutputPanel } from "@/components/output-panel";
import { ContextMenu, NewItemDialog } from "@/components/context-menu";
import { FileConflictDialog } from "@/components/file-conflict-dialog";
import { SettingsPanel } from "@/components/settings-panel";
import { useFileTree } from "@/hooks/use-file-tree";
import { useRunner } from "@/hooks/use-runner";
import { cancelFileTimer, removeFileQuery } from "@/hooks/use-file-content";
import { useEditorStore } from "@/stores/editor-store";
import { useSettingsStore } from "@/stores/settings-store";
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
  const queryClient = useQueryClient();

  // Data selectors — only re-render when these specific values change
  const openFiles = useEditorStore((s) => s.openFiles);
  const activeFile = useEditorStore((s) => s.activeFile);
  const conflict = useEditorStore((s) => s.conflict);

  // Actions — stable references, never trigger re-renders
  const openFile = useEditorStore((s) => s.openFile);
  const storeCloseFile = useEditorStore((s) => s.closeFile);
  const storeCloseAll = useEditorStore((s) => s.closeAllFiles);
  const closeFilesUnderPath = useEditorStore((s) => s.closeFilesUnderPath);
  const renameOpenFile = useEditorStore((s) => s.renameOpenFile);
  const isOpen = useEditorStore((s) => s.isOpen);
  const isRecentSave = useEditorStore((s) => s.isRecentSave);
  const setConflict = useEditorStore((s) => s.setConflict);
  const setDirty = useEditorStore((s) => s.setDirty);
  const clearPendingContent = useEditorStore((s) => s.clearPendingContent);

  // Safe close: confirm if file has unsaved changes, then clean up all caches.
  const safeCloseFile = useCallback(
    (path: string) => {
      const file = useEditorStore.getState().openFiles.find((f) => f.path === path);
      if (
        file?.dirty &&
        !window.confirm(`"${path.split("/").pop()}" has unsaved changes. Close anyway?`)
      ) {
        return;
      }
      cancelFileTimer(path);
      storeCloseFile(path);
      removeFileQuery(path);
    },
    [storeCloseFile]
  );

  const safeCloseAllFiles = useCallback(() => {
    const dirty = useEditorStore.getState().openFiles.filter((f) => f.dirty);
    if (
      dirty.length > 0 &&
      !window.confirm(
        `${dirty.length} file(s) have unsaved changes. Close all anyway?`
      )
    ) {
      return;
    }
    const files = useEditorStore.getState().openFiles;
    for (const f of files) {
      cancelFileTimer(f.path);
      removeFileQuery(f.path);
    }
    storeCloseAll();
  }, [storeCloseAll]);

  const handleExternalModify = useCallback(
    (path: string) => {
      if (!isOpen(path)) return;
      if (isRecentSave(path)) return;

      const file = useEditorStore.getState().openFiles.find((f) => f.path === path);
      if (file?.dirty) {
        setConflict(path);
      } else {
        clearPendingContent(path);
        queryClient.invalidateQueries({ queryKey: ["file", path] });
      }
    },
    [isOpen, isRecentSave, setConflict, clearPendingContent, queryClient]
  );

  const { data: tree = [] } = useFileTree(handleExternalModify);
  const { data: info } = useQuery({
    queryKey: ["info"],
    queryFn: getInfo,
    staleTime: Infinity,
  });
  const initSettings = useSettingsStore((s) => s.init);
  const currentTheme = useSettingsStore((s) => s.theme);
  useEffect(() => {
    if (info) initSettings(info.config);
  }, [info, initSettings]);

  useEffect(() => {
    const isDark = currentTheme === "vs-dark" || currentTheme === "hc-black";
    document.documentElement.classList.toggle("dark", isDark);
  }, [currentTheme]);

  const runner = useRunner();

  const [showSettings, setShowSettings] = useState(false);
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    node: FileNode | null;
  } | null>(null);
  const [dialog, setDialog] = useState<DialogMode>(null);

  const runnableExtensions = useMemo(() => {
    if (!info) return [];
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
              closeFilesUnderPath(oldPath);
            } else {
              await renameFile(oldPath, newPath);
              renameOpenFile(oldPath, newPath);
              queryClient.removeQueries({ queryKey: ["file", oldPath] });
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
    [dialog, queryClient, openFile, renameOpenFile, closeFilesUnderPath]
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
          const prefix = node.path.endsWith("/") ? node.path : node.path + "/";
          const affected = useEditorStore.getState().openFiles.filter(
            (f) => f.path.startsWith(prefix)
          );
          for (const f of affected) {
            cancelFileTimer(f.path);
            removeFileQuery(f.path);
          }
          closeFilesUnderPath(node.path);
        } else {
          await deleteFile(node.path);
          cancelFileTimer(node.path);
          storeCloseFile(node.path);
          removeFileQuery(node.path);
        }
        queryClient.invalidateQueries({ queryKey: ["tree"] });
      } catch (err) {
        console.error("delete failed:", err);
      }
    },
    [queryClient, storeCloseFile, closeFilesUnderPath]
  );

  const handleConflictKeep = useCallback(() => {
    setConflict(null);
  }, [setConflict]);

  const handleConflictLoad = useCallback(() => {
    if (conflict) {
      clearPendingContent(conflict);
      setDirty(conflict, false);
      queryClient.invalidateQueries({ queryKey: ["file", conflict] });
    }
    setConflict(null);
  }, [conflict, setConflict, clearPendingContent, setDirty, queryClient]);

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
    <div className="flex h-screen w-screen bg-background text-foreground">
      <SidebarCustom
        tree={tree}
        selectedFile={activeFile}
        openFiles={openFiles}
        onSelectFile={(path) => {
          setShowSettings(false);
          handleSelectFile(path);
        }}
        onCloseFile={safeCloseFile}
        onCloseAllFiles={safeCloseAllFiles}
        onContextMenu={handleContextMenu}
        onPrefetch={handlePrefetch}
        onNewFile={() => setDialog({ type: "new-file", parentPath: "" })}
        onNewFolder={() => setDialog({ type: "new-folder", parentPath: "" })}
        onSettings={() => setShowSettings(true)}
      />

      <div className="flex-1 flex flex-col min-w-0">
        {showSettings ? (
          <SettingsPanel onClose={() => setShowSettings(false)} />
        ) : (
          <>
            <EditorPanel
              runnableExtensions={runnableExtensions}
              onRun={handleRun}
              isRunning={runner.running}
              onCloseFile={safeCloseFile}
            />
            <OutputPanel
              lines={runner.lines}
              running={runner.running}
              exitCode={runner.exitCode}
              onStop={runner.stop}
              onClear={runner.clear}
            />
          </>
        )}
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

      {conflict && (
        <FileConflictDialog
          path={conflict}
          onKeep={handleConflictKeep}
          onLoadFromDisk={handleConflictLoad}
        />
      )}
    </div>
  );
}
