import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface OpenFile {
  path: string;
  dirty: boolean;
  etag: string | null;
}

interface EditorState {
  openFiles: OpenFile[];
  activeFile: string | null;
  conflict: string | null;
  recentSaves: Record<string, number>;
  pendingContent: Record<string, string>;

  openFile: (path: string, etag?: string | null) => void;
  closeFile: (path: string) => void;
  closeAllFiles: () => void;
  closeFilesUnderPath: (dirPath: string) => void;
  renameOpenFile: (oldPath: string, newPath: string) => void;
  setActiveFile: (path: string) => void;
  setDirty: (path: string, dirty: boolean) => void;
  setEtag: (path: string, etag: string | null) => void;
  isOpen: (path: string) => boolean;
  markRecentSave: (path: string) => void;
  isRecentSave: (path: string) => boolean;
  setConflict: (path: string | null) => void;
  setPendingContent: (path: string, content: string) => void;
  clearPendingContent: (path: string) => void;
}

const SELF_SAVE_WINDOW_MS = 2000;

export const useEditorStore = create<EditorState>()(
  persist(
    (set, get) => ({
      openFiles: [],
      activeFile: null,
      conflict: null,
      recentSaves: {},
      pendingContent: {},

      openFile: (path, etag = null) => {
        const { openFiles } = get();
        if (!openFiles.some((f) => f.path === path)) {
          set({ openFiles: [...openFiles, { path, dirty: false, etag }] });
        }
        set({ activeFile: path });
      },

      closeFile: (path) => {
        const { openFiles, activeFile, conflict, pendingContent } = get();
        const remaining = openFiles.filter((f) => f.path !== path);
        let newActive = activeFile;
        if (activeFile === path) {
          newActive =
            remaining.length > 0 ? remaining[remaining.length - 1].path : null;
        }
        const { [path]: _, ...restPending } = pendingContent;
        set({
          openFiles: remaining,
          activeFile: newActive,
          conflict: conflict === path ? null : conflict,
          pendingContent: restPending,
        });
      },

      closeAllFiles: () =>
        set({ openFiles: [], activeFile: null, conflict: null, pendingContent: {} }),

      closeFilesUnderPath: (dirPath) => {
        const prefix = dirPath.endsWith("/") ? dirPath : dirPath + "/";
        const state = get();
        const toClose = state.openFiles.filter((f) => f.path.startsWith(prefix));
        if (toClose.length === 0) return;

        const remaining = state.openFiles.filter((f) => !f.path.startsWith(prefix));
        const newPending = { ...state.pendingContent };
        for (const f of toClose) delete newPending[f.path];

        let newActive = state.activeFile;
        if (newActive && newActive.startsWith(prefix)) {
          newActive = remaining.length > 0 ? remaining[remaining.length - 1].path : null;
        }

        set({
          openFiles: remaining,
          activeFile: newActive,
          pendingContent: newPending,
          conflict: state.conflict?.startsWith(prefix) ? null : state.conflict,
        });
      },

      renameOpenFile: (oldPath, newPath) => {
        const state = get();
        const { [oldPath]: pending, ...restPending } = state.pendingContent;
        const { [oldPath]: recentTs, ...restRecent } = state.recentSaves;

        set({
          openFiles: state.openFiles.map((f) =>
            f.path === oldPath ? { ...f, path: newPath } : f
          ),
          activeFile: state.activeFile === oldPath ? newPath : state.activeFile,
          pendingContent:
            pending !== undefined ? { ...restPending, [newPath]: pending } : restPending,
          recentSaves:
            recentTs !== undefined ? { ...restRecent, [newPath]: recentTs } : restRecent,
          conflict: state.conflict === oldPath ? newPath : state.conflict,
        });
      },

      setActiveFile: (path) => set({ activeFile: path }),

      setDirty: (path, dirty) => {
        const file = get().openFiles.find((f) => f.path === path);
        if (!file || file.dirty === dirty) return;
        set((state) => ({
          openFiles: state.openFiles.map((f) =>
            f.path === path ? { ...f, dirty } : f
          ),
        }));
      },

      setEtag: (path, etag) => {
        set((state) => ({
          openFiles: state.openFiles.map((f) =>
            f.path === path ? { ...f, etag } : f
          ),
        }));
      },

      isOpen: (path) => get().openFiles.some((f) => f.path === path),

      markRecentSave: (path) => {
        const now = Date.now();
        const prev = get().recentSaves;
        const pruned: Record<string, number> = {};
        for (const [k, v] of Object.entries(prev)) {
          if (now - v < SELF_SAVE_WINDOW_MS) pruned[k] = v;
        }
        pruned[path] = now;
        set({ recentSaves: pruned });
      },

      isRecentSave: (path) => {
        const ts = get().recentSaves[path];
        return !!ts && Date.now() - ts < SELF_SAVE_WINDOW_MS;
      },

      setConflict: (path) => set({ conflict: path }),

      setPendingContent: (path, content) => {
        set((state) => ({
          pendingContent: { ...state.pendingContent, [path]: content },
        }));
      },

      clearPendingContent: (path) => {
        const { [path]: _, ...rest } = get().pendingContent;
        set({ pendingContent: rest });
      },
    }),
    {
      name: "webfinder-editor",
      version: 1,
      partialize: (s) => ({
        openFiles: s.openFiles.map((f) => ({ path: f.path })),
        activeFile: s.activeFile,
      }),
      merge: (persisted, currentState) => {
        const saved = persisted as { openFiles?: { path: string }[]; activeFile?: string | null } | null;
        if (!saved) return currentState;
        const openFiles = (saved.openFiles ?? []).map((f) => ({
          path: f.path,
          dirty: false,
          etag: null,
        }));
        return {
          ...currentState,
          openFiles,
          activeFile: saved.activeFile ?? null,
        };
      },
    },
  ),
);
