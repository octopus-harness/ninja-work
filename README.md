# octo-work (`work`)

```text
 ▄▄▄▄   ▄▄▄▄  ▄▄▄▄▄▄  ▄▄▄▄          ▄▄ ▄▄  ▄▄▄▄   ▄▄▄▄  ▄▄ ▄▄ 
██  ██ ██       ██   ██  ██  ▄▄▄▄▄  ██ ██ ██  ██ ██▄█▀ ██ ██ 
 ▀██▀   ▀████   ██    ▀██▀          ▀███▀  ▀██▀  ██    ▀███▀ 
```

A fast, keyboard-driven CLI and 3-pane Ratatui TUI workspace manager for [Octopus](https://github.com/leouofa/octopus) AI agent harnesses.

`octo-work` provides a compiled drop-in replacement for Octopus's original `bin/work` shell script while expanding it with a full workspace lifecycle engine (`add`, `archive`, `restore`, `rename`, `delete`, `list`, `path`, `init`) and a companion terminal dashboard modeled after `doer-tui` and `noteking-tui`.

---

## Features

- **Drop-in Compatibility**: Fully compatible with `work add [name]`, `work archive [name]`, and interactive `work` prompts.
- **3-Pane Ratatui TUI**:
  - **Pane 1 (Workspaces)**: Browse, switch between `Active` and `Archive` scopes, filter in real-time (`/`), add, archive/restore, rename, and delete.
  - **Pane 2 (Files)**: Inspect files inside the active workspace, create new files (`n`), rename (`r`), delete (`d`), or open directly in `$EDITOR` (`e`).
  - **Pane 3 (Preview & Metadata)**: Instant text and Markdown preview with syntax styling (headers, checklists, code blocks, blockquotes) and raw/rendered toggle (`v`).
- **Smart Timestamp Collision Handling**:
  - Automatically deduplicates collisions on archive with `-YYYYMMDD-HHMMSS`.
  - Automatically strips the timestamp suffix upon restore when the base name is available.
- **Clipboard Integration**:
  - One-key yank for workspace path, file path, and file preview contents using native system clipboard integration.
- **Flexible Root Resolution**:
  - Operates on `./workspace` and `./archive` in the current directory, or via `--root <PATH>` / `WORK_ROOT` environment variable.

---

## Installation

### From Source

```bash
git clone https://github.com/octopus-harness/octo-work.git
cd octo-work
cargo install --path .
```

This installs both the `work` binary (primary command) and the `octo-work` binary alias into `~/.cargo/bin`.

### Development Symlink

To link the compiled release binary directly to `/usr/local/bin` during development:

```bash
./scripts/link-dev.sh
```

This builds `./target/release/octo-work` (if needed) and creates symlinks for `/usr/local/bin/work` and `/usr/local/bin/octo-work`.

---

## Quickstart

```bash
# Initialize ./workspace and ./archive directories
work init

# Create an active task workspace
work add q4-planning

# Launch the 3-pane interactive TUI dashboard
work

# List active workspaces
work list

# Move finished task to ./archive
work archive q4-planning

# Restore an archived task back to ./workspace
work restore q4-planning
```

---

## CLI Reference

| Command | Description |
| --- | --- |
| `work` / `work tui` | Launch the interactive 3-pane Ratatui TUI dashboard |
| `work init` | Initialize `./workspace` and `./archive` in the target directory |
| `work add [name]` | Create a workspace under `./workspace` (prompts if omitted) |
| `work archive [name]` | Move workspace to `./archive` (interactive picker if omitted) |
| `work restore [name] [--target <name>]` | Move workspace from `./archive` back to `./workspace` |
| `work rename <old> <new> [--archived]` | Rename a workspace |
| `work delete <name> [-y] [--archived]` | Delete a workspace and its files (prompts confirmation unless `-y`) |
| `work list` (`ls`) `[--archived|--all|--json]` | List workspaces with file counts, byte sizes, and timestamps |
| `work path [name] [--archived]` | Print the full filesystem path of a workspace |
| `work help` | Display command-line options and usage summary |

### Global Flags

- `--root <DIR>`: Override root working directory (defaults to current directory or `WORK_ROOT` env var).

---

## TUI Keyboard Shortcuts

### Global Navigation

| Key | Action |
| --- | --- |
| `1` / `2` / `3` | Focus Pane 1 (Workspaces), Pane 2 (Files), or Pane 3 (Preview) |
| `Tab` / `s` | Toggle Scope (`Active` ↔ `Archive`) |
| `/` | Real-time workspace filter / search |
| `R` | Refresh directory listings |
| `?` | Show interactive help dialog |
| `q` / `Esc` | Clear active search filter, close modal, or quit application |

### Pane 1: Workspaces

| Key | Action |
| --- | --- |
| `j` / `k` / `↑` / `↓` | Navigate workspaces |
| `Enter` / `l` / `→` | Open selected workspace (focus Pane 2: Files) |
| `a` / `n` | Add new workspace |
| `A` | Archive selected workspace (if Active) or Restore (if Archive) |
| `r` / `m` | Rename workspace |
| `d` / `x` | Delete workspace (with confirmation prompt) |
| `y` | Copy workspace path to clipboard |

### Pane 2: Files

| Key | Action |
| --- | --- |
| `j` / `k` / `↑` / `↓` | Navigate workspace files |
| `h` / `←` | Return focus to Pane 1 (Workspaces) |
| `Enter` / `l` / `→` | Move focus to Pane 3 (Preview) |
| `n` / `a` | Create a new file in the selected workspace |
| `r` / `m` | Rename file |
| `d` / `x` | Delete file |
| `e` / `E` | Open file in external `$EDITOR` (suspends and resumes TUI) |
| `y` | Copy file path to clipboard |

### Pane 3: Preview

| Key | Action |
| --- | --- |
| `j` / `k` / `d` / `u` | Scroll preview down/up |
| `v` | Toggle Raw vs. Styled Markdown rendering |
| `e` / `E` | Open file in external `$EDITOR` |
| `y` | Copy file path to clipboard |
| `Y` | Copy preview contents to clipboard |

---

## Octopus Agent Protocol Integration

In the [Octopus](https://github.com/leouofa/octopus) harness, AI agents isolate ephemeral task files inside `./workspace/<task>` and archive them upon task completion.

Because `octo-work` exposes the exact same syntax:
```bash
work add <task>
work archive <task>
```
Agents running `$$` commands can transparently manage workspaces with zero change to their system prompts or prompt templates.

---

## License

MIT License. Copyright (c) 2026 Leonid Medovyy.
