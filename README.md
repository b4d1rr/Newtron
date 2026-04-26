<div align="center">

# ⚡ Newtron

### Your computer's new central nervous system.

**One keystroke. Every tool. Zero tab switching.**

![Status](https://img.shields.io/badge/status-under%20active%20development-yellow?style=flat-square)
![Stack](https://img.shields.io/badge/stack-Rust%20%2B%20TypeScript-blue?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)
![Last Updated](https://img.shields.io/badge/updated-April%202026-lightgrey?style=flat-square)

</div>

---

## What is Newtron?

Newtron is a system-wide command bar built for people who live on their keyboard. Press `Alt + N` from anywhere on your machine and get instant access to your files, apps, the web, and every AI you use — all in one fast, lightweight interface.

No Electron bloat. No subscription. No switching tabs.

Built with **Rust** for performance and **React + TypeScript** for a fluid, modern UI.

---

## Getting Started

### Prerequisites

Make sure you have these installed before running Newtron:

- [Node.js (LTS)](https://nodejs.org)
- [Rust](https://rustup.rs)

### Running Newtron

1. Clone the repository
   ```bash
   git clone https://github.com/b4d1rr/Newtron.git
   cd Newtron
   ```

2. Run the setup script by double-clicking `setup.bat` or running it in your terminal:
   ```bash
   ./setup.bat
   ```
   The script will automatically verify Node and Rust, install the Tauri CLI if missing, sync all dependencies, and launch Newtron in dev mode.

3. Press `Alt + N` — Newtron appears instantly.

> ⚠️ If setup.bat errors on Node or Rust, install them from the links above and rerun.

---

## Features

### 🧠 AI Command Bar
Connect your own API keys for OpenAI, Anthropic, or Gemini — stored securely in your OS keychain, never on our servers. Switch models mid-session directly from the bar.

```
@claude explain this function
@gpt4o rewrite this email
@local summarize my clipboard
```

No account required to get started. Newtron ships with a built-in local AI (via Ollama) that works instantly, offline, and for free.

### 🌐 Web Search in the Dropdown
Type a query, see Google results rendered directly inside Newtron. No browser switch. No context loss.

### 📁 Lightning-Fast File Search
Rust-powered local file indexer backed by SQLite. Finds anything on your machine in milliseconds — files, folders, Git repos, system settings.

### 🚀 App Launcher
Launch any application from the bar. No mouse required.

### 🔒 Private by Design
- File indexing happens entirely on your machine
- API keys live in your OS keychain (AES-256)
- Your queries go directly to the AI provider — no middleman, no logging
- Local AI mode means nothing leaves your machine at all

---

## How It Works

```
Press Alt + N from anywhere
           ↓
┌──────────────────────────────┐
│  > ________________________  │
├──────────────────────────────┤
│  🧠  Ask AI                 │
│  🌐  Search Google          │
│  📁  Files matching...      │
│  🚀  Launch App             │
└──────────────────────────────┘
```

One input. Every result type. You choose what to act on.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Core | Rust (Tauri) |
| Frontend | React + TypeScript |
| Styling | TailwindCSS + Framer Motion |
| Local AI | Ollama |
| Cloud AI | OpenAI / Anthropic / Gemini (BYOK) |
| File Index | SQLite (local) |
| Key Storage | OS Keychain via Tauri `keyring` |

---

## AI Setup

Newtron works out of the box with local AI. To connect cloud models, add your own API key in settings — it takes 30 seconds and is stored exclusively in your system keychain. No accounts. No login. No middleman.

| Model | Provider | Free? |
|---|---|---|
| Llama 3 / Mistral | Ollama (local) | ✅ Always free |
| Gemini 1.5 Flash | Google AI Studio | ✅ 1,500 req/day |
| Gemini 1.5 Pro | Google AI Studio | ✅ 50 req/day |
| GPT-4o | OpenAI (BYOK) | Your key |
| Claude 3.5 | Anthropic (BYOK) | Your key |

---

## Roadmap to Alpha

- [ ] Rust-based global shortcut listener (`Alt + N`)
- [ ] File indexer + SQLite search engine
- [ ] Google search result rendering in dropdown
- [ ] Ollama local AI integration
- [ ] BYOK key manager (OS keychain)
- [ ] Cloud AI routing (`@model` syntax)
- [ ] Glassmorphism UI kit + animations
- [ ] App launcher (OS-level)
- [ ] Public Alpha Release

---

## Current Status

> 🏗️ **Stealth Development** — Core engine is being stabilized before the first alpha. No public binaries yet.

Newtron is a closed-contribution project while we finalize the architectural foundation. An open-source call to action is coming with the alpha release.

---

## Privacy & Security

- **Local stays local.** File indexing never leaves your machine.
- **Your keys, your control.** API keys are stored in your OS native keychain — we never see them.
- **No accounts.** Newtron uses a BYOK model — no login, no sessions, no tracking.
- **No middleman.** Queries go directly from your machine to the AI provider.
- **Offline capable.** Local AI mode works with zero internet connection.

---

<div align="center">

**Newtron — Stop switching tabs. Start thinking faster.**

</div>
