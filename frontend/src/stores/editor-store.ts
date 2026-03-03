import { create } from "zustand";

export interface OpenFile {
  path: string;
  dirty: boolean;
  etag: string | null;
}

interface EditorState {
  openFiles: OpenFile[];
  activeFile: string | null;

  openFile: (path: string, etag?: string | null) => void;
  closeFile: (path: string) => void;
  closeAllFiles: () => void;
  setActiveFile: (path: string) => void;
  setDirty: (path: string, dirty: boolean) => void;
  setEtag: (path: string, etag: string | null) => void;
  isOpen: (path: string) => boolean;
}

export const useEditorStore = create<EditorState>((set, get) => ({
  openFiles: [],
  activeFile: null,

  openFile: (path, etag = null) => {
    const { openFiles } = get();
    if (!openFiles.some((f) => f.path === path)) {
      set({ openFiles: [...openFiles, { path, dirty: false, etag }] });
    }
    set({ activeFile: path });
  },

  closeFile: (path) => {
    const { openFiles, activeFile } = get();
    const remaining = openFiles.filter((f) => f.path !== path);
    let newActive = activeFile;
    if (activeFile === path) {
      newActive =
        remaining.length > 0 ? remaining[remaining.length - 1].path : null;
    }
    set({ openFiles: remaining, activeFile: newActive });
  },

  closeAllFiles: () => set({ openFiles: [], activeFile: null }),

  setActiveFile: (path) => set({ activeFile: path }),

  setDirty: (path, dirty) => {
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
}));
