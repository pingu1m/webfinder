export interface FileNode {
  name: string;
  path: string;
  type: "file" | "dir";
  children?: FileNode[];
}

export interface FileResponse {
  path: string;
  content: string | null;
  language: string;
  size: number;
  modified: string;
  binary: boolean;
}

export interface InfoResponse {
  root: string;
  name: string;
  version: string;
  config: {
    runners: string[];
    editor_theme: string;
    auto_save: boolean;
    font_size: number;
    tab_size: number;
    word_wrap: string;
    show_hidden: boolean;
  };
}

export interface SearchResult {
  path: string;
  name: string;
  type: string;
  line?: number;
  snippet?: string;
}

export interface RunResponse {
  id: string;
}

export interface RunStatusResponse {
  id: string;
  running: boolean;
  exit_code: number | null;
}

export interface FsEvent {
  kind: "create" | "modify" | "remove" | "rename";
  path: string;
}

export interface OutputLine {
  stream: "stdout" | "stderr" | "exit";
  data: string;
}
