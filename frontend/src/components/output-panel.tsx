import { useCallback, useEffect, useRef, useState } from "react";
import { ChevronDown, ChevronUp, Square, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { OutputLine } from "@/api/types";

interface OutputPanelProps {
  lines: OutputLine[];
  running: boolean;
  exitCode: number | null;
  onStop: () => void;
  onClear: () => void;
}

export function OutputPanel({
  lines,
  running,
  exitCode,
  onStop,
  onClear,
}: OutputPanelProps) {
  const [collapsed, setCollapsed] = useState(true);
  const [height, setHeight] = useState(200);
  const scrollRef = useRef<HTMLDivElement>(null);

  // Drag-resize state and listeners stored in refs so the cleanup
  // effect can always reach them, even if the browser loses focus
  // before mouseup fires.
  const dragStateRef = useRef<{ startY: number; startHeight: number } | null>(null);
  const handlersRef = useRef<{ move: (e: MouseEvent) => void; up: () => void } | null>(null);

  // Cleanup drag listeners on unmount (or if they leaked)
  useEffect(() => {
    return () => {
      if (handlersRef.current) {
        document.removeEventListener("mousemove", handlersRef.current.move);
        document.removeEventListener("mouseup", handlersRef.current.up);
        handlersRef.current = null;
      }
    };
  }, []);

  useEffect(() => {
    if (running || lines.length > 0) {
      setCollapsed(false);
    }
  }, [running, lines.length]);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [lines]);

  const handleDragStart = useCallback(
    (e: React.MouseEvent) => {
      // Remove any stale listeners from a previous drag
      if (handlersRef.current) {
        document.removeEventListener("mousemove", handlersRef.current.move);
        document.removeEventListener("mouseup", handlersRef.current.up);
      }

      dragStateRef.current = { startY: e.clientY, startHeight: height };

      const handleMove = (e: MouseEvent) => {
        if (!dragStateRef.current) return;
        const delta = dragStateRef.current.startY - e.clientY;
        setHeight(Math.max(100, Math.min(600, dragStateRef.current.startHeight + delta)));
      };

      const handleUp = () => {
        dragStateRef.current = null;
        document.removeEventListener("mousemove", handleMove);
        document.removeEventListener("mouseup", handleUp);
        handlersRef.current = null;
      };

      handlersRef.current = { move: handleMove, up: handleUp };
      document.addEventListener("mousemove", handleMove);
      document.addEventListener("mouseup", handleUp);
    },
    [height]
  );

  const hasOutput = lines.length > 0 || exitCode !== null;

  if (!hasOutput && !running) return null;

  return (
    <div className="border-t bg-background shrink-0">
      {!collapsed && (
        <div
          className="h-1 cursor-row-resize hover:bg-primary/20 transition-colors"
          onMouseDown={handleDragStart}
        />
      )}

      <div className="flex items-center justify-between px-3 py-1 border-b bg-muted/30">
        <button
          className="flex items-center gap-1.5 text-xs font-medium"
          onClick={() => setCollapsed(!collapsed)}
        >
          {collapsed ? (
            <ChevronUp className="h-3.5 w-3.5" />
          ) : (
            <ChevronDown className="h-3.5 w-3.5" />
          )}
          Output
          {running && (
            <span className="ml-1 h-2 w-2 rounded-full bg-green-500 animate-pulse" />
          )}
          {exitCode !== null && (
            <span
              className={cn(
                "ml-1 px-1.5 py-0.5 rounded text-[10px]",
                exitCode === 0 ? "bg-green-500/10 text-green-500" : "bg-red-500/10 text-red-500"
              )}
            >
              exit {exitCode}
            </span>
          )}
        </button>
        <div className="flex items-center gap-1">
          {running && (
            <Button variant="ghost" size="icon-xs" onClick={onStop} title="Stop">
              <Square className="h-3 w-3 text-red-500" />
            </Button>
          )}
          <Button variant="ghost" size="icon-xs" onClick={onClear} title="Clear">
            <Trash2 className="h-3 w-3" />
          </Button>
        </div>
      </div>

      {!collapsed && (
        <div
          ref={scrollRef}
          className="overflow-auto font-mono text-xs p-3 bg-muted/50 text-foreground"
          style={{ height }}
        >
          {lines.map((line, i) => (
            <div
              key={i}
              className={cn(
                "whitespace-pre-wrap",
                line.stream === "stderr" && "text-destructive"
              )}
            >
              {line.data}
            </div>
          ))}
          {lines.length === 0 && running && (
            <div className="text-muted-foreground animate-pulse">
              Waiting for output...
            </div>
          )}
        </div>
      )}
    </div>
  );
}
