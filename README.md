# Newtron

**A lightweight, privacy-first command bar for fast local search and local AI.**

Newtron is a system-wide command bar that lives on a global keyboard shortcut. Press it from anywhere, and search your files, folders, and applications — or ask a locally-running AI model a question — without touching your mouse or leaving what you're doing.

This is a ground-up rebuild of the project, scoped intentionally to two things done well rather than a broad feature set: **fast local search** and **local AI**.

---

## Current Status

🏗️ **Pre-development.** The previous prototype has been scrapped and this rebuild is starting from a clean slate. Nothing described below is implemented yet — this document describes the intended direction and serves as the working spec for the build.

There is no working build, installer, or release at this stage.

---

## Key Features (Target)

### 1. Fast Local Search

Open Newtron with a global shortcut and search your computer for files, folders, and applications. The goal is search that feels instant, backed by an indexed database rather than a live filesystem scan on every keystroke.

Planned capabilities:

- File and folder metadata indexing
- Incremental indexing (only re-index what changed)
- Filename and path search with prefix, partial, and fuzzy matching
- Relevance ranking and recently-used/opened results
- Full keyboard navigation
- Opening files/folders and launching applications directly from results

### 2. Local AI

Newtron connects to a locally-running LLM through [Ollama](https://ollama.com), so prompts and model responses stay on your machine — no API keys, no accounts, no data leaving your computer.

Planned capabilities:

- Detecting a running Ollama instance and its available models
- Selecting a model to use
- Sending prompts and streaming responses back into the command bar
- Cancelling an in-progress generation
- Conversation history within a session
- Optional local file context (asking questions about files on your machine)

---

## Architecture (Planned)

```text
Global Shortcut
      ↓
┌───────────────────────────────────────┐
│ 🔍 Search files, apps, or ask AI...    │
├───────────────────────────────────────┤
│ 📄 Newtron Architecture.pdf            │
│ 📁 Newtron                             │
│ 💻 Visual Studio Code                  │
│ 🤖 Ask local AI: "summarize this..."   │
└───────────────────────────────────────┘
```

**Search path:**

```text
React/TypeScript UI → Tauri → Rust backend → local file index → SQLite / FTS5
```

**Local AI path:**

```text
React/TypeScript UI → Tauri → Rust backend → Ollama → local LLM
```

The UI itself is a minimal, keyboard-first popup — not a headline feature in its own right, just the shell the two capabilities above are delivered through.

---

## Technology Stack (Target)

| Layer         | Technology                  |
| ------------- | ---------------------------- |
| Shell         | Tauri v2                     |
| Backend       | Rust                         |
| Frontend      | React + TypeScript + Vite    |
| Styling       | Tailwind CSS                 |
| File index    | SQLite + SQLite FTS5         |
| Local AI      | Ollama                       |

This table reflects the intended stack for the rebuild. It will be corrected against the actual codebase (`package.json`, `Cargo.toml`, `tauri.conf.json`) once implementation begins.

---

## Roadmap

### Phase 1 — Foundation
- [ ] Tauri application shell
- [ ] React + TypeScript frontend
- [ ] Rust backend
- [ ] Frontend ↔ Rust IPC
- [ ] SQLite database setup
- [ ] Global shortcut + popup command bar

### Phase 2 — File Indexing
- [ ] Directory traversal
- [ ] File metadata extraction
- [ ] SQLite indexing
- [ ] Incremental indexing
- [ ] Handling deleted/moved files
- [ ] Indexing performance testing

### Phase 3 — Search
- [ ] SQLite FTS5 integration
- [ ] Prefix and partial matching
- [ ] Fuzzy matching
- [ ] Relevance ranking
- [ ] Recent-result ranking
- [ ] Keyboard navigation
- [ ] Open files/folders, launch apps

### Phase 4 — Local AI
- [ ] Ollama detection
- [ ] Model detection and selection
- [ ] Prompt interface
- [ ] Streaming responses
- [ ] Generation cancellation
- [ ] Conversation history
- [ ] Local file context

### Phase 5 — Performance & Reliability
- [ ] Large-filesystem testing
- [ ] Search latency benchmarking
- [ ] Indexing/memory/CPU optimization
- [ ] Error and crash handling
- [ ] Edge-case testing

### Phase 6 — Release
- [ ] Installer
- [ ] Application icon
- [ ] Versioning and settings persistence
- [ ] Documentation, screenshots, demo
- [ ] v1.0.0 release build

---

## Target Milestones

These are development targets, not completed work.

**November 1, 2026 — MVP**
- Command bar shell
- Fast local file search
- Application launching
- Initial local AI integration (single model via Ollama)

**December 1, 2026 — v1.0**
- Stable, fast search
- Reliable incremental indexing
- Working local AI with conversation history
- Performance optimization and error handling
- Packaging and documentation

---

## Development

> Setup instructions will be filled in once the project scaffold exists.

Expected prerequisites, based on the target stack:

- [Node.js (LTS)](https://nodejs.org)
- [Rust](https://rustup.rs)
- [Ollama](https://ollama.com) (for local AI, once implemented)

---

## Privacy

Local-first is the point of this rebuild, not an afterthought:

- File indexing is intended to happen entirely on-device.
- Local AI mode (via Ollama) is designed so prompts and responses never leave your machine.
- No accounts, no telemetry, no cloud dependency for the core features.

These are design commitments for the rebuild, not verified guarantees until the corresponding code exists.

---

## Contributing

This project is not yet open for contributions while the foundation is being rebuilt. That will change once there's a stable base to build on.

---

## License

MIT — see [LICENSE](LICENSE) for details.
