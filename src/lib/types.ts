/** Mirrors `SearchResult` in src-tauri/src/search/mod.rs */
export interface SearchResult {
  title: string;
  snippet: string;
  url: string;
  favicon?: string | null;
  source: string;
}

/** Mirrors `Suggestion` in src-tauri/src/index/mod.rs */
export interface Suggestion {
  url: string;
  domain: string;
  title?: string | null;
  favicon: string;
  source: "builtin" | "history" | "user" | "alias";
  score: number;
}

export interface SystemItem {
  name: string;
  kind: string;
  path: string;
}

export type WebSearchStatus = "idle" | "loading" | "done" | "error";
