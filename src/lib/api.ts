/**
 * Typed facade over the Tauri command surface. Components never call
 * `invoke` directly — this is the single place command names and payload
 * shapes live on the frontend.
 */
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

import type { SearchResult, Suggestion, SystemItem } from "./types";

export function webSearch(query: string): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("web_search", { query });
}

export function urlSuggest(query: string, limit = 6): Promise<Suggestion[]> {
  return invoke<Suggestion[]>("url_suggest", { query, limit });
}

export function getSystemResults(query: string): Promise<SystemItem[]> {
  return invoke<SystemItem[]>("get_system_results", { query });
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

/** Build a search-engine URL for "open in browser" fallbacks. */
export function browserSearchUrl(query: string): string {
  return `https://duckduckgo.com/?q=${encodeURIComponent(query)}`;
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
