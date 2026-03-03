import { create } from "zustand";
import { persist } from "zustand/middleware";

interface UiState {
  sidebarCollapsed: boolean;
  sidebarWidth: number;
  expandedFolders: Record<string, boolean>;

  setSidebarCollapsed: (collapsed: boolean) => void;
  toggleSidebar: () => void;
  setSidebarWidth: (width: number) => void;
  toggleFolder: (path: string) => void;
  isFolderExpanded: (path: string) => boolean;
  collapseAllFolders: () => void;
}

const SIDEBAR_DEFAULT_WIDTH = 256;

export const useUiStore = create<UiState>()(
  persist(
    (set, get) => ({
      sidebarCollapsed: false,
      sidebarWidth: SIDEBAR_DEFAULT_WIDTH,
      expandedFolders: {},

      setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
      toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),

      setSidebarWidth: (width) => set({ sidebarWidth: width }),

      toggleFolder: (path) =>
        set((s) => ({
          expandedFolders: {
            ...s.expandedFolders,
            [path]: !s.expandedFolders[path],
          },
        })),

      isFolderExpanded: (path) => !!get().expandedFolders[path],

      collapseAllFolders: () => set({ expandedFolders: {} }),
    }),
    {
      name: "webfinder-ui",
      version: 1,
      partialize: (s) => ({
        sidebarCollapsed: s.sidebarCollapsed,
        sidebarWidth: s.sidebarWidth,
        expandedFolders: s.expandedFolders,
      }),
    },
  ),
);
