import { useCallback, useRef, useState } from "react";
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

  const run = useCallback(async (path: string) => {
    setState({ running: true, runId: null, lines: [], exitCode: null });

    try {
      const { id } = await startRun(path);
      setState((s) => ({ ...s, runId: id }));

      const ws = connectRunStream(id, (line) => {
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
      setState((s) => ({ ...s, running: false }));
      console.error("run failed:", err);
    }
  }, []);

  const stop = useCallback(async () => {
    if (state.runId) {
      try {
        await stopRun(state.runId);
      } catch {}
    }
    wsRef.current?.close();
    setState((s) => ({ ...s, running: false }));
  }, [state.runId]);

  const clear = useCallback(() => {
    setState((s) => ({ ...s, lines: [], exitCode: null }));
  }, []);

  return { ...state, run, stop, clear };
}
