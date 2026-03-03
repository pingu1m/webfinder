import type {
  FileNode,
  FileResponse,
  InfoResponse,
  RunResponse,
  RunStatusResponse,
  SearchResult,
} from "./types";

// Derive base path from the page URL so API calls work behind a reverse proxy.
// e.g. if loaded from /api/v1/workspace/default/proxy/8819/, BASE becomes that path.
const BASE = (() => {
  const path = window.location.pathname;
  return path.endsWith("/") ? path.slice(0, -1) : path;
})();

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${url}`, init);
  if (!res.ok) {
    const body = await res.text();
    throw new Error(`${res.status}: ${body}`);
  }
  return res.json();
}

// Tree
export async function getTree(): Promise<FileNode[]> {
  return fetchJson("/api/tree");
}

// File
export async function getFile(
  path: string,
  etag?: string
): Promise<{ data: FileResponse | null; etag: string | null; notModified: boolean }> {
  const headers: Record<string, string> = {};
  if (etag) headers["If-None-Match"] = `"${etag}"`;

  const res = await fetch(`${BASE}/api/file?path=${encodeURIComponent(path)}`, {
    headers,
  });

  if (res.status === 304) {
    return { data: null, etag: etag ?? null, notModified: true };
  }
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);

  const newEtag = res.headers.get("etag")?.replace(/"/g, "") || null;
  const data: FileResponse = await res.json();
  return { data, etag: newEtag, notModified: false };
}

export async function putFile(path: string, content: string): Promise<void> {
  const res = await fetch(
    `${BASE}/api/file?path=${encodeURIComponent(path)}`,
    {
      method: "PUT",
      body: content,
      headers: { "Content-Type": "text/plain" },
    }
  );
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
}

export async function createFile(
  path: string,
  content = ""
): Promise<void> {
  await fetchJson("/api/file", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ path, content }),
  });
}

export async function deleteFile(path: string): Promise<void> {
  const res = await fetch(
    `${BASE}/api/file?path=${encodeURIComponent(path)}`,
    { method: "DELETE" }
  );
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
}

export async function renameFile(from: string, to: string): Promise<void> {
  await fetchJson("/api/file/rename", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ from, to }),
  });
}

export async function copyFile(from: string, to: string): Promise<void> {
  await fetchJson("/api/file/copy", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ from, to }),
  });
}

// Folder
export async function createFolder(path: string): Promise<void> {
  await fetchJson("/api/folder", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ path }),
  });
}

export async function deleteFolder(path: string): Promise<void> {
  const res = await fetch(
    `${BASE}/api/folder?path=${encodeURIComponent(path)}`,
    { method: "DELETE" }
  );
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
}

export async function renameFolder(from: string, to: string): Promise<void> {
  await fetchJson("/api/folder/rename", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ from, to }),
  });
}

// Search
export async function searchFiles(
  q: string,
  mode: "filename" | "content" = "filename"
): Promise<SearchResult[]> {
  return fetchJson(
    `/api/search?q=${encodeURIComponent(q)}&mode=${mode}`
  );
}

// Info
export async function getInfo(): Promise<InfoResponse> {
  return fetchJson("/api/info");
}

// Settings
export async function putSettings(
  settings: Partial<{
    auto_save: boolean;
    font_size: number;
    tab_size: number;
    word_wrap: string;
    theme: string;
  }>
): Promise<void> {
  const res = await fetch(`${BASE}/api/settings`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(settings),
  });
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
}

// Runner
export async function startRun(path: string): Promise<RunResponse> {
  return fetchJson("/api/run", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ path }),
  });
}

export async function stopRun(id: string): Promise<void> {
  const res = await fetch(`${BASE}/api/run/${id}`, { method: "DELETE" });
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
}

export async function getRunStatus(id: string): Promise<RunStatusResponse> {
  return fetchJson(`/api/run/${id}`);
}

// WebSocket helpers

export interface ManagedSocket {
  close: () => void;
}

/**
 * Connect to the file-watcher WebSocket with automatic reconnect on disconnect.
 * Returns a handle with a `close()` method to permanently disconnect.
 */
export function connectWatch(
  onEvent: (event: { kind: string; path: string }) => void
): ManagedSocket {
  let ws: WebSocket | null = null;
  let closed = false;
  let retryMs = 1000;

  function connect() {
    if (closed) return;
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    ws = new WebSocket(`${protocol}//${window.location.host}${BASE}/api/watch`);

    ws.onopen = () => {
      retryMs = 1000;
    };

    ws.onmessage = (e) => {
      try {
        onEvent(JSON.parse(e.data));
      } catch { /* malformed message */ }
    };

    ws.onclose = () => {
      if (closed) return;
      setTimeout(connect, retryMs);
      retryMs = Math.min(retryMs * 2, 30_000);
    };

    ws.onerror = () => {
      ws?.close();
    };
  }

  connect();

  return {
    close: () => {
      closed = true;
      ws?.close();
    },
  };
}

export function connectRunStream(
  id: string,
  onLine: (line: { stream: string; data: string }) => void
): WebSocket {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const ws = new WebSocket(
    `${protocol}//${window.location.host}${BASE}/api/run/${id}/stream`
  );
  ws.onmessage = (e) => {
    try {
      onLine(JSON.parse(e.data));
    } catch { /* malformed message */ }
  };
  return ws;
}
