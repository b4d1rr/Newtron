/** Mirrors `SearchResult` in src-tauri/src/search/mod.rs (future APIProvider
 *  path — not used by the default web-search flow, kept for `webSearch()`). */
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

/** Mirrors `AppEntry` in src-tauri/src/index/local.rs */
export interface AppEntry {
  id: number;
  name: string;
  path: string;
  icon?: string | null;
  usage_count: number;
  last_opened?: number | null;
  score: number;
}

/** Mirrors `FileEntry` in src-tauri/src/index/local.rs */
export interface FileEntry {
  id: number;
  name: string;
  full_path: string;
  extension?: string | null;
  size: number;
  created_date: number;
  modified_date: number;
  last_accessed?: number | null;
  score: number;
}

/** Mirrors `RouteItem` in src-tauri/src/router.rs (internally tagged on `kind`). */
export type AppRouteItem = { kind: "app" } & AppEntry;
export type FileRouteItem = { kind: "file" } & FileEntry;
export type GoToRouteItem = { kind: "go_to" } & Suggestion;
export type WebSearchRouteItem = { kind: "web_search"; query: string };
export type RouteItem = AppRouteItem | FileRouteItem | GoToRouteItem | WebSearchRouteItem;

/** Mirrors `RouteResult` in src-tauri/src/router.rs */
export interface RouteResult {
  items: RouteItem[];
}

/** Mode 1 = search files and web, Mode 2 = ask AI (placeholder). */
export type Mode = "search" | "ai";

export type WebSearchStatus = "idle" | "loading" | "done" | "error";
