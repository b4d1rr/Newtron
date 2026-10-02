# Setting up Newtron on your machine

Newtron is Windows-only for now (Windows 10 or 11). Do this once and you're set.

## 1. Install the stuff

| Tool | Get it from | Notes |
|---|---|---|
| **Git** | git-scm.com | Default options are fine |
| **Node 24** | nodejs.org | Install **version 24 specifically**. The repo is locked to it so we all build the same way |
| **Rust** | rustup.rs | Install with defaults. The repo pins the exact Rust version, and rustup downloads it for you automatically the first time you build |
| **Visual Studio Build Tools** | visualstudio.microsoft.com/visual-cpp-build-tools | In the installer, tick **"Desktop development with C++"**. Rust needs this to compile on Windows |
| **WebView2** | Already on Windows 11 and most Windows 10 | If the app window never shows up, install the "Evergreen" runtime from Microsoft |

Restart your terminal after installing so it picks everything up.

## 2. Check it worked

```powershell
node -v      # should start with v24
cargo -V     # should print a version
git --version
```

## 3. Get the code and run it

```powershell
git clone https://github.com/b4d1rr/Newtron.git
cd Newtron
npm ci
npm run tauri dev
```

- `npm ci` (not `npm install`) installs the exact versions from the lockfile.
- The **first run takes several minutes** because Rust compiles all the dependencies. That's normal. After that, restarts take seconds.
- When a Newtron window opens, you're done.

Inside the project folder, `rustc -V` will show the pinned version, not necessarily whatever you have globally. That's the point.

## 4. Editor

We use VS Code. When you open the folder it will suggest two extensions (rust-analyzer and Tauri), say yes.

For docs, open the `docs/` folder as a vault in Obsidian.

## Stuff that will trip you up (and how to fix it)

**`npm run build` fails on an "unused" variable or import.**
Our TypeScript settings treat unused imports, variables, and parameters as errors. It's annoying the first time, but it keeps the code clean. Delete the unused thing and it passes.

**`linker 'link.exe' not found` or a similar C++ error.**
The Build Tools aren't installed, or you skipped the "Desktop development with C++" box. Rerun the Build Tools installer and add it.

**`Port 1420 is already in use`.**
Another copy of `tauri dev` is still running. Close it (check Task Manager for `node` or `newtron`) and try again.

**`Blocking waiting for file lock on artifact directory`.**
Another cargo or rust-analyzer process is using the build folder. Wait a moment, or close the other terminal.

**Rust errors about memory allocation or "stack overflow" during the first build.**
That's the compiler crashing, not your code. Close heavy apps and run the build again, and if it keeps happening, tell the group chat.

**`npm ci` complains the lockfile is out of sync.**
Don't fix it yourself, tell the group chat so we regenerate and commit it once for everyone.

**Wrong Node version warning (`EBADENGINE`).**
You're not on Node 24. Install it from nodejs.org and reopen your terminal.

**Installs or builds are super slow.**
Windows Defender scanning thousands of tiny files can slow everything down. Add exclusions for the project folder, `C:\Users\<you>\.cargo`, and `C:\Users\<you>\.rustup`. Also keep the repo out of OneDrive-synced folders.

**Git says "LF will be replaced by CRLF".**
Harmless warning, ignore it.

## What you don't need yet

Ollama (for the local AI part) isn't needed until we start building that module. We'll add instructions here when we do.

## Next

Read `CONTRIBUTING.md` for how we work: branches, PRs, and docs.