import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import "./App.css";

import { askNewtron, looksLikeUrl, openExternal, openLocalItem, openWebSearch, searchAll, toUrl } from "./lib/api";
import type { GoToRouteItem, Mode, RouteItem, RouteResult } from "./lib/types";
import { useDebouncedValue } from "./hooks/useDebouncedValue";
import { SuggestionRow } from "./components/SuggestionRow";
import { LocalResultRow } from "./components/LocalResultRow";
import { WebSearchRow } from "./components/WebSearchRow";

const appWindow = getCurrentWebviewWindow();

// Every backend query here is a local SQLite lookup (files/apps/URL index) —
// no network round-trip, so a short debounce is purely about not re-querying
// on every single keystroke while typing fast.
const SEARCH_DEBOUNCE_MS = 90;

const EMPTY_RESULT: RouteResult = { items: [] };

function App() {
  const [input, setInput] = useState("");
  const [mode, setMode] = useState<Mode>("search");
  const [aiResponse, setAiResponse] = useState("");
  const [aiLoading, setAiLoading] = useState(false);
  const [routeResult, setRouteResult] = useState<RouteResult>(EMPTY_RESULT);
  const [selectedIndex, setSelectedIndex] = useState(-1);
  // Bumped on every window focus so the entrance animation can replay by
  // remounting `.main-container` (see key={animKey} below).
  const [animKey, setAnimKey] = useState(0);

  const inputRef = useRef<HTMLInputElement>(null);
  const searchToken = useRef(0);
  const debouncedInput = useDebouncedValue(input, SEARCH_DEBOUNCE_MS);

  const resetAll = useCallback(() => {
    setInput("");
    setAiResponse("");
    setRouteResult(EMPTY_RESULT);
    setSelectedIndex(-1);
  }, []);

  const items = routeResult.items;

  /** Ghost-text completion: remainder of the top "go to" suggestion's domain. */
  const topGoTo = useMemo(() => items.find((i): i is GoToRouteItem => i.kind === "go_to"), [items]);
  const ghostRest = useMemo(() => {
    if (mode !== "search" || !input.trim() || !topGoTo) return "";
    const typed = input.trim().toLowerCase();
    const top = topGoTo.domain.toLowerCase();
    const bare = top.startsWith("www.") ? top.slice(4) : top;
    if (bare.startsWith(typed) && bare.length > typed.length) return bare.slice(typed.length);
    return "";
  }, [input, topGoTo, mode]);

  // Mode 1: local files + apps + go-to suggestions + web-search row, all in
  // one ranked backend call (see router.rs).
  useEffect(() => {
    if (mode !== "search") return;
    const q = debouncedInput.trim();
    const token = ++searchToken.current;
    if (!q || looksLikeUrl(q)) {
      setRouteResult(EMPTY_RESULT);
      return;
    }
    searchAll(q)
      .then((r) => {
        if (token === searchToken.current) setRouteResult(r);
      })
      .catch(() => {
        if (token === searchToken.current) setRouteResult(EMPTY_RESULT);
      });
  }, [debouncedInput, mode]);

  useEffect(() => {
    setSelectedIndex(-1);
  }, [items]);

  const openItem = useCallback((item: RouteItem) => {
    if (item.kind === "app") {
      openLocalItem(item.path, "app");
    } else if (item.kind === "file") {
      openLocalItem(item.full_path, "file");
    } else if (item.kind === "go_to") {
      openExternal(item.url, item.title ?? undefined);
    } else {
      openWebSearch(item.query);
    }
    appWindow.hide();
  }, []);

  /** Enter with nothing selected: URL goes straight to browser, ghost-text
   *  completion is accepted, otherwise fall through to a web search. */
  const handleEnterDefault = useCallback(() => {
    const q = input.trim();
    if (!q) return;
    if (looksLikeUrl(q)) {
      openExternal(toUrl(q));
      appWindow.hide();
      return;
    }
    if (ghostRest) {
      openExternal(toUrl(q + ghostRest));
      appWindow.hide();
      return;
    }
    openWebSearch(q);
    appWindow.hide();
  }, [input, ghostRest]);

  const askAi = useCallback(async (query: string) => {
    if (!query.trim()) {
      setAiResponse("");
      return;
    }
    setAiLoading(true);
    try {
      const res = await askNewtron(query);
      setAiResponse(res);
    } finally {
      setAiLoading(false);
    }
  }, []);

  // Reset to a fresh prompt whenever the window hides; refocus + replay the
  // entrance animation on show.
  useEffect(() => {
    const unlisten = appWindow.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        setAnimKey((k) => k + 1);
        inputRef.current?.focus();
      } else {
        resetAll();
      }
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, [resetAll]);

  // Global keyboard handling — the launcher is keyboard-first.
  useEffect(() => {
    inputRef.current?.focus();
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl/Cmd+1 and +2 switch modes without hijacking digits typed into
      // the query itself.
      if ((e.ctrlKey || e.metaKey) && (e.key === "1" || e.key === "2")) {
        e.preventDefault();
        setMode(e.key === "1" ? "search" : "ai");
        return;
      }

      // Ctrl+L: focus and select the input, like a browser address bar.
      if (e.ctrlKey && e.key.toLowerCase() === "l") {
        e.preventDefault();
        inputRef.current?.focus();
        inputRef.current?.select();
        return;
      }

      if (e.key === "Escape") {
        e.preventDefault();
        if (items.length > 0 || selectedIndex !== -1) {
          searchToken.current++;
          setRouteResult(EMPTY_RESULT);
          setSelectedIndex(-1);
        } else {
          appWindow.hide();
        }
        return;
      }

      if (mode === "search") {
        if (e.key === "ArrowDown") {
          e.preventDefault();
          if (items.length > 0) setSelectedIndex((i) => (i + 1) % items.length);
          return;
        }
        if (e.key === "ArrowUp") {
          e.preventDefault();
          if (items.length > 0) setSelectedIndex((i) => (i <= 0 ? items.length - 1 : i - 1));
          return;
        }
        if (e.key === "Tab" && ghostRest) {
          e.preventDefault();
          setInput(input.trim() + ghostRest);
          return;
        }
        if (e.key === "Enter") {
          e.preventDefault();
          if (e.shiftKey) {
            // Explicit "open in browser" for the raw query, regardless of selection.
            if (input.trim()) {
              openWebSearch(input.trim());
              appWindow.hide();
            }
            return;
          }
          if (selectedIndex >= 0 && selectedIndex < items.length) {
            openItem(items[selectedIndex]);
          } else {
            handleEnterDefault();
          }
          return;
        }
      } else if (e.key === "Enter" && !e.shiftKey) {
        askAi(input);
      }
    };
    window.addEventListener("keydown", handleKeyDown, true);
    return () => window.removeEventListener("keydown", handleKeyDown, true);
  }, [input, mode, items, ghostRest, selectedIndex, openItem, handleEnterDefault, askAi]);

  const showPanel = mode === "ai" ? Boolean(aiResponse) || aiLoading : items.length > 0;

  const localItems = items.filter((i): i is Extract<RouteItem, { kind: "app" | "file" }> => i.kind === "app" || i.kind === "file");
  const goToItems = items.filter((i): i is GoToRouteItem => i.kind === "go_to");
  const webSearchItem = items.find((i): i is Extract<RouteItem, { kind: "web_search" }> => i.kind === "web_search");

  return (
    <div className="wrapper">
      <div className="main-container" key={animKey}>
        <div className="search-box">
          <span className="ai-icon">{mode === "ai" ? "✨" : "⌕"}</span>
          <div className="input-stack">
            {/* Ghost layer sits behind the input; the typed prefix is
                transparent so only the completion remainder is visible. */}
            <div className="ghost-layer" aria-hidden="true">
              <span className="ghost-typed">{input}</span>
              <span className="ghost-rest">{ghostRest}</span>
            </div>
            <input
              ref={inputRef}
              type="text"
              value={input}
              onChange={(e) => {
                setInput(e.target.value);
                if (mode === "ai" && !e.target.value.trim()) setAiResponse("");
              }}
              placeholder={mode === "ai" ? "Ask Newtron anything..." : "Search files, apps, or the web..."}
              className="search-input"
              spellCheck={false}
              autoComplete="off"
            />
          </div>
          {mode === "search" && ghostRest && <kbd className="key-hint tab-hint">Tab</kbd>}

          <div className="mode-switch" role="tablist" aria-label="Mode">
            <div className={`mode-indicator ${mode === "ai" ? "mode-indicator-ai" : ""}`} aria-hidden="true" />
            <button
              type="button"
              role="tab"
              aria-selected={mode === "search"}
              className={mode === "search" ? "active" : ""}
              onClick={() => setMode("search")}
            >
              <kbd>1</kbd> Search
            </button>
            <button
              type="button"
              role="tab"
              aria-selected={mode === "ai"}
              className={mode === "ai" ? "active" : ""}
              onClick={() => setMode("ai")}
            >
              <kbd>2</kbd> Ask AI
            </button>
          </div>
        </div>

        <div className={`results-wrapper ${showPanel ? "expanded" : ""}`}>
          <div className="results-inner">
            <div className="results-scroll">
              {mode === "ai" && (
                <div className="ai-result-block">
                  <div className="section-title">Newtron AI</div>
                  {aiLoading ? (
                    <div className="web-state">
                      <div className="spinner" aria-label="Thinking" />
                      <span>Thinking…</span>
                    </div>
                  ) : (
                    <div className="ai-content">{aiResponse}</div>
                  )}
                </div>
              )}

              {mode === "search" && localItems.length > 0 && (
                <div className="suggestions-block">
                  <div className="section-title">Files &amp; Apps</div>
                  {localItems.map((item) => {
                    const idx = items.indexOf(item);
                    return (
                      <LocalResultRow
                        key={`${item.kind}-${item.kind === "app" ? item.path : item.full_path}`}
                        item={item}
                        selected={idx === selectedIndex}
                        onOpen={() => openItem(item)}
                        onHover={() => setSelectedIndex(idx)}
                      />
                    );
                  })}
                </div>
              )}

              {mode === "search" && goToItems.length > 0 && (
                <div className="suggestions-block">
                  <div className="section-title">Go To</div>
                  {goToItems.map((s) => {
                    const idx = items.indexOf(s);
                    return (
                      <SuggestionRow
                        key={s.url}
                        suggestion={s}
                        selected={idx === selectedIndex}
                        onOpen={() => openItem(s)}
                        onHover={() => setSelectedIndex(idx)}
                      />
                    );
                  })}
                </div>
              )}

              {mode === "search" && webSearchItem && (
                <WebSearchRow
                  query={webSearchItem.query}
                  selected={items.indexOf(webSearchItem) === selectedIndex}
                  onOpen={() => openItem(webSearchItem)}
                  onHover={() => setSelectedIndex(items.indexOf(webSearchItem))}
                />
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
