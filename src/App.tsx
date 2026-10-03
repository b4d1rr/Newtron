import { useCallback, useEffect, useRef, useState } from "react";
import type {
  CSSProperties,
  KeyboardEvent as ReactKeyboardEvent,
  ReactElement,
} from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import "./App.css";

// Swap this to change which search engine the button uses.
const SEARCH_URL = "https://www.google.com/search?q=";

type IconName = "search" | "apps" | "files" | "ai" | "globe";

// Simple line icons, drawn inline so we don't need an icon library.
const paths: Record<IconName, ReactElement> = {
  search: (
    <>
      <circle cx="11" cy="11" r="7" />
      <path d="m20 20-3.5-3.5" />
    </>
  ),
  apps: (
    <>
      <rect x="4" y="4" width="6.5" height="6.5" rx="1.5" />
      <rect x="13.5" y="4" width="6.5" height="6.5" rx="1.5" />
      <rect x="4" y="13.5" width="6.5" height="6.5" rx="1.5" />
      <rect x="13.5" y="13.5" width="6.5" height="6.5" rx="1.5" />
    </>
  ),
  files: (
    <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7z" />
  ),
  ai: <path d="M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8L12 3z" />,
  globe: (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="M3 12h18" />
      <path d="M12 3a14 14 0 0 1 0 18a14 14 0 0 1 0-18z" />
    </>
  ),
};

function Icon({ name }: { name: IconName }) {
  return (
    <svg
      className="icon"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      {paths[name]}
    </svg>
  );
}

// PLACEHOLDERS: none of this is real data yet.
const placeholderResults: { icon: IconName; title: string; sub: string }[] = [
  { icon: "apps", title: "Placeholder App", sub: "Application" },
  { icon: "files", title: "placeholder-document.pdf", sub: "Documents" },
  { icon: "files", title: "Placeholder Folder", sub: "Folder" },
  { icon: "ai", title: "Ask local AI", sub: "Coming soon" },
];

// How long the close animation runs. Read from the CSS (--close-ms) so the
// timing lives in one place, and reduced-motion can shorten it automatically.
const closeMs = () =>
  parseFloat(
    getComputedStyle(document.documentElement).getPropertyValue("--close-ms"),
  ) || 0;

export default function App() {
  const [query, setQuery] = useState("");
  const [sel, setSel] = useState(0); // which result row is highlighted
  const [shown, setShown] = useState(false); // drives every open/close animation

  // Refs hold values that event handlers need to read without going stale.
  const shownRef = useRef(false);
  const hideTimer = useRef<number | undefined>(undefined);
  const inputRef = useRef<HTMLInputElement>(null);

  const text = query.trim();
  const open = text.length > 0;
  const count = placeholderResults.length;

  // Play the open animation. Safe to call mid-close: it cancels the pending hide,
  // and the CSS transitions pick up from wherever they currently are.
  const present = useCallback(() => {
    window.clearTimeout(hideTimer.current);
    hideTimer.current = undefined;
    if (!shownRef.current) {
      setQuery("");
      setSel(0);
    }
    shownRef.current = true;
    setShown(true);
    inputRef.current?.focus(); // caret is ready right away
  }, []);

  // Play the close animation, then hide the native window once it has finished.
  const dismiss = useCallback(() => {
    if (!shownRef.current) return; // already closing or hidden
    shownRef.current = false;
    setShown(false);
    hideTimer.current = window.setTimeout(() => {
      hideTimer.current = undefined;
      getCurrentWindow().hide();
    }, closeMs() + 20);
  }, []);

  // Rust tells us when to show, toggle, or hide.
  useEffect(() => {
    const subs = Promise.all([
      listen("newtron:show", present),
      listen("newtron:toggle", () => (shownRef.current ? dismiss() : present())),
      listen("newtron:hide", dismiss),
    ]);
    return () => {
      subs.then((fns) => fns.forEach((off) => off()));
    };
  }, [present, dismiss]);

  // During development the window can already be up when the page loads.
  useEffect(() => {
    if (document.hasFocus()) present();
  }, [present]);

  // Open the query as a web search in the default browser, then close the bar.
  const webSearch = async () => {
    if (!text) return;
    try {
      await openUrl(SEARCH_URL + encodeURIComponent(text));
      dismiss();
    } catch (err) {
      console.error("Web search failed:", err);
    }
  };

  const onKeyDown = (e: ReactKeyboardEvent) => {
    switch (e.key) {
      case "Escape":
        // Clear the text first, then close.
        if (query) setQuery("");
        else dismiss();
        break;
      case "Enter":
        webSearch();
        break;
      case "ArrowDown":
        e.preventDefault(); // stops the text cursor from jumping
        setSel((s) => (s + 1) % count);
        break;
      case "ArrowUp":
        e.preventDefault();
        setSel((s) => (s - 1 + count) % count);
        break;
    }
  };

  return (
    <main className="stage" data-shown={shown} onKeyDown={onKeyDown}>
      <div className="row">
        <div className="bar">
          {/* Soft edge shadow, kept separate so it isn't clipped by the field. */}
          <div className="bar__halo" aria-hidden="true" />
          <label className="bar__field">
            <Icon name="search" />
            <input
              ref={inputRef}
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                setSel(0);
              }}
              placeholder="Search Newtron"
              spellCheck={false}
            />
          </label>
        </div>

        <div className="go-wrap">
          <button
            className="go"
            disabled={!open}
            title="Search the web"
            aria-label="Search the web"
            onClick={webSearch}
          >
            <Icon name="globe" />
          </button>
        </div>
      </div>

      <section className={`results ${open ? "results--open" : ""}`}>
        <div className="results__clip">
          <div
            className="results__card"
            style={{ "--sel": sel } as CSSProperties}
          >
            {/* One highlight that slides between rows instead of each row lighting up. */}
            <div className="highlight" />
            {placeholderResults.map((r, i) => (
              <div
                key={r.title}
                className="item"
                style={{ "--i": i } as CSSProperties}
                onMouseEnter={() => setSel(i)}
              >
                <span className="item__icon">
                  <Icon name={r.icon} />
                </span>
                <span className="item__text">
                  <span className="item__title">{r.title}</span>
                  <span className="item__sub">{r.sub}</span>
                </span>
              </div>
            ))}
          </div>
        </div>
      </section>
    </main>
  );
}