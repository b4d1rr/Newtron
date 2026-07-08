/**
 * Typed facade over the Tauri command surface. Components never call
 * `invoke` directly — this is the single place command names and payload
 * shapes live on the frontend.
 */
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

import type { RouteResult, SearchResult, Suggestion } from "./types";

/** Mode 1: local files + apps + go-to suggestions + web-search affordance,
 *  merged and ranked server-side (see `router.rs`). */
export function searchAll(query: string): Promise<RouteResult> {
  return invoke<RouteResult>("search_all", { query });
}

/** WebSearch::DefaultBrowserFallback — hands the query to the OS default
 *  browser instead of fetching results in-process. */
export function openWebSearch(query: string): Promise<void> {
  return invoke("open_web_search", { query });
}

/** Open a locally-indexed file or app with the OS default handler. */
export function openLocalItem(path: string, kind: "app" | "file"): Promise<void> {
  return invoke("open_local_item", { path, kind });
}

export function addIndexedFolder(path: string): Promise<void> {
  return invoke("add_indexed_folder", { path });
}

export function removeIndexedFolder(path: string): Promise<void> {
  return invoke("remove_indexed_folder", { path });
}

export function listIndexedFolders(): Promise<string[]> {
  return invoke<string[]>("list_indexed_folders");
}

/** Embedded provider-chain web search (future `APIProvider` path) — not
 *  called by the default UI flow; kept reachable for when it's re-enabled. */
export function webSearch(query: string): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("web_search", { query });
}

export function urlSuggest(query: string, limit = 6): Promise<Suggestion[]> {
  return invoke<Suggestion[]>("url_suggest", { query, limit });
}

export function askNewtron(message: string): Promise<string> {
  return invoke<string>("ask_newtron", { message });
}

/**
 * Open a URL in the default browser, recording the visit first so the
 * adaptive index learns from it. Fire-and-forget on the recording side.
 */
export async function openExternal(url: string, title?: string): Promise<void> {
  invoke("record_visit", { url, title: title ?? null }).catch(() => {});
  await openUrl(url);
}

/** True when the text already looks like a URL or bare domain. */
export function looksLikeUrl(text: string): boolean {
  const t = text.trim();
  return /^https?:\/\/\S+$/i.test(t) || /^[\w-]+(\.[\w-]+)+(:\d+)?(\/\S*)?$/.test(t);
}

/** Normalize typed text into an openable URL (assumes looksLikeUrl passed). */
export function toUrl(text: string): string {
  const t = text.trim();
  return /^https?:\/\//i.test(t) ? t : `https://${t}`;
}
