---
module: db
owner: Bader
status: in progress
last_updated: 2026-10-04
tags: [module]
---

# DB (SQLite + FTS5)

> Living doc: update this whenever the module's behavior changes. History goes in the devlog, not here.

## Purpose
The storage and search layer for Newtron. It will hold the file index and answer search queries quickly using SQLite's FTS5 full-text search. Today it is only a proof that Rust can talk to SQLite and that FTS5 prefix search works. The app does not use it yet.

## How it works now

**Files**
- `src-tauri/src/db.rs`: all database code and its test
- `src-tauri/src/lib.rs`: contains `mod db;` (the only change to this file)
- `src-tauri/Cargo.toml`: depends on `rusqlite` with the `bundled` feature

**Flow**
1. `open_memory()` opens a database that lives in RAM and creates a virtual FTS5 table: `files_fts(name, path)`.
2. Rows are inserted with a normal `INSERT`. FTS5 splits each value into words and stores them in an inverted index (word to rows), the same idea as the index at the back of a book.
3. `search()` appends `*` to the text it is given (so `rep` becomes `rep*`) and runs `WHERE files_fts MATCH ?1 ORDER BY rank`. FTS5 looks the prefix up in the index instead of scanning every row, and `rank` orders the best matches first.
4. The result is the matching `name` values as a `Vec<String>`.

**Tokenizing:** FTS5's default tokenizer splits on punctuation, so `report.pdf` is stored as the words `report` and `pdf`. Prefix search matches the *start* of a word, so `rep*` finds it, but `port*` does not.

## Public interface
Rust functions only. Nothing is exposed to the UI as a Tauri command yet.

| Function | Input | Output |
|---|---|---|
| `open_memory()` | none | `Result<Connection>`: an in-memory database with the `files_fts` table created |
| `search(conn, prefix)` | a `&Connection` and a `&str` prefix | `Result<Vec<String>>`: matching file names, best match first |

## Data
- **`files_fts(name, path)`**: FTS5 virtual table, in memory only.
- Nothing is saved to disk, so everything is lost when the connection closes.

## How to test
From `src-tauri`:

```powershell
cargo test
```

Expected: `db::tests::prefix_search_finds_report ... ok` and `1 passed`. The test inserts three fake rows (`report.pdf`, `notes.txt`, `photo.png`) and checks that searching `rep` returns only `report.pdf`.

You will see two `never used` warnings for `open_memory` and `search`. That is expected until something outside the tests calls them.

## Known limits / TODO
- **Raw user input is not safe yet.** FTS5 has its own query syntax, so typing `"`, `-`, `:` or an empty string produces a syntax error. Each word needs to be wrapped in double quotes (and any `"` inside doubled) before real user text goes in.
- `MATCH` on the whole table searches **every column**, so `docs` also matches rows whose path contains `docs`. Use `name:rep*` to search file names only.
- In-memory only. Next step is opening a database file in the app's data folder.
- Only the FTS table exists. A real `files` table (path, size, modified time, last-seen time) is needed for incremental indexing.
- No indexer yet, and no Tauri command for the UI to call.
- `Connection` can't be shared between threads. We need a plan (mutex, or one connection per thread) before the indexer runs in the background.

## Related
- Decisions: 0001 (restart with a smaller scope, which chose SQLite + FTS5), 0003 (rusqlite with bundled SQLite)
- Devlog entries: `0003 26-10-04 bader sqlite hello world`