# Newtron docs

Everything about how Newtron is built lives here. Open this folder as a vault in Obsidian, or just browse it on GitHub.

The code tells you *what* the app does. These docs tell you *how it works right now*, *why we chose it*, and *what changed along the way*.

## Start here

| If you want to... | Read |
|---|---|
| Get the app running on your machine | [setup.md](setup.md) |
| Know how we work (branches, PRs, docs) | [../contributing.md](../contributing.md) |
| Understand the product and roadmap | [../README.md](../README.md) |
| See what's assigned right now | [tasks/](tasks/) |

New to the project? Read them in that order.

## What's in this folder

```text
docs/
├── setup.md          one-time dev environment setup (Windows)
├── modules/          how each part of the app works right now
├── decisions/        why we picked one option over another
├── devlog/           what changed, one entry per PR
├── tasks/            who is doing what, grouped by date
└── _templates/       copy these when you write a new doc
```

### `modules/`: how it works now
One file per part of the app. These are **living docs**: if you change how a module behaves, update its file in the same PR. History does not go here, it goes in the devlog.

- [shell](modules/shell.md): tray, global shortcut, frameless window, command bar UI and its open/close animation

Planned modules, added when the code lands: `db` (SQLite + FTS5), `indexer`, `search`, `ai` (Ollama).

### `decisions/`: why we chose it
Write one whenever we pick between real options (a library, an approach, a scope change). Each one lists the context, the options considered, what we chose, and when to revisit it. Number them in order.

- [0001 Restart from scratch with a smaller scope](<decisions/0001 scope-reduction and restart.md>)
- [0002 The UI owns the open/close animation](<decisions/0002 ui owns window animation.md>)
- [0002-1 Alt+N via the global-shortcut plugin](<decisions/0002-1 alt n via global shortcut plugin.md>)

### `devlog/`: what changed
One entry per PR, written by the person who made the change. Don't edit other people's entries (this avoids merge conflicts). The most important section is **"What I'd explain to a teammate"**.

- [0001 Scaffold and setup](<devlog/0001 26-10-02 bader scaffold and setup.md>)
- [0002 Tray, shortcut and bar UI](<devlog/0002 26-10-03 bader tray shortcut and bar ui.md>)

### `tasks/`: who is doing what
A folder per day with the task list for that day.

- [First tasks (Oct 3)](<tasks/2026-10-03/first tasks.md>)

### `_templates/`
Copy one of these instead of starting from a blank file:

- [decision.md](_templates/decision.md)
- [devlog.md](_templates/devlog.md)
- [module.md](_templates/module.md)

## Writing a new doc

1. Copy the right template from `_templates/`.
2. Put it in the matching folder.
3. Fill in the header (author, date, module, tags) and every section. Don't leave a section empty, write "None" if it truly doesn't apply.
4. Link to related docs so people can follow the trail (module to decisions to devlog).
5. Include it in the same PR as the code.

Every PR needs a devlog entry. Add a module doc or update one if behavior changed, and add a decision if you chose between real options. See [contributing.md](../contributing.md) for the full checklist.

## Rules of thumb

- **Be honest about status.** If something isn't built, say so. Docs describe what is true today, and plans go in the roadmap.
- **Explain in your own words.** If you can't explain it, it isn't ready to merge.
- **Keep module docs short and current.** A stale module doc is worse than none.
- **Record the "why".** Code shows what we did, the decision log shows what we rejected and why.