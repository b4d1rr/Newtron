---
number: 0003
date: 2026-10-04
status: accepted
deciders: [Bader]
tags: [decision]
---

# 0003: Use rusqlite with bundled SQLite for the file index

## Context
Search needs an index it can query in milliseconds, not a live scan of the filesystem on every keystroke. Decision 0001 already settled on SQLite with FTS5 (full-text search) as the storage and search engine. What was still open is *which Rust library* talks to SQLite, and *where SQLite itself comes from* on the user's machine.

Constraints:
- Windows only for now, and Windows doesn't ship a SQLite library we can rely on.
- The team is still learning Rust, so the API needs to be small and explainable.
- Nothing here needs to be async yet. The first job is "open a database, create a table, query it".

## Options considered
1. **rusqlite with the `bundled` feature**: a thin Rust wrapper around SQLite. `bundled` compiles SQLite's C source into our app, so there is nothing to install and every teammate gets the same SQLite version. Small, synchronous API. Cost: the first build is slower (about a minute extra on my machine) and the app binary is a bit larger.
2. **rusqlite without `bundled`** (use a system SQLite): faster build and smaller binary, but Windows has no standard SQLite to link against. Every teammate would need to install and point to one, and a version mismatch could silently break FTS5. This is the "works on my machine" problem we pinned versions to avoid.
3. **sqlx or diesel**: bigger libraries that add async, migrations, and query builders. Powerful, but much more to learn and explain, and we don't need any of it yet.
4. **Skip SQLite and use `LIKE` over plain files or a Rust `Vec`**: no new dependency, but it scans everything on every search and can't rank results. This was already ruled out in decision 0001.

## Decision
Option 1: `rusqlite` with the `bundled` feature, using SQLite's FTS5 virtual tables for search. Database code lives in its own file, `src-tauri/src/db.rs`.

## Consequences
- **Good:** one command to add, nothing to install, and the same SQLite and FTS5 on every machine. The API is small enough that everyone can explain it. Tests can use an in-memory database, so they are fast and leave nothing on disk.
- **Bad:** the first compile takes noticeably longer because SQLite's C code is built too. The `rusqlite` `Connection` is not safe to share across threads, so we will need a plan (a mutex, or one connection per thread) once the indexer runs in the background.
- **Revisit when:** we need async database access, a migration system, or fuzzy matching that FTS5's default tokenizer can't do (for example trigram matching). Also revisit if the build time or binary size becomes a real problem.