---
number: 0002
date: 2026-10-03
status: accepted
deciders: [Bader]
tags: [decision]
---

# 0002: The UI owns the open/close animation, Rust only sends messages

## Context
We want Newtron to feel like it grows out of its search field and shrinks back, not like a window popping in and out. The native window is controlled from Rust, but the animation lives in the UI (CSS). If Rust hides the window the moment you press Alt+N, the close animation never gets to play, because the window is already gone.

## Options considered
1. **Rust hides after a delay.** Easy, but it needs a hardcoded wait that has to match the CSS by hand, and it breaks if either side changes or the user toggles quickly.
2. **The UI owns the animation and tells Rust when it's done.** Rust sends messages (`show`, `toggle`, `hide`), the UI animates, then the UI hides the window itself when the close animation has finished.
3. **Native OS window animation.** Not stylable, and we can't make it grow out of a search field.

## Decision
Option 2. Rust never hides the window on its own for Alt+N, Esc, click-away, or the close button. It sends an event, and the UI decides what to do. The close timing comes from a single CSS variable (`--close-ms`) that the UI reads.

## Consequences
- **Good:** the close animation always plays, timing lives in one place, reduced motion can shorten it, and a quick Alt+N during a close can reopen cleanly because the UI knows its own state.
- **Bad:** more moving parts (three events, a timer, a state flag), and anything that wants to hide the window must go through the UI or the animation gets skipped.
- **Revisit when:** we add more windows, or if the delay between "close requested" and "window hidden" ever feels laggy.