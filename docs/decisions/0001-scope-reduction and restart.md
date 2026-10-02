---
number: 0001
date: 2026-10-02
status: accepted
deciders: [Bader]
tags: [decision]
---

# 0001: Restart from scratch with a smaller scope

## Context
- We have roughly two months. Targets: MVP on Nov 1, 2026 and v1.0 on Dec 1, 2026.
- The first version had a lot going on: web search, cloud AI with your own API keys, URL autocomplete, a file indexer, an app launcher, and a fancy UI.
- The team is still learning, so we need a codebase everyone can follow.

## Options considered
1. **Keep the old version and trim it down.** Faster to start, but we'd inherit code nobody fully understands, and we'd still be debugging it.
2. **Restart from a clean scaffold with a smaller scope.** Slower on day one, but every piece gets built on purpose and understood.

## Decision
Restart from a clean Tauri v2 + React + TypeScript scaffold, and narrow the product to two things done well: **fast local search** (SQLite + FTS5, with app launching) and **local AI through Ollama**.

New rule: we only merge code we can explain. Tools can help, but if you can't say why a line is there, it doesn't go in.

## Consequences
- **Good:** a codebase we all understand, a scope we can actually finish, and docs that explain the reasoning as we go.
- **Cut for now:** embedded web search, cloud AI with your own keys (OpenAI / Anthropic / Gemini), URL autocomplete from browser history, and the glass UI kit.
- **Bad:** we're rebuilding the shell (global shortcut, command bar) that existed before, so some early progress is redone.
- **Revisit when:** MVP ships on Nov 1. If we're ahead of schedule, cloud AI is the first thing to reconsider.