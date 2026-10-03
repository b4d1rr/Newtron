---
date: 2026-10-03
author: Bader
tags: [tasks]
---

# First tasks (Oct 3)

Goal for today is small: **everyone has Newtron running on their own machine and has merged one real PR.** Nothing here should take more than a few hours. If you get stuck, post in the group chat with the exact error, don't grind alone.

Before you start, read `docs/setup.md` and `CONTRIBUTING.md`.

---

## Everyone: get it running (do this first)

1. Follow `docs/setup.md` to install everything.
2. Clone the repo, then:
   ```powershell
   npm ci
   npm run tauri dev
   ```
   The first build takes a few minutes. That's normal.
3. Try it:
   - Nothing pops up at first, that's on purpose. Look for the Newtron icon in the system tray (it may be hiding under the ^ arrow near the clock).
   - Press **Alt + N**. The bar should grow out of the search field.
   - Type something. A dropdown slides open (placeholder results).
   - Press **Esc** to close, and try right-clicking the tray icon for **Show Newtron** / **Quit Newtron**.
4. Make a branch, add a devlog entry saying what happened (`docs/devlog/0000-YYMMDD-NAME-TOPIC.md`, copy `docs/_templates/devlog.md`). Write down anything in the setup guide that was wrong or confusing.
5. Open a PR. Bader reviews.

**Done when:** your setup PR is merged.

If Alt+N does nothing for you, tell the group chat. Another app might be using that shortcut, and the terminal will say `Could not register Alt+N`.

---

## Mohammed (backend, Rust): SQLite hello world

**Goal:** prove Rust can talk to SQLite, with full-text search (FTS5) working, in a way you can fully explain. No UI, no file indexing yet.

**Steps**
1. From `src-tauri`: `cargo add rusqlite --features bundled`
2. Create `src-tauri/src/db.rs` and add `mod db;` to `lib.rs` (that one line is the only change to `lib.rs`).
3. In `db.rs`, write a function that opens an **in-memory** database and creates an FTS5 table, for example `files_fts(name, path)`.
4. Insert 3 fake rows, then query them with a prefix search (like `MATCH 'rep*'` should find `report.pdf`).
5. Write a `#[cfg(test)]` test that does all of that and checks the result. `cargo test` should pass.

**Questions to answer in your devlog (in your own words)**
- What does the `bundled` feature do, and why do we use it?
- What is FTS5, and how is it different from `WHERE name LIKE '%rep%'`?
- Why use an in-memory database for the test?

**Don't touch:** anything in `src/` or the window/tray code in `lib.rs`.
**You own:** `src-tauri/src/db.rs`.
**Stretch (only if you finish):** open a database file in the app's data folder instead of memory.

---

## Essa (frontend): split `App.tsx` into components

**Goal:** `App.tsx` is getting long. Pull two pieces out into their own files **without changing how anything looks or behaves**.

**Steps**
1. Move `Icon` (plus the `paths` object and the `IconName` type) into `src/components/Icon.tsx`.
2. Move one result row into `src/components/ResultRow.tsx`. It should take props for the icon, title, and subtitle, and the row's index (used for the stagger animation).
3. Import both in `App.tsx`.
4. Run it and check the animations, arrow keys, and mouse hover highlight all work exactly as before.
5. `npm run build` must pass. Careful: unused imports are errors in this project.

**Questions to answer in your devlog**
- What props does `ResultRow` need, and why those?
- Why does the dropdown highlight depend on every row being the same height? (Hint: look for `--row` in `App.css`.)

**Don't touch:** the `present`, `dismiss`, and event-listener code in `App.tsx`, or any animation timing. That's the open/close logic and it's easy to break.
**You own:** `src/components/`.
**Bader:** hold off on editing `App.tsx` until this is merged, so you don't conflict.
**Stretch:** change `--accent` in `App.css` to another color and write down which parts of the UI changed.

---

## Ahmed (webdev): Newtron landing page, version 0

**Goal:** a simple one-page site for Newtron, separate from the app.

**Steps**
1. Create a `website/` folder at the repo root with `index.html` and `style.css`. Plain HTML and CSS, no frameworks, no build step.
2. Sections:
   - Hero with the name and tagline: *Search your machine. Ask your AI. Never leave the keyboard.*
   - Two features: **Fast local search** and **Local AI**.
   - A status line saying it's in **early development**.
   - A link to the GitHub repo, and a disabled "Download (coming soon)" button.
3. Match the app's look: dark panel `#16171c`, text `#ececf1`, accent `#7c9cff`, system font.
4. Make it work on a phone-width screen too.
5. No external requests (no CDN links), so it works offline.
6. Test by opening `index.html` in your browser.

**Rule for the copy:** everything on the page has to be true *today*. Nothing is released and search and AI aren't built yet. "Planned" and "coming soon" are fine, claims we can't back up are not.

**Questions to answer in your devlog**
- How did you make it responsive, and which CSS did the work?
- What would you change to host it for free later?

**Don't touch:** `src/` or `src-tauri/`.
**You own:** `website/`.
**Setup note:** do the "get it running" steps too, but if the app setup fights you for more than about 30 minutes, tell Bader and move on to the website.

---

## Rules for all of this

- Branch names: `feat/...`, `fix/...`, `docs/...`
- Every PR has a devlog entry, and you can explain every line.
- Don't merge your own PR. Reviewers: leave at least one question on every PR, not just "looks good".
- Stay in your own folder. If you need to change something outside it, ask first.