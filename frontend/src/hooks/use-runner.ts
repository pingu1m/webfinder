import { useCallback, useEffect, useRef, useState } from "react";
import { startRun, stopRun, connectRunStream } from "@/api/client";
import type { OutputLine } from "@/api/types";

export interface RunState {
  running: boolean;
  runId: string | null;
  lines: OutputLine[];
  exitCode: number | null;
}

export function useRunner() {
  const [state, setState] = useState<RunState>({
    running: false,
    runId: null,
    lines: [],
    exitCode: null,
  });

  const wsRef = useRef<WebSocket | null>(null);
  const runIdRef = useRef<string | null>(null);
  const mountedRef = useRef(true);

  useEffect(() => {
    return () => {
      mountedRef.current = false;
      wsRef.current?.close();
      wsRef.current = null;
    };
  }, []);

  const run = useCallback(async (path: string) => {
    // Tear down any previous run's WebSocket before starting a new one
    wsRef.current?.close();
    wsRef.current = null;

    setState({ running: true, runId: null, lines: [], exitCode: null });
    runIdRef.current = null;

    try {
      const { id } = await startRun(path);
      if (!mountedRef.current) return;

      runIdRef.current = id;
      setState((s) => ({ ...s, runId: id }));

      const ws = connectRunStream(id, (line) => {
        if (!mountedRef.current) return;
        if (line.stream === "exit") {
          setState((s) => ({
            ...s,
            running: false,
            exitCode: parseInt(line.data, 10),
          }));
          ws.close();
        } else {
          setState((s) => ({
            ...s,
            lines: [...s.lines, line as OutputLine],
          }));
        }
      });
      wsRef.current = ws;
    } catch (err) {
      if (!mountedRef.current) return;
      setState((s) => ({ ...s, running: false }));
      console.error("run failed:", err);
    }
  }, []);

  const stop = useCallback(async () => {
    // Use ref for current runId — avoids stale closure over state.runId
    const id = runIdRef.current;
    if (id) {
      try {
        await stopRun(id);
      } catch { /* server may already have cleaned up */ }
    }
    wsRef.current?.close();
    wsRef.current = null;
    runIdRef.current = null;
    setState((s) => ({ ...s, running: false }));
  }, []);

  const clear = useCallback(() => {
    setState((s) => ({ ...s, lines: [], exitCode: null }));
  }, []);

  return { ...state, run, stop, clear };
}
