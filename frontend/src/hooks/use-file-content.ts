import { QueryClient, useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useRef } from "react";
import { getFile, putFile } from "@/api/client";
import { useEditorStore } from "@/stores/editor-store";
import { useSettingsStore } from "@/stores/settings-store";
import type { FileResponse } from "@/api/types";

// ---------------------------------------------------------------------------
// Module-level singletons
// ---------------------------------------------------------------------------

// Per-file auto-save timers — survive re-renders, scoped per file path.
const saveTimers = new Map<string, ReturnType<typeof setTimeout>>();

// QueryClient reference set by the hook on first render.  persistFile and
// removeFileQuery use it to keep the React Query cache in sync with disk.
let _qc: QueryClient | null = null;

// ---------------------------------------------------------------------------
// Public helpers (importable by App.tsx / other modules)
// ---------------------------------------------------------------------------

export function cancelFileTimer(path: string) {
  const t = saveTimers.get(path);
  if (t) {
    clearTimeout(t);
    saveTimers.delete(path);
  }
}

/** Remove the React Query cache entry for a file.
 *  Call this when a file is closed so reopening always fetches fresh content. */
export function removeFileQuery(path: string) {
  _qc?.removeQueries({ queryKey: ["file", path] });
}

// ---------------------------------------------------------------------------
// Save logic
// ---------------------------------------------------------------------------

async function persistFile(filePath: string) {
  const snapshot = useEditorStore.getState().pendingContent[filePath];
  if (snapshot === undefined) return;

  cancelFileTimer(filePath);

  try {
    await putFile(filePath, snapshot);
    const s = useEditorStore.getState();
    s.markRecentSave(filePath);

    // Only mark clean if the user hasn't typed more while the save was in-flight.
    if (s.pendingContent[filePath] === snapshot) {
      s.setDirty(filePath, false);
    }

    // Invalidate the React Query cache so it refetches the new disk content.
    // While the refetch is in-flight, pendingContent still serves as the
    // display source, so the user never sees a flash of stale data.
    _qc?.invalidateQueries({ queryKey: ["file", filePath] });
  } catch (err) {
    console.error("save failed:", err);
  }
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

export function useFileContent(path: string | null) {
  const queryClient = useQueryClient();
  // Keep the module-level reference current so persistFile / removeFileQuery
  // can reach the QueryClient without prop-drilling.
  _qc = queryClient;

  const setEtag = useEditorStore((s) => s.setEtag);
  const setDirty = useEditorStore((s) => s.setDirty);
  const setPendingContent = useEditorStore((s) => s.setPendingContent);
  const openFiles = useEditorStore((s) => s.openFiles);
  const autoSave = useSettingsStore((s) => s.autoSave);
  const currentEtag = openFiles.find((f) => f.path === path)?.etag ?? undefined;

  const autoSaveRef = useRef(autoSave);
  autoSaveRef.current = autoSave;

  const query = useQuery({
    queryKey: ["file", path],
    queryFn: async () => {
      if (!path) return null;
      const result = await getFile(path, currentEtag);
      if (result.notModified) {
        // Preserve the existing cached data — returning null would wipe it.
        return queryClient.getQueryData<FileResponse | null>(["file", path]) ?? null;
      }
      if (result.etag && path) {
        setEtag(path, result.etag);
      }
      return result.data;
    },
    enabled: !!path,
    staleTime: 30_000,
  });

  // Display priority: pending edits > fetched content > empty.
  // getState() is intentionally non-reactive here — re-renders are already
  // triggered by activeFile changes (tab switch) and dirty flag changes (save).
  const pending = useEditorStore.getState().pendingContent[path ?? ""];
  const displayContent = pending ?? query.data?.content ?? "";

  const handleChange = useCallback(
    (content: string) => {
      if (!path) return;

      setPendingContent(path, content);
      setDirty(path, true);

      if (autoSaveRef.current) {
        cancelFileTimer(path);
        const filePath = path;
        saveTimers.set(
          filePath,
          setTimeout(() => persistFile(filePath), 150)
        );
      }
    },
    [path, setPendingContent, setDirty]
  );

  const saveNow = useCallback(() => {
    if (!path) return;
    persistFile(path);
  }, [path]);

  return {
    isLoading: query.isLoading,
    displayContent,
    handleChange,
    saveNow,
  };
}
