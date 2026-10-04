---
author: Bader
date: 2026-10-04
module: db
commit:
pr: (add the PR link once it's open)
tags: [devlog]
---

# SQLite hello world: rusqlite, an FTS5 table, and a passing prefix-search test

## What I did
Added `rusqlite` to the Rust backend and created `src-tauri/src/db.rs`. It opens an in-memory database, creates an FTS5 table (`files_fts(name, path)`), and has a `search()` function that does prefix matching. A test inserts three fake files and checks that searching `rep` finds only `report.pdf`. `cargo test` passes. I also filled in the empty `docs/README.md` so the docs folder has an index.

Nothing in the app calls this code yet, and there is no UI change.

## Why
Search is half of what Newtron is for, and everything in it depends on being able to store files and query them fast. Before building the indexer, I wanted proof that the pieces work together: Rust, SQLite, and FTS5. Doing it as a tiny in-memory test keeps it small enough to fully understand before building on top of it. The library choice is explained in `docs/decisions/0003 rusqlite with bundled sqlite.md`.

## How it works
- `Cargo.toml` has `rusqlite` with the `bundled` feature. `lib.rs` has one new line, `mod db;`.
- `open_memory()` makes an in-RAM database and runs `CREATE VIRTUAL TABLE files_fts USING fts5(name, path)`.
- `search(conn, prefix)` turns `rep` into `rep*` and runs `SELECT name FROM files_fts WHERE files_fts MATCH ?1 ORDER BY rank`. The `?1` is a parameter, so the value is passed separately from the SQL.
- `query_map` runs the query and reads the first column of each row as a `String`. `collect()` turns the results into one `Result<Vec<String>>`.
- Full details are in `docs/modules/db.md`.

## How to test it
1. `cd src-tauri`
2. `cargo test`
3. You should see `test db::tests::prefix_search_finds_report ... ok` and `1 passed; 0 failed`.
4. Two `never used` warnings (`open_memory`, `search`) are expected, because only the test calls them.

The first run compiles SQLite's C code, so it took about a minute for me. Later runs are fast.

## What I'd explain to a teammate
**Walkthrough:** the test makes a database in memory, adds three rows, and asks for everything starting with `rep`. FTS5 keeps an index of words instead of rows of text, so it jumps straight to the match.

**What does `bundled` do, and why do we use it?** It compiles SQLite's C source into our app. Windows has no SQLite library we can count on, so without it each person would have to install one, and a different version could behave differently. With it, everyone gets the same SQLite and FTS5 from one `cargo add`. The cost is a slower first build.

**What is FTS5, and how is it different from `LIKE '%rep%'`?** `LIKE '%rep%'` reads every row and checks the text each time, so it gets slower as the number of files grows. FTS5 splits text into words when you insert it and builds an index from word to rows, so a lookup doesn't read every row, and it can rank results. The tradeoff is that it matches whole words or word starts, so `port*` won't find `report`.

**Why an in-memory database in the test?** It's fast, it leaves nothing on disk to clean up, and every test starts from an empty database, so tests can't affect each other.

**Other lines worth knowing:**
- The `?` after a call returns the error to the caller instead of crashing.
- Parameters (`?1`) keep user input out of the SQL string. Pasting strings into SQL is how injection bugs happen.
- `ORDER BY rank` sorts the best matches first.

## Things I'm unsure about
- **User input isn't sanitized.** If someone types `"`, `-`, `:` or nothing at all, FTS5 returns a syntax error. I need to quote each word before connecting the search bar.
- `MATCH` searches both columns, so a query can match on the path, not just the name. I may want `name:` to restrict it.
- It's in memory only. Opening a real database file in the app's data folder is next.
- `Connection` isn't safe to share across threads. Once the indexer runs in the background I need a plan for that (a mutex, or one connection per thread), and probably a decision doc.
- The dead-code warnings will stay until the app uses these functions.

## Links
- PR: (add the PR link once it's open)
- Decision: `docs/decisions/0003 rusqlite with bundled sqlite.md`
- Module doc: `docs/modules/db.md`
- Earlier decision that chose SQLite + FTS5: `docs/decisions/0001 scope-reduction and restart.md`