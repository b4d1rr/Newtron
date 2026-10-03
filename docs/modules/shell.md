---
module: shell
owner: Bader
status: in progress
last_updated: 2026-10-03
tags: [module]
---

# Shell (window, tray, shortcut, bar UI)

> Living doc: update this whenever the module's behavior changes. History goes in the devlog, not here.

## Purpose
The outer layer of Newtron: the tray icon, the global shortcut, the frameless window, and the command bar UI with its open/close animation. Search and AI plug into the dropdown under the bar.

## How it works now

**Files**
- `src-tauri/src/lib.rs`: tray, shortcut, window events, messages to the UI
- `src-tauri/tauri.conf.json`: window settings (frameless, transparent, starts hidden, 820x500, no shadow, not resizable)
- `src-tauri/capabilities/default.json`: permissions the UI is allowed (includes `core:window:allow-hide`)
- `src/App.tsx`: the bar, the dropdown, and the open/close state
- `src/App.css`: all styling and the animation choreography

**Showing and hiding**
1. Alt+N pressed. If the window is hidden, Rust shows and focuses it, then sends `newtron:show`. If it's visible, Rust sends `newtron:toggle`.
2. Window loses focus or the window's close button is used: Rust sends `newtron:hide` (and cancels the real close).
3. The UI handles these in `present()` and `dismiss()`. `dismiss()` flips `data-shown` to `false`, waits for the close animation (`--close-ms`), then hides the native window.
4. Esc clears the text first, then calls `dismiss()`.
5. Tray *Show Newtron* calls the same show path as Alt+N. *Quit Newtron* exits the app.

**Animation**
- The only switch is `data-shown="true|false"` on `<main class="stage">`.
- Each element has a closed look and an open look. Transitions use the timing of the state they head into, so opening and closing have separate choreography.
- Open (~240ms): the search field grows from a small circle (clip-path reveal driven by the `--reveal` number), the text settles in, the round button pops out, the dropdown opens only when there's a query.
- Close (~190ms): the dropdown folds up, the button and text drop out, the field contracts to the circle, then the window fades.
- Reduced motion: no movement, 120ms fade only.

## Public interface
- **Events (Rust to UI):** `newtron:show`, `newtron:toggle`, `newtron:hide`
- **CSS variables to tune:** `--open-ms`, `--close-ms`, `--ease-out`, `--ease-in`, `--row` (row height, other CSS depends on it)
- **Constants in `App.tsx`:** `SEARCH_URL` (web search engine)
- **Shortcut:** Alt+N (either Alt key), registered in `lib.rs`

## Data
None. The shell stores nothing yet.

## How to test
See the steps in `docs/devlog/2026-10-03-bader-tray-shortcut-and-bar-ui.md`. Quick version: Alt+N opens, Alt+N / Esc / click-away closes, mash Alt+N and make sure nothing sticks.

## Known limits / TODO
- Right-Alt-only is not possible with the current shortcut method (see decision 0002-1).
- Transparent margin around the bar still blocks clicks underneath. Fix: size the window to its content.
- Window position is just "center". Should sit near the top-center like Spotlight, and follow the active monitor.
- Not always-on-top yet.
- Dropdown content is placeholder, Enter only does a web search.
- Enter should later open the selected result, with web search on Shift+Enter.
- Possible first-show frame drop on Windows.

## Related
- Decisions: 0002 (UI owns the window animation), 0002-1 (Alt+N via the global-shortcut plugin)
- Devlog: `0001 26-10-03 bader scaffold and setup`, `0002 26-10-03 bader tray shortcut and bar ui`