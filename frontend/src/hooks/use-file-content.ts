import { useQuery } from "@tanstack/react-query";
import { useCallback, useRef } from "react";
import { getFile, putFile } from "@/api/client";
import { useEditorStore } from "@/stores/editor-store";

export function useFileContent(path: string | null) {
  const setEtag = useEditorStore((s) => s.setEtag);
  const setDirty = useEditorStore((s) => s.setDirty);
  const openFiles = useEditorStore((s) => s.openFiles);
  const currentEtag = openFiles.find((f) => f.path === path)?.etag ?? undefined;

  const query = useQuery({
    queryKey: ["file", path],
    queryFn: async () => {
      if (!path) return null;
      const result = await getFile(path, currentEtag);
      if (result.notModified) return null;
      if (result.etag && path) {
        setEtag(path, result.etag);
      }
      return result.data;
    },
    enabled: !!path,
    staleTime: 30_000,
  });

  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const saveFile = useCallback(
    (content: string) => {
      if (!path) return;
      setDirty(path, true);

      if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
      saveTimerRef.current = setTimeout(async () => {
        try {
          await putFile(path, content);
          setDirty(path, false);
        } catch (err) {
          console.error("save failed:", err);
        }
      }, 150);
    },
    [path, setDirty]
  );

  return { ...query, saveFile };
}
