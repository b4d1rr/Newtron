import type { SearchResult, WebSearchStatus } from "../lib/types";
import { Favicon } from "./Favicon";

export function WebResults({
  status,
  results,
  error,
  selectedIndex,
  onOpen,
  onHover,
  onRetry,
}: {
  status: WebSearchStatus;
  results: SearchResult[];
  error: string | null;
  selectedIndex: number;
  onOpen: (result: SearchResult) => void;
  onHover: (index: number) => void;
  onRetry: () => void;
}) {
  if (status === "loading") {
    return (
      <div className="web-state">
        <div className="spinner" aria-label="Searching" />
        <span>Searching…</span>
      </div>
    );
  }

  if (status === "error") {
    return (
      <div className="web-state">
        <span className="web-state-error">Search failed{error ? ` — ${error}` : ""}</span>
        <button className="retry-btn" onClick={onRetry}>
          Retry
        </button>
      </div>
    );
  }

  if (status === "done" && results.length === 0) {
    return (
      <div className="web-state">
        <span>No results found</span>
      </div>
    );
  }

  if (results.length === 0) {
    return null;
  }

  return (
    <div role="listbox" aria-label="Web results">
      <div className="section-title">
        {results[0].source} Results
        <span className="section-hint">↵ open · shift+↵ search in browser</span>
      </div>
      {results.map((r, i) => (
        <div
          key={r.url + i}
          className={`web-result-row ${i === selectedIndex ? "selected" : ""}`}
          onClick={() => onOpen(r)}
          onMouseMove={() => onHover(i)}
          role="option"
          aria-selected={i === selectedIndex}
        >
          <Favicon src={r.favicon} label={r.title} />
          <div className="web-result-body">
            <div className="web-result-title">{r.title}</div>
            {r.snippet && <div className="web-result-snippet">{r.snippet}</div>}
            <div className="web-result-url">{r.url}</div>
          </div>
          {i === selectedIndex && <kbd className="key-hint">↵</kbd>}
        </div>
      ))}
    </div>
  );
}
