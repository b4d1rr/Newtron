---
number: 0002-1
date: 2026-10-03
status: accepted
deciders: [Bader]
tags: [decision]
---

# 0003: Summon with Alt+N through the global-shortcut plugin (either Alt key)

## Context
Newtron needs a shortcut that works from anywhere on the machine. I wanted **right Alt + N** specifically. Windows' normal global-shortcut system can't tell left Alt from right Alt, so the shortcut it offers is "either Alt + N". On some non-US keyboard layouts, Windows also treats right Alt as AltGr (reported as Ctrl+Alt), which would make plain Alt+N not fire.

## Options considered
1. **Tauri's global-shortcut plugin (Alt+N, either Alt).** A few lines of Rust, well supported, and easy to explain. Can't be right-Alt-only.
2. **A low-level keyboard hook.** Can tell left and right Alt apart, but it's much more code, easier to get wrong, and it listens to every keypress system-wide, which has privacy and reliability costs.

## Decision
Option 1 for now. It matches what the README already promises (`Alt + N`) and keeps the code simple enough for everyone on the team to understand. If registering the shortcut fails (another app owns it), the app prints a message instead of crashing.

## Consequences
- **Good:** small, readable, and reliable on a normal setup.
- **Bad:** left Alt+N also summons Newtron, and Alt+N is swallowed from whatever app is focused. It may not fire on layouts where right Alt acts as AltGr.
- **Revisit when:** someone hits a real conflict, or the shortcut should become configurable in settings (that would be a good point to look at the hook again).