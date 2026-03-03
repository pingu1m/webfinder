import React, { lazy, Suspense, useCallback, useMemo, useRef } from "react";
import { FileCode, Play, X } from "lucide-react";
import { useFileContent } from "@/hooks/use-file-content";
import { useEditorStore } from "@/stores/editor-store";
import { cn } from "@/lib/utils";
import type { editor as MonacoEditor } from "monaco-editor";

const Editor = lazy(() => import("@monaco-editor/react"));

function getLanguage(path: string): string {
  const ext = path.split(".").pop()?.toLowerCase() ?? "";
  const map: Record<string, string> = {
    ts: "typescript",
    tsx: "typescriptreact",
    js: "javascript",
    jsx: "javascriptreact",
    json: "json",
    html: "html",
    css: "css",
    scss: "scss",
    md: "markdown",
    py: "python",
    rs: "rust",
    go: "go",
    rb: "ruby",
    sh: "shell",
    bash: "shell",
    yaml: "yaml",
    yml: "yaml",
    toml: "toml",
    sql: "sql",
    xml: "xml",
    svg: "xml",
  };
  return map[ext] ?? "plaintext";
}

interface EditorPanelProps {
  runnableExtensions: string[];
  onRun: (path: string) => void;
  isRunning: boolean;
}

export function EditorPanel({ runnableExtensions, onRun, isRunning }: EditorPanelProps) {
  const { openFiles, activeFile, setActiveFile, closeFile } = useEditorStore();
  const { data: fileData, saveFile } = useFileContent(activeFile);
  const editorRef = useRef<MonacoEditor.IStandaloneCodeEditor | null>(null);

  const language = useMemo(
    () => (activeFile ? getLanguage(activeFile) : "plaintext"),
    [activeFile]
  );

  const isRunnable = useMemo(() => {
    if (!activeFile) return false;
    const ext = activeFile.split(".").pop()?.toLowerCase() ?? "";
    return runnableExtensions.includes(ext);
  }, [activeFile, runnableExtensions]);

  const handleEditorMount = useCallback(
    (editor: MonacoEditor.IStandaloneCodeEditor) => {
      editorRef.current = editor;
    },
    []
  );

  const handleChange = useCallback(
    (value: string | undefined) => {
      if (value !== undefined) {
        saveFile(value);
      }
    },
    [saveFile]
  );

  if (!activeFile) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        <div className="text-center">
          <FileCode className="h-12 w-12 mx-auto mb-3 opacity-30" />
          <p className="text-sm">Select a file to edit</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col min-h-0">
      {/* Tab bar */}
      <div className="flex items-center border-b bg-muted/30 overflow-x-auto shrink-0">
        {openFiles.map((file) => (
          <div
            key={file.path}
            className={cn(
              "group flex items-center gap-1.5 border-r px-3 py-1.5 text-xs cursor-pointer select-none shrink-0",
              file.path === activeFile
                ? "bg-background text-foreground border-b-2 border-b-primary"
                : "text-muted-foreground hover:bg-muted/50"
            )}
            onClick={() => setActiveFile(file.path)}
          >
            <FileCode className="h-3.5 w-3.5 shrink-0" />
            <span className="truncate">
              {file.dirty && <span className="text-yellow-500 mr-0.5">●</span>}
              {file.path.split("/").pop()}
            </span>
            <button
              onClick={(e) => {
                e.stopPropagation();
                closeFile(file.path);
              }}
              className={cn(
                "ml-1 rounded p-0.5 hover:bg-muted-foreground/20",
                file.path === activeFile ? "opacity-100" : "opacity-0 group-hover:opacity-100"
              )}
            >
              <X className="h-3 w-3" />
            </button>
          </div>
        ))}

        {/* Run button */}
        {isRunnable && (
          <button
            className={cn(
              "ml-auto mr-2 flex items-center gap-1 px-2 py-1 text-xs rounded",
              isRunning
                ? "bg-red-500/10 text-red-500"
                : "bg-green-500/10 text-green-500 hover:bg-green-500/20"
            )}
            onClick={() => activeFile && onRun(activeFile)}
            disabled={isRunning}
          >
            <Play className="h-3 w-3" />
            {isRunning ? "Running..." : "Run"}
          </button>
        )}
      </div>

      {/* File path bar */}
      <div className="flex items-center px-3 py-1 border-b text-xs text-muted-foreground font-mono bg-muted/20 shrink-0">
        {activeFile}
      </div>

      {/* Monaco Editor */}
      <div className="flex-1 min-h-0">
        <Suspense
          fallback={
            <div className="flex-1 flex items-center justify-center">
              <div className="animate-pulse text-sm text-muted-foreground">
                Loading editor...
              </div>
            </div>
          }
        >
          <Editor
            key={activeFile}
            height="100%"
            language={language}
            value={fileData?.content ?? ""}
            onChange={handleChange}
            onMount={handleEditorMount}
            theme="vs-dark"
            options={{
              fontSize: 14,
              tabSize: 2,
              wordWrap: "on",
              minimap: { enabled: false },
              codeLens: false,
              "semanticHighlighting.enabled": false,
              scrollBeyondLastLine: false,
              renderLineHighlight: "line",
              padding: { top: 8 },
              smoothScrolling: true,
              cursorBlinking: "smooth",
              automaticLayout: true,
            }}
          />
        </Suspense>
      </div>
    </div>
  );
}
