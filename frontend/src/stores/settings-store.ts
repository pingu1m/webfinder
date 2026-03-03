import { create } from "zustand";
import { putSettings } from "@/api/client";

export interface Settings {
  autoSave: boolean;
  fontSize: number;
  tabSize: number;
  wordWrap: string;
  theme: string;
}

interface SettingsState extends Settings {
  initialized: boolean;
  init: (config: {
    auto_save: boolean;
    font_size: number;
    tab_size: number;
    word_wrap: string;
    editor_theme: string;
  }) => void;
  update: <K extends keyof Settings>(key: K, value: Settings[K]) => void;
}

let persistTimer: ReturnType<typeof setTimeout> | null = null;
let pendingPatch: Record<string, unknown> = {};

const KEY_MAP: Record<string, string> = {
  autoSave: "auto_save",
  fontSize: "font_size",
  tabSize: "tab_size",
  wordWrap: "word_wrap",
  theme: "theme",
};

function flushSettings() {
  if (persistTimer) {
    clearTimeout(persistTimer);
    persistTimer = null;
  }
  if (Object.keys(pendingPatch).length === 0) return;
  const patch = pendingPatch;
  pendingPatch = {};
  putSettings(patch).catch((err) =>
    console.error("failed to persist setting:", err)
  );
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  initialized: false,
  autoSave: false,
  fontSize: 14,
  tabSize: 2,
  wordWrap: "on",
  theme: "light",

  init: (config) => {
    if (get().initialized) return;
    set({
      initialized: true,
      autoSave: config.auto_save,
      fontSize: config.font_size,
      tabSize: config.tab_size,
      wordWrap: config.word_wrap,
      theme: config.editor_theme,
    });
  },

  update: (key, value) => {
    set({ [key]: value });

    const apiKey = KEY_MAP[key];
    if (apiKey) {
      pendingPatch[apiKey] = value;
      if (persistTimer) clearTimeout(persistTimer);
      persistTimer = setTimeout(flushSettings, 300);
    }
  },
}));
