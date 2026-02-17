import { invoke } from "@tauri-apps/api/core";
import { useState, useCallback, useRef } from "react";

export function useModels() {
  const [models, setModels] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const lastFetched = useRef<{ url: string; key: string } | null>(null);

  const fetchModels = useCallback(async (url: string, apiKey: string) => {
    if (!url.trim() || !apiKey.trim()) {
      setModels([]);
      setError(null);
      lastFetched.current = null;
      return;
    }

    // Skip if same url+key already fetched
    if (
      lastFetched.current &&
      lastFetched.current.url === url &&
      lastFetched.current.key === apiKey
    ) {
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const result = await invoke<string[]>("fetch_models", {
        url,
        apiKey,
      });
      setModels(result);
      lastFetched.current = { url, key: apiKey };
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      setModels([]);
      lastFetched.current = null;
    } finally {
      setLoading(false);
    }
  }, []);

  const clearModels = useCallback(() => {
    setModels([]);
    setError(null);
    lastFetched.current = null;
  }, []);

  return { models, loading, error, fetchModels, clearModels };
}
