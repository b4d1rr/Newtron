<div align="center">

# ⚡ Newtron

### Search your machine. Ask your AI. Never leave the keyboard.

[![Status](https://img.shields.io/badge/status-pre--development-orange?style=flat-square)](#current-status)
[![License: MIT](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%20v2-24C8DB?style=flat-square)](https://tauri.app)
[![Rust](https://img.shields.io/badge/backend-Rust-DEA584?style=flat-square)](https://www.rust-lang.org)
[![Local AI](https://img.shields.io/badge/AI-100%25%20local-blueviolet?style=flat-square)](#2-local-ai)

</div>

---

Newtron is a system-wide command bar. One global shortcut, and you can search every file, folder, and app on your machine — or hand a question to an AI running entirely on your own hardware. No cloud round-trip, no account, no tab switching.

This is a deliberate rebuild. The earlier prototype tried to do too much at once; this version does two things and does them well:

> 🔎 **Fast local search** &nbsp;·&nbsp; 🧠 **Local AI, via Ollama**

---

## 📍 Current Status

**Pre-development.** The previous prototype has been scrapped and this rebuild starts from a clean slate — nothing in this README is implemented yet. Treat it as the working spec, not a changelog.

No build, installer, or release exists at this stage.

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

Once code exists, this table gets checked against the real `package.json` / `Cargo.toml` / `tauri.conf.json` and corrected if anything's off.

---

## 🗺️ Roadmap

<details open>
<summary><strong>Phase 1 — Foundation</strong></summary>

- [ ] Tauri application shell
- [ ] React + TypeScript frontend
- [ ] Rust backend
- [ ] Frontend ↔ Rust IPC
- [ ] SQLite database setup
- [ ] Global shortcut + popup command bar
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

> Setup instructions land here once the project scaffold exists.

Expected prerequisites:

- [Node.js (LTS)](https://nodejs.org)
- [Rust](https://rustup.rs)
- [Ollama](https://ollama.com) — for local AI, once implemented

---

## 🔒 Privacy

Local-first isn't a checkbox here, it's the reason the project exists:

- File indexing happens entirely on-device.
- Local AI mode (Ollama) means prompts and responses never leave your machine.
- No accounts, no telemetry, no cloud dependency for either core feature.

These are design commitments for the rebuild — not verified guarantees until the corresponding code ships.

---

## 🤝 Contributing

Closed to contributions while the foundation gets rebuilt. That changes once there's a stable base to build on.

---

## 📄 License

MIT — see [LICENSE](LICENSE).

<div align="center">

---

**Newtron — one shortcut, your files, your AI.**

</div>
