---
author: Bader
date: 2026-10-03
module: shell
commit: 4369bc1
pr: n/a (pushed directly to main so the team can start from it)
tags:
  - devlog
---

# Tray app, Alt+N shortcut, Spotlight-style bar, and the open/close animation

## What I did
Newtron now behaves like a real command bar. It lives in the system tray, starts hidden, and Alt+N brings up a dark, minimal search bar that grows out of its search field. It closes with Alt+N, Esc, or clicking anywhere else. Typing opens a dropdown (placeholder results), and a round globe button (or Enter) searches the web for the query in the default browser.

## Why
This is the shell everything else plugs into: search, app launching, and local AI all appear inside this bar, so it had to feel right first. A tray app with a global shortcut is how Spotlight-style tools work, and the animation makes it feel like one object instead of a window popping up.

## How it works
- **Tray:** a tray icon with *Show Newtron* and *Quit Newtron*. Closing the window only hides it, so the app keeps running until you quit from the tray.
- **Window:** frameless and transparent (`tauri.conf.json`), starts hidden. While it's visible it shows on the taskbar.
- **Alt+N:** registered from Rust with the global-shortcut plugin. If the window is hidden it shows it, otherwise it asks the UI to close.
- **Click-away and Esc:** when the window loses focus, Rust tells the UI to close. Esc clears the text first, then closes.
- **Animation:** Rust never hides the window directly anymore. It sends the UI an event (`newtron:show`, `newtron:toggle`, `newtron:hide`), the UI flips one switch (`data-shown`), CSS plays the open or close animation, and the UI hides the native window when the close animation is done. Details are in `docs/modules/shell.md` and `docs/decisions/0002-ui-owns-window-animation.md`.
- **Web search:** the button and Enter open `https://www.google.com/search?q=...` through the opener plugin, then close the bar.
- **Everything in the dropdown is placeholder.**

## How to test it
1. `npm run tauri dev` and find the tray icon (check under the ^ arrow).
2. Press **Alt+N**: the bar should grow out of the small search circle.
3. Type a word: the dropdown slides open and the globe button lights up. Use ↑/↓ to move the highlight.
4. Press **Enter** (or click the globe): your browser opens a Google search and the bar closes.
5. Press Alt+N again, then Esc once (clears the text) and Esc again (closes).
6. Open it, then click another window: it should close.
7. Mash Alt+N a few times quickly: it should never get stuck half-open.
8. Tray menu: *Show Newtron* and *Quit Newtron* both work.

## What I'd explain to a teammate
- **Why Rust asks the UI instead of hiding the window:** if Rust hides the window right away, the close animation never gets to play. So the UI plays the animation first and then hides the window itself.
- **Why one `data-shown` switch:** CSS transitions use the timing of the state they're heading into, so opening and closing can have different choreography, and reversing halfway just continues from where it is.
- **Why `clip-path` for the field opening:** scaling the pill would squash its round ends. Clipping reveals it left to right and keeps the ends round.
- **Why Esc and click-away both route through the same close function:** one code path means one place for the animation and the hide timer, and no duplicate or stuck states.
- **Why the window is bigger than the bar:** the shadow needs room, so the window has transparent padding around the bar.
- **Why `core:window:allow-hide` is in the capabilities file:** Tauri blocks the UI from controlling the window unless we allow it.

## Things I'm unsure about
- Alt+N works with either Alt key. Right-Alt-only would need a low-level keyboard hook (see decision 0003).
- On some non-US keyboard layouts, Windows may treat right Alt as AltGr and Alt+N might not fire.
- The animation timings are a first pass and I haven't tuned them across other machines.
- The transparent margin around the bar still blocks clicks on whatever is underneath.
- Search results are placeholders, and web search is hardcoded to Google.
- Possible stutter on the very first show.

## Links
- Module doc: `docs/modules/shell.md`
- Decisions: `docs/decisions/0002-ui-owns-window-animation.md`, `docs/decisions/0003-alt-n-via-global-shortcut-plugin.md`