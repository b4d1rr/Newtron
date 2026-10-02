---
author: Bader
date: 2026-10-03
module: infra
commit: 258296f
pr: n/a (first commit of the rewrite, pushed straight to main)
tags: [devlog]
---

# Fresh Tauri + React scaffold, and the team setup around it

## What I did
Scrapped the old build and started over with a clean Tauri v2 + React + TypeScript project. Around it I set up everything the team needs to work on it: pinned tool versions, docs folder and templates, a contributing guide, a setup guide, a PR template, and cleaned-up git config. The README now matches the new, smaller scope.

## Why
We have about two months, so the old version was too big, and I'd leaned on generated code more than I wanted to. Starting clean means everything in the repo is something we chose and understand. The team setup is there so "works on my machine" doesn't happen: everyone gets the same Rust, the same Node, and the same dependency versions.

(The full reasoning for the restart is in `docs/decisions/0001-scope-reduction-and-restart.md`.)

## How it works
- Scaffolded with `create-tauri-app` (React + TypeScript template), then renamed from `newtron-tmp` to `newtron`.
- `npm run tauri dev` starts Vite (the UI dev server, port 1420) and builds and runs the Rust app, which opens a window showing the UI.
- Pinned versions: Rust `1.93.0` (`rust-toolchain.toml`), Node 24 (`.nvmrc` and `engines` in `package.json`). Lockfiles are committed.
- Docs live in `docs/` (templates in `docs/_templates/`), team rules in `CONTRIBUTING.md`, setup steps in `docs/setup.md`.

## How to test it
1. Clone the repo into a fresh folder.
2. `npm ci`
3. `npm run tauri dev`
4. A Newtron window should open. The first build takes a few minutes, later ones are fast.

## What I'd explain to a teammate
- **Two `src` folders:** root `src/` is the React UI, `src-tauri/src/` is the Rust backend.
- **`main.rs` vs `lib.rs`:** `main.rs` just calls `run()` from `lib.rs`, and `lib.rs` is where the real app code goes. When I renamed the project, I had to change the library name in `Cargo.toml` and the call in `main.rs`, otherwise the build broke with "unresolved crate".
- **Why `npm ci`:** it installs exactly what the lockfile says, instead of possibly updating versions, so we all get the same build.
- **Why the versions are pinned:** a different compiler or Node version can change how things build, and we don't want to debug that.
- **Why `tauri dev` restarts by itself:** it watches `src-tauri` for Rust changes and Vite hot-reloads the UI.

## Things I'm unsure about
- SQLite isn't added yet. That's the next step (`rusqlite` with the `bundled` feature, and an FTS5 test table).
- CI (automatic build checks on PRs) isn't set up yet.
- I assumed Windows-only for now.
- Haven't done the fresh-clone test on a second machine yet.

## Links
- Decision: `docs/decisions/0001-scope-reduction-and-restart.md`
- Setup: `docs/setup.md`
- Team guide: `CONTRIBUTING.md`

- Commit: [`258296f`](https://github.com/b4d1rr/Newtron/commit/258296f)