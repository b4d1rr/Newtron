# How we work on Newtron

Quick guide so we're all doing things the same way. Nothing scary, just read it once and come back when you forget something.

## The one rule

**Don't merge stuff you can't explain.** If someone asks "why is this line here?", you should have an answer. That goes for code you wrote, code you found online, and code a tool gave you. Don't get it? Ask first, that's what the team is for.

It's also the whole point of the project: this goes on our college applications, so we want to actually understand what we built when someone asks us about it.

## Getting set up

1. Do the steps in `docs/setup.md`.
2. Clone and run it:
   ```powershell
   git clone https://github.com/b4d1rr/Newtron.git
   cd Newtron
   npm ci
   npm run tauri dev
   ```
3. The first run takes a few minutes while Rust compiles everything. After that it's fast.
4. A Newtron window pops up? You're good. Anything else, drop the full error in the group chat.

Use `npm ci`, not `npm install`. It installs the exact versions in the lockfile so we all have the same build.

## Where stuff lives

```
src/                  the UI (React + TypeScript)
src-tauri/src/        the backend (Rust), lib.rs is where our code goes
src-tauri/Cargo.toml  Rust dependencies
docs/                 all our docs (open this folder in Obsidian)
```

Yes there are two `src` folders. Root one = UI, the one in `src-tauri` = Rust.

## The loop

```powershell
git switch main
git pull
git switch -c feat/what-youre-building     # like feat/file-indexer
npm run tauri dev                           # updates live when you save
```

Commit often, small steps. Never commit straight to `main`.

- **Branch names:** `feat/...` for new stuff, `fix/...` for bugs, `docs/...` for docs.
- **Commit messages:** say what changed. `Add rescan timestamp to index table` is good, `stuff` is not.
- **main moved while you were working?**
  ```powershell
  git fetch
  git merge origin/main
  ```
  Fix any conflicts, make sure the app still runs, keep going. Stuck on a conflict? Ask before you start guessing.

## Before you open a PR

From `src-tauri`:
```powershell
cargo fmt
cargo clippy
cargo test
```
From the repo root:
```powershell
npm run build
```
Then read your own diff on GitHub first. You'll spot stuff.

## Opening a PR

- Target `main`, keep it small (one thing per PR), say what changed and how to test it.
- Tick the checklist:
  - [ ] I added a devlog entry
  - [ ] I updated the module doc (if something changed)
  - [ ] I can explain every line in this PR
- Don't merge your own PR. Once it's approved, hit **Squash and merge**.

## Docs

Docs go in the same PR as the code. Everything lives in `docs/`.

- **Devlog** (`docs/devlog/`): one per PR. Copy `_templates/devlog.md` and name it `YYYY-MM-DD-yourname-topic.md`.
- **Module docs** (`docs/modules/`): how a part of the app works *right now*. If you change how it works, update the file.
- **Decisions** (`docs/decisions/`): when we pick between real options (like a library). Number them in order.

Don't edit other people's devlog entries, that's how we avoid merge conflicts. The most important section is **"What I'd explain to a teammate"**, so write it for real.

## Testing

- Anything that's just logic (indexing, ranking, timing) gets a `cargo test`.
- UI stuff: test it by hand and write the steps in your devlog under "How to test it".
- When reviewing someone's PR, actually run their branch, don't just read the code:
  ```powershell
  git fetch
  git worktree add ..\newtron-review their-branch-name
  cd ..\newtron-review
  npm ci
  npm run tauri dev
  ```
  Only one Newtron can run at a time (the shortcut clashes), so close yours first. When you're done: `git worktree remove ..\newtron-review`.

## Don'ts

- Don't commit `node_modules/`, `src-tauri/target/`, API keys, or `.env` files.
- Don't force-push (`git push --force`) unless it's your own branch and you know why.
- Don't add a dependency or change the Rust/Node version without telling everyone first, it changes everybody's build.
- Don't reformat whole files you aren't changing, it hides the real changes.

## Stuck?

1. Read the error. Rust errors are usually really specific and often tell you the fix.
2. Search the docs, someone might've hit it already.
3. Ask in the group chat with what you tried, the exact error, and your branch name.