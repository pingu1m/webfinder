import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { getTree, connectWatch } from "@/api/client";
import type { ManagedSocket } from "@/api/client";
import type { FileNode, FsEvent } from "@/api/types";

function patchTree(nodes: FileNode[], event: FsEvent): FileNode[] {
  const parts = event.path.split("/");

  if (event.kind === "remove") {
    return removeFromTree(nodes, parts);
  }
  if (event.kind === "create" || event.kind === "modify") {
    return insertIntoTree(nodes, parts, event.path);
  }
  return nodes;
}

function removeFromTree(nodes: FileNode[], parts: string[]): FileNode[] {
  if (parts.length === 1) {
    return nodes.filter((n) => n.name !== parts[0]);
  }
  return nodes.map((n) => {
    if (n.type === "dir" && n.name === parts[0] && n.children) {
      return { ...n, children: removeFromTree(n.children, parts.slice(1)) };
    }
    return n;
  });
}

function insertIntoTree(
  nodes: FileNode[],
  parts: string[],
  fullPath: string
): FileNode[] {
  if (parts.length === 1) {
    if (nodes.some((n) => n.name === parts[0])) return nodes;
    const newNode: FileNode = {
      name: parts[0],
      path: fullPath,
      type: "file",
    };
    return sortNodes([...nodes, newNode]);
  }

  const dirName = parts[0];
  let found = false;
  const result = nodes.map((n) => {
    if (n.type === "dir" && n.name === dirName) {
      found = true;
      return {
        ...n,
        children: insertIntoTree(n.children ?? [], parts.slice(1), fullPath),
      };
    }
    return n;
  });

  if (!found) {
    // Compute the correct directory path based on depth, not indexOf.
    // fullPath has N segments; parts has M remaining. The directory we're
    // creating is at depth (N - M + 1) from the root.
    const allSegments = fullPath.split("/");
    const depth = allSegments.length - parts.length + 1;
    const dirPath = allSegments.slice(0, depth).join("/");

    result.push({
      name: dirName,
      path: dirPath,
      type: "dir",
      children: insertIntoTree([], parts.slice(1), fullPath),
    });
    return sortNodes(result);
  }
  return result;
}

function sortNodes(nodes: FileNode[]): FileNode[] {
  return [...nodes].sort((a, b) => {
    if (a.type !== b.type) return a.type === "dir" ? -1 : 1;
    return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
  });
}

export function useFileTree(onModifyEvent?: (path: string) => void) {
  const queryClient = useQueryClient();
  const socketRef = useRef<ManagedSocket | null>(null);
  const onModifyRef = useRef(onModifyEvent);
  onModifyRef.current = onModifyEvent;

  const query = useQuery({
    queryKey: ["tree"],
    queryFn: getTree,
    staleTime: Infinity,
  });

  useEffect(() => {
    const socket = connectWatch((event) => {
      queryClient.setQueryData<FileNode[]>(["tree"], (old) => {
        if (!old) return old;
        return patchTree(old, event as FsEvent);
      });

      if (event.kind === "modify" && onModifyRef.current) {
        onModifyRef.current(event.path);
      }
    });
    socketRef.current = socket;

    return () => {
      socket.close();
      socketRef.current = null;
    };
  }, [queryClient]);

  return query;
}
