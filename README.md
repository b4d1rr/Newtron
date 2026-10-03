<div align="center">

# ⚡ Newtron

### Search your machine. Ask your AI. Never leave the keyboard.

[![Status](https://img.shields.io/badge/status-early%20development-orange?style=flat-square)](#current-status)
[![License: MIT](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%20v2-24C8DB?style=flat-square)](https://tauri.app)
[![Rust](https://img.shields.io/badge/backend-Rust-DEA584?style=flat-square)](https://www.rust-lang.org)
[![Local AI](https://img.shields.io/badge/AI-100%25%20local-blueviolet?style=flat-square)](#-what-it-does-target)

</div>

---

Newtron is a system-wide command bar. One global shortcut, and you can search every file, folder, and app on your machine — or hand a question to an AI running entirely on your own hardware. No cloud round-trip, no account, no tab switching.

Newtron is scoped deliberately to two things, done well:

> 🔎 **Fast local search** &nbsp;·&nbsp; 🧠 **Local AI, via Ollama**

---

## 📍 Current Status

**Early development.** The shell works: Newtron runs from the system tray, `Alt + N` brings up the command bar, and it opens and closes with a short animation that grows out of the search field. Search and AI aren't built yet, so the results dropdown shows placeholder data.

**Works today**

- Tray icon with *Show Newtron* and *Quit Newtron*, starts hidden
- `Alt + N` shows or hides the bar, `Esc` or clicking anywhere else closes it
- Minimal dark bar with an animated results dropdown (placeholder results)
- `Enter` or the globe button opens your text as a web search in your browser

**Not built yet**

- File and app search, app launching
- Local AI
- The SQLite index behind search

No installer or release yet.

---

## ✨ What It Does *(target)*

<table>
<tr>
<td width="50%" valign="top">

### 🔎 Fast Local Search

Instant, indexed search — not a live filesystem scan on every keystroke.

- File & folder metadata indexing
- Incremental indexing (only what changed)
- Prefix, partial, and fuzzy matching
- Relevance ranking + recently-used results
- Full keyboard navigation
- Open files/folders or launch apps, no mouse

</td>
<td width="50%" valign="top">

### 🧠 Local AI

Runs through [Ollama](https://ollama.com) — prompts and responses never leave your machine.

- Detects a running Ollama instance & its models
- Model selection
- Streaming responses in the bar
- Cancel generation mid-stream
- Session conversation history
- Optional local file context

</td>
</tr>
</table>

---

## 🏗️ How It Fits Together *(planned)*

```text
                    Global Shortcut
                          │
                          ▼
        ┌───────────────────────────────────┐
        │ 🔍 Search files, apps, or ask AI…  │
        ├───────────────────────────────────┤
        │ 📄 Newtron Architecture.pdf        │
        │ 📁 Newtron                         │
        │ 💻 Visual Studio Code              │
        │ 🤖 Ask local AI: "summarize this…" │
        └───────────────────────────────────┘
```

| Path | Flow |
|---|---|
| **Search** | React/TypeScript UI → Tauri → Rust backend → local file index → SQLite / FTS5 |
| **Local AI** | React/TypeScript UI → Tauri → Rust backend → Ollama → local LLM |

The command bar itself is deliberately just the shell — minimal, keyboard-first, and in service of the two capabilities above rather than a feature in its own right.

---

## 🧰 Tech Stack *(target)*

| Layer | Technology |
|---|---|
| Shell | Tauri v2 |
| Backend | Rust |
| Frontend | React + TypeScript + Vite |
| Styling | Tailwind CSS |
| File index | SQLite + FTS5 |
| Local AI | Ollama |

The table is the plan. What differs so far: styling is plain CSS, and there's no SQLite yet. It gets checked against the real `package.json` / `Cargo.toml` / `tauri.conf.json` as things land.

---

## 🗺️ Roadmap

<details open>
<summary><strong>Phase 1 — Foundation</strong></summary>

- [x] Tauri application shell
- [x] React + TypeScript frontend
- [x] Rust backend
- [x] Frontend ↔ Rust messaging
- [ ] SQLite database setup
- [x] Global shortcut + popup command bar
</details>

<details>
<summary><strong>Phase 2 — File Indexing</strong></summary>

- [ ] Directory traversal
- [ ] File metadata extraction
- [ ] SQLite indexing
- [ ] Incremental indexing
- [ ] Handling deleted/moved files
- [ ] Indexing performance testing
</details>

<details>
<summary><strong>Phase 3 — Search</strong></summary>

- [ ] SQLite FTS5 integration
- [ ] Prefix and partial matching
- [ ] Fuzzy matching
- [ ] Relevance ranking
- [ ] Recent-result ranking
- [ ] Keyboard navigation
- [ ] Open files/folders, launch apps
</details>

<details>
<summary><strong>Phase 4 — Local AI</strong></summary>

- [ ] Ollama detection
- [ ] Model detection and selection
- [ ] Prompt interface
- [ ] Streaming responses
- [ ] Generation cancellation
- [ ] Conversation history
- [ ] Local file context
</details>

<details>
<summary><strong>Phase 5 — Performance & Reliability</strong></summary>

- [ ] Large-filesystem testing
- [ ] Search latency benchmarking
- [ ] Indexing/memory/CPU optimization
- [ ] Error and crash handling
- [ ] Edge-case testing
</details>

<details>
<summary><strong>Phase 6 — Release</strong></summary>

- [ ] Installer
- [ ] Application icon
- [ ] Versioning and settings persistence
- [ ] Documentation, screenshots, demo
- [ ] v1.0.0 release build
</details>

---

## 🎯 Target Milestones

*Development targets — not completed work.*

| Milestone | Date | Scope |
|---|---|---|
| **MVP** | Nov 1, 2026 | Command bar shell · fast local search · app launching · initial local AI (single model) |
| **v1.0** | Dec 1, 2026 | Stable search · reliable indexing · local AI with conversation history · performance tuning · packaging & docs |

---

## 🛠️ Development

Windows only for now. Full setup steps are in [`docs/setup.md`](docs/setup.md), and how we work (branches, PRs, docs) is in [`CONTRIBUTING.md`](CONTRIBUTING.md).

The short version:

```bash
git clone https://github.com/b4d1rr/Newtron.git
cd Newtron
npm ci
npm run tauri dev
```

The first build takes a few minutes. Newtron starts hidden in the system tray, so look for its icon (it may be under the ^ arrow), then press `Alt + N`.

Ollama is only needed for the local AI part, once we start building it.

---

## 📚 Docs

Everything about how Newtron is built lives in [`docs/`](docs/):

- [`docs/setup.md`](docs/setup.md) — getting a dev environment running
- [`docs/modules/`](docs/modules/) — how each part works right now
- [`docs/decisions/`](docs/decisions/) — why we chose what we chose
- [`docs/devlog/`](docs/devlog/) — what changed, one entry per PR

---

## 🔒 Privacy

Local-first isn't a checkbox here, it's the reason the project exists:

- File indexing happens entirely on-device.
- Local AI mode (Ollama) means prompts and responses never leave your machine.
- No accounts, no telemetry, no cloud dependency for either core feature.

These are design commitments — not verified guarantees until the corresponding code ships. (The web search button hands your text to your browser and search engine by design, and it's separate from local search.)

---

## 🤝 Contributing

Closed to outside contributions while the foundation is being built. Team members, see [`CONTRIBUTING.md`](CONTRIBUTING.md).

---

## 📄 License

MIT — see [LICENSE](LICENSE).

<div align="center">

---

**Newtron — one shortcut, your files, your AI.**

</div>