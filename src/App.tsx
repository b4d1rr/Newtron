import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import "./App.css";

import {
  askNewtron,
  browserSearchUrl,
  getSystemResults,
  looksLikeUrl,
  openExternal,
  toUrl,
  urlSuggest,
  webSearch,
} from "./lib/api";
import type { SearchResult, Suggestion, SystemItem, WebSearchStatus } from "./lib/types";
import { useDebouncedValue } from "./hooks/useDebouncedValue";
import { SuggestionRow } from "./components/SuggestionRow";
import { WebResults } from "./components/WebResults";

const appWindow = getCurrentWebviewWindow();

const SUGGEST_DEBOUNCE_MS = 120;
const SEARCH_DEBOUNCE_MS = 450;
const MIN_SEARCH_CHARS = 3;
const MAX_SUGGESTIONS = 5;

type Tab = "files" | "ai" | "web";

function App() {
  const [input, setInput] = useState("");
  const [activeTab, setActiveTab] = useState<Tab>("web");
  const [aiResponse, setAiResponse] = useState("");
  const [systemResults, setSystemResults] = useState<SystemItem[]>([]);

  // Web tab state
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);
  const [webResults, setWebResults] = useState<SearchResult[]>([]);
  const [webStatus, setWebStatus] = useState<WebSearchStatus>("idle");
  const [webError, setWebError] = useState<string | null>(null);
  const [selectedIndex, setSelectedIndex] = useState(-1);

  const inputRef = useRef<HTMLInputElement>(null);
  // Monotonic tokens discard stale async responses (fast typing races).
  const suggestToken = useRef(0);
  const searchToken = useRef(0);

  const debouncedSuggestInput = useDebouncedValue(input, SUGGEST_DEBOUNCE_MS);
  const debouncedSearchInput = useDebouncedValue(input, SEARCH_DEBOUNCE_MS);

  const resetAll = useCallback(() => {
    setInput("");
    setAiResponse("");
    setSystemResults([]);
    setSuggestions([]);
    setWebResults([]);
    setWebStatus("idle");
    setWebError(null);
    setSelectedIndex(-1);
  }, []);

  /** Ghost-text completion: remainder of the top suggestion's domain. */
  const ghostRest = useMemo(() => {
    if (activeTab !== "web" || !input.trim() || suggestions.length === 0) return "";
    const typed = input.trim().toLowerCase();
    const top = suggestions[0].domain.toLowerCase();
    const bare = top.startsWith("www.") ? top.slice(4) : top;
    if (bare.startsWith(typed) && bare.length > typed.length) return bare.slice(typed.length);
    return "";
  }, [input, suggestions, activeTab]);

  /** Combined keyboard-navigable list: suggestions first, then web results. */
  const navLength = suggestions.length + webResults.length;

  const runWebSearch = useCallback(async (query: string) => {
    const q = query.trim();
    if (!q) return;
    const token = ++searchToken.current;
    setWebStatus("loading");
    setWebError(null);
    try {
      const results = await webSearch(q);
      if (token !== searchToken.current) return;
      setWebResults(results);
      setWebStatus("done");
    } catch (e) {
      if (token !== searchToken.current) return;
      setWebResults([]);
      setWebStatus("error");
      setWebError(String(e));
    }
  }, []);

  // Live URL suggestions while typing in the web tab.
  useEffect(() => {
    if (activeTab !== "web") return;
    const q = debouncedSuggestInput.trim();
    const token = ++suggestToken.current;
    if (!q) {
      setSuggestions([]);
      return;
    }
    urlSuggest(q, MAX_SUGGESTIONS)
      .then((s) => {
        if (token === suggestToken.current) setSuggestions(s);
      })
      .catch(() => {});
  }, [debouncedSuggestInput, activeTab]);

  // Debounced auto web search.
  useEffect(() => {
    if (activeTab !== "web") return;
    const q = debouncedSearchInput.trim();
    if (q.length < MIN_SEARCH_CHARS || looksLikeUrl(q)) return;
    runWebSearch(q);
  }, [debouncedSearchInput, activeTab, runWebSearch]);

  // Selection resets whenever the underlying list changes.
  useEffect(() => {
    setSelectedIndex(-1);
  }, [suggestions, webResults]);

  const openSuggestion = useCallback((s: Suggestion) => {
    openExternal(s.url, s.title ?? undefined);
    appWindow.hide();
  }, []);

  const openResult = useCallback((r: SearchResult) => {
    openExternal(r.url, r.title);
    appWindow.hide();
  }, []);

  /** Enter with nothing selected: URL goes straight to browser, text searches. */
  const handleEnterDefault = useCallback(() => {
    const q = input.trim();
    if (!q) return;
    if (looksLikeUrl(q)) {
      openExternal(toUrl(q));
      appWindow.hide();
      return;
    }
    runWebSearch(q);
  }, [input, runWebSearch]);

  const handleFilesAndAi = useCallback(
    async (query: string) => {
      if (!query.trim()) {
        setAiResponse("");
        setSystemResults([]);
        return;
      }
      if (activeTab === "ai") {
        const res = await askNewtron(query);
        setAiResponse(res);
        setSystemResults([]);
      } else if (activeTab === "files") {
        const results = await getSystemResults(query);
        setSystemResults(results);
        setAiResponse("");
      }
    },
    [activeTab],
  );

  // Reset to a fresh prompt whenever the window hides; refocus on show.
  useEffect(() => {
    const unlisten = appWindow.onFocusChanged(({ payload: focused }) => {
      if (focused) {
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
      // Ctrl+L: focus and select the input, like a browser address bar.
      if (e.ctrlKey && e.key.toLowerCase() === "l") {
        e.preventDefault();
        inputRef.current?.focus();
        inputRef.current?.select();
        return;
      }

      if (e.key === "Escape") {
        e.preventDefault();
        // First Esc closes the results panel, second hides the window.
        if (webResults.length > 0 || webStatus !== "idle" || suggestions.length > 0) {
          searchToken.current++; // cancel in-flight search
          setWebResults([]);
          setSuggestions([]);
          setWebStatus("idle");
          setSelectedIndex(-1);
        } else {
          appWindow.hide();
        }
        return;
      }

      if (activeTab === "web") {
        if (e.key === "ArrowDown") {
          e.preventDefault();
          if (navLength > 0) setSelectedIndex((i) => (i + 1) % navLength);
          return;
        }
        if (e.key === "ArrowUp") {
          e.preventDefault();
          if (navLength > 0) setSelectedIndex((i) => (i <= 0 ? navLength - 1 : i - 1));
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
            // Explicit "open in browser" for the raw query.
            openExternal(browserSearchUrl(input.trim()));
            appWindow.hide();
            return;
          }
          if (selectedIndex >= 0 && selectedIndex < suggestions.length) {
            openSuggestion(suggestions[selectedIndex]);
          } else if (selectedIndex >= suggestions.length && selectedIndex < navLength) {
            openResult(webResults[selectedIndex - suggestions.length]);
          } else if (ghostRest) {
            openExternal(toUrl(input.trim() + ghostRest));
            appWindow.hide();
          } else {
            handleEnterDefault();
          }
          return;
        }
      } else if (e.key === "Enter" && !e.shiftKey) {
        handleFilesAndAi(input);
      }
    };
    window.addEventListener("keydown", handleKeyDown, true);
    return () => window.removeEventListener("keydown", handleKeyDown, true);
  }, [
    input,
    activeTab,
    navLength,
    ghostRest,
    selectedIndex,
    suggestions,
    webResults,
    webStatus,
    openSuggestion,
    openResult,
    handleEnterDefault,
    handleFilesAndAi,
  ]);

  const showPanel =
    Boolean(aiResponse) ||
    systemResults.length > 0 ||
    (activeTab === "web" && (suggestions.length > 0 || webResults.length > 0 || webStatus !== "idle"));

  return (
    <div className="wrapper">
      <div className="main-container">
        <div className="search-box">
          <span className="ai-icon">✨</span>
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
                if (activeTab !== "web") handleFilesAndAi(e.target.value);
              }}
              placeholder="Search or ask Newtron..."
              className="search-input"
              spellCheck={false}
              autoComplete="off"
            />
          </div>
          {activeTab === "web" && ghostRest && <kbd className="key-hint tab-hint">Tab</kbd>}
        </div>

        <div className="options-container">
          <button className={activeTab === "files" ? "active" : ""} onClick={() => setActiveTab("files")}>
            1 Search Files
          </button>
          <button className={activeTab === "ai" ? "active" : ""} onClick={() => setActiveTab("ai")}>
            2 Ask AI
          </button>
          <button
            className={activeTab === "web" ? "active" : ""}
            onClick={() => {
              setActiveTab("web");
              if (input.trim()) runWebSearch(input);
            }}
          >
            3 Search Web
          </button>
        </div>

        <div className={`results-wrapper ${showPanel ? "expanded" : ""}`}>
          <div className="results-inner">
            <div className="results-scroll">
              {aiResponse && (
                <div className="ai-result-block">
                  <div className="section-title">Newtron AI</div>
                  <div className="ai-content">{aiResponse}</div>
                </div>
              )}

              {systemResults.length > 0 && (
                <div className="system-results-block">
                  <div className="section-title">System Results</div>
                  {systemResults.map((item, idx) => (
                    <div key={idx} className="system-row">
                      <div className="item-icon-box">{item.kind[0]}</div>
                      <div className="item-details">
                        <div className="item-name">{item.name}</div>
                        <div className="item-path">{item.path}</div>
                      </div>
                      <div className="item-tag">{item.kind}</div>
                    </div>
                  ))}
                </div>
              )}

              {activeTab === "web" && suggestions.length > 0 && (
                <div className="suggestions-block">
                  <div className="section-title">Go To</div>
                  {suggestions.map((s, i) => (
                    <SuggestionRow
                      key={s.url}
                      suggestion={s}
                      selected={i === selectedIndex}
                      onOpen={() => openSuggestion(s)}
                      onHover={() => setSelectedIndex(i)}
                    />
                  ))}
                </div>
              )}

              {activeTab === "web" && (
                <WebResults
                  status={webStatus}
                  results={webResults}
                  error={webError}
                  selectedIndex={selectedIndex - suggestions.length}
                  onOpen={openResult}
                  onHover={(i) => setSelectedIndex(i + suggestions.length)}
                  onRetry={() => runWebSearch(input)}
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
