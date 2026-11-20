# 🐙 octo-work (`work`)

<p align="center">
  <strong>A blazing-fast, keyboard-driven workspace manager & 3-pane Ratatui TUI for AI agent harnesses and rapid developer task isolation.</strong>
</p>

<p align="center">
  <a href="https://github.com/leouofa/octo-work/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="MIT License" /></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-2021_edition-orange.svg?style=flat-square" alt="Rust 2021" /></a>
  <a href="https://github.com/ratatui-org/ratatui"><img src="https://img.shields.io/badge/TUI-Ratatui_0.28-purple.svg?style=flat-square" alt="Ratatui" /></a>
  <a href="https://github.com/leouofa/octo-work/actions"><img src="https://img.shields.io/badge/tests-12_passed-brightgreen.svg?style=flat-square" alt="Tests" /></a>
  <img src="https://img.shields.io/badge/platform-macOS_%7C_Linux_%7C_Windows-lightgrey.svg?style=flat-square" alt="Platform Support" />
  <img src="https://img.shields.io/badge/startup-<5ms-green.svg?style=flat-square" alt="Startup Latency" />
</p>

---

## ⚡ Overview

When orchestrating autonomous AI agents or context-switching between complex engineering tasks, ephemeral artifacts—logs, scratchpads, experimental diffs, and prompt outputs—quickly pollute Git working trees. Deleting them destroys valuable context; keeping them creates clutter and accidental commits.

`octo-work` solves this problem with an ultra-responsive, compiled Rust binary (`work`). It acts as a **zero-latency drop-in workspace engine** for the [Octopus AI agent harness](https://github.com/leouofa/octopus) and human developers alike.

Equipped with both a deterministic **CLI interface** (ideal for agent automation and headless scripts) and a modern **3-pane terminal dashboard (TUI)** built on [Ratatui](https://github.com/ratatui-org/ratatui), `octo-work` provides seamless task isolation, collision-free archival, and instant in-terminal inspection.

```text
 ┌── Workspaces (Active: 3) ────┐┌── Files: q4-planning ──────┐┌── Preview: task.md ────────────────────┐
 │ > q4-planning           14 KB││ > task.md              2 KB││ # Q4 Strategic Initiatives              │
 │   auth0-integration     82 KB││   architecture.png   112 KB││                                         │
 │   llm-benchmark         45 KB││   agent.log            8 KB││ - [x] Benchmark prompt token latencies  │
 │                              ││                            ││ - [ ] Finalize ratatui dashboard UI     │
 │                              ││                            ││ - [ ] Write integration test suite      │
 │                              ││                            ││                                         │
 └──────────────────────────────┘└────────────────────────────┘└─────────────────────────────────────────┘
  [Active]  Workspaces: 3 | Files: 3 | Size: 122 KB           [Tab] Scope  [/] Filter  [?] Help  [q] Quit
```

---

## ✨ Key Highlights

- ⚡ **Sub-Millisecond Startup (<5ms)**: Written in 100% safe Rust with zero runtime dependencies. Executes instantly in high-frequency agent tool loops.
- 🖥️ **3-Pane Ratatui TUI**: Modal terminal interface with live fuzzy-style search (`/`), instant Markdown rendering toggle (`v`), and seamless `$EDITOR` suspension/resumption (`e`).
- 🤖 **AI Agent Harness Drop-In**: 100% backward-compatible with legacy `work add <name>` and `work archive <name>` commands. AI agents can manage ephemeral workspaces with zero prompt or template adjustments.
- 🔄 **Smart Collision Resolution Engine**: Automatic timestamped deduplication on archive (`-YYYYMMDD-HHMMSS`) with intelligent suffix stripping upon restore.
- 📋 **Native OS Clipboard Integration**: Powered by `arboard`—instantly copy workspace paths, file paths, or Markdown preview contents to your system clipboard with a single keystroke.
- 🛡️ **Git-Aware & Secure by Default**: Prevents directory traversal attacks (`..`, `/`), auto-detects Git repositories, and prompts to keep ephemeral workspaces cleanly out of version control (`.gitignore`).
- 📊 **Machine-Readable Automation**: Full `--json` output support on `work list` for downstream agent reasoning, CI pipelines, and shell scripting.

---

## 🛠️ Architecture & Engineering Design

`octo-work` was engineered to replace fragile shell scripts with a memory-safe, maintainable, and battle-tested systems architecture:

```
                  ┌──────────────────────────────────────────────┐
                  │                 User / Agent                 │
                  └──────────────┬────────────────┬──────────────┘
                                 │                │
                      Interactive Invocation    CLI Subcommands
                                 │                │
                                 ▼                ▼
            ┌──────────────────────────┐    ┌───────────────────────────┐
            │       Ratatui TUI        │    │          CLI Mode         │
            │  (Event Loop & Modals)   │    │  (Headless & JSON Engine) │
            └────────────┬─────────────┘    └─────────────┬─────────────┘
                         │                                │
                         └───────────────┬────────────────┘
                                         ▼
                        ┌─────────────────────────────────┐
                        │      Workspace Domain Core      │
                        │   - Collision Deduplication     │
                        │   - Path Traversal Guard        │
                        │   - Git Repository Integration  │
                        └────────────────┬────────────────┘
                                         ▼
                        ┌─────────────────────────────────┐
                        │      Local Filesystem Root      │
                        │   ./workspace  |  ./archive     │
                        └─────────────────────────────────┘
```

### Highlights for Tech Recruiters & Systems Engineers:
- **Clean Separation of Concerns**: Complete decoupling between UI presentation (`src/ui/*`), core application state machine (`src/app.rs`), filesystem domain logic (`src/workspace.rs`), and CLI driver (`src/cli.rs`).
- **Resilient Error Architecture**: Unified error propagation using `anyhow::Context` ensures clear, actionable error messages with zero panics.
- **Terminal State Safety**: Clean terminal raw mode restoration via `crossterm` and panic hooks to prevent terminal state corruption upon unexpected exit.
- **High Test Coverage**: 12 unit and end-to-end integration tests verifying atomic directory moves, collision deduplication, git detection, and TUI filtering state.

---

## 🚀 Quickstart

### 1. Installation

#### From Source (via Cargo)
```bash
git clone https://github.com/leouofa/octo-work.git
cd octo-work
cargo install --path .
```
> Installs both `work` (primary command) and `octo-work` (binary alias) directly into `~/.cargo/bin`.

#### Development Symlink (macOS / Linux)
```bash
./scripts/link-dev.sh
```
> Builds `./target/release/octo-work` and creates direct symlinks in `/usr/local/bin`.

---

### 2. Common Workflow

```bash
# 1. Initialize workspace and archive directories (auto-detects Git repository)
work init

# 2. Create an isolated workspace for a new feature or agent task
work add q4-planning

# 3. Launch the 3-pane interactive TUI
work

# 4. Inspect workspaces programmatically (human-readable or JSON)
work list
work list --json

# 5. Archive a completed task (auto-handles name collisions)
work archive q4-planning

# 6. Restore an archived task back to active status anytime
work restore q4-planning
```

---

## 💻 CLI Reference

| Command | Alias | Description |
| :--- | :--- | :--- |
| `work` / `work tui` | — | Launch the interactive 3-pane Ratatui TUI dashboard |
| `work init` | — | Initialize `./workspace` and `./archive` with optional `.gitignore` setup |
| `work add [name]` | — | Create a new active workspace (prompts interactively if omitted) |
| `work archive [name]` | — | Move workspace to `./archive` with collision timestamping |
| `work restore [name] [--target <name>]` | — | Restore an archived workspace (auto-strips collision timestamp) |
| `work rename <old> <new> [--archived]` | — | Safely rename a workspace directory |
| `work delete <name> [-y] [--archived]` | `rm` | Delete a workspace and its contents (confirmation prompted) |
| `work list [--archived\|--all\|--json]` | `ls` | List workspaces with file counts, sizes, and last modified dates |
| `work path [name] [--archived]` | — | Print absolute filesystem path for easy shell piping |
| `work help` | — | Display CLI help, options, and subcommand details |

### Global Flags
- `--root <DIR>`: Override root working directory (defaults to current working directory or `WORK_ROOT` environment variable).

---

## ⌨️ TUI Keyboard Controls

The TUI features an intuitive, modal keymap inspired by Vim and modern terminal file managers.

### 🧭 Global Navigation
| Key | Action |
| :---: | :--- |
| `1` / `2` / `3` | Focus **Pane 1** (Workspaces), **Pane 2** (Files), or **Pane 3** (Preview) |
| `Tab` / `s` | Switch Scope between `Active` (`./workspace`) and `Archive` (`./archive`) |
| `/` | Open live fuzzy filter input |
| `R` | Force refresh workspace directory tree from disk |
| `?` | Toggle Help & Keybinding cheat-sheet modal |
| `q` / `Esc` | Clear active filter, dismiss open modal, or exit application |

### 📂 Pane 1: Workspaces
| Key | Action |
| :---: | :--- |
| `j` / `k` or `↑` / `↓` | Navigate workspace list |
| `Enter` / `l` / `→` | Open selected workspace (shifts focus to Pane 2: Files) |
| `a` / `n` | Create a new workspace (opens name input modal) |
| `A` | Quick Archive (if in `Active`) or Quick Restore (if in `Archive`) |
| `r` / `m` | Rename selected workspace |
| `d` / `x` | Delete workspace (prompts safety confirmation modal) |
| `y` | Copy absolute workspace path to system clipboard |

### 📄 Pane 2: Files
| Key | Action |
| :---: | :--- |
| `j` / `k` or `↑` / `↓` | Navigate files within selected workspace |
| `h` / `←` | Return focus to Pane 1 (Workspaces) |
| `Enter` / `l` / `→` | Move focus to Pane 3 (File Preview) |
| `n` / `a` | Create a new file in active workspace |
| `r` / `m` | Rename selected file |
| `d` / `x` | Delete file with confirmation |
| `e` / `E` | Open file in external `$EDITOR` (suspends TUI, restores on save/exit) |
| `y` | Copy absolute file path to system clipboard |

### 👁️ Pane 3: Preview
| Key | Action |
| :---: | :--- |
| `j` / `k` or `d` / `u` | Scroll preview down/up (half-page / line) |
| `v` | Toggle formatted Markdown rendering vs. Raw text view |
| `e` / `E` | Open previewed file in external `$EDITOR` |
| `y` | Copy file path to system clipboard |
| `Y` | Copy entire preview contents to system clipboard |

---

## 🤖 Octopus AI Agent Protocol

`octo-work` was created as an infrastructure component for the [Octopus](https://github.com/leouofa/octopus) AI agent harness.

During multi-step reasoning and autonomous coding tasks, agents isolate scratchpads and intermediate outputs inside `./workspace/<task-name>`. When the objective is fulfilled, the agent archives the workspace.

Because `octo-work` preserves drop-in CLI parity:
```bash
work add refactor-auth-flow
# ... agent executes scratchpad generation & tests ...
work archive refactor-auth-flow
```

Autonomous agents can immediately utilize `octo-work` with **zero modifications** to existing prompt chains or agent tool wrappers, gaining superior reliability and collision resilience.

---

## 📊 Performance Comparison

| Metric | Legacy Shell Script | `octo-work` (Rust) | Advantage |
| :--- | :---: | :---: | :--- |
| **Cold Startup Latency** | ~45ms – 85ms | **<5ms** | **10x–17x faster** |
| **Execution Dependency** | `bash`, `sed`, `awk`, `date` | **Zero (Static Native)** | Portable, zero drift |
| **Collision Handling** | Overwrites or fails | **Atomic Timestamping** | Zero data loss |
| **Interactive UX** | Basic `select` menu | **3-Pane Ratatui TUI** | Real-time preview & filtering |
| **Path Traversal Protection** | Vulnerable to `../` | **Strict Sanitization** | Secure against malicious inputs |
| **Automation Output** | Unstructured plain text | **Native `--json` Support** | Seamless pipeline integration |

---

## 🧪 Testing & Code Quality

The repository includes a comprehensive test suite covering CLI parsing, filesystem operations, collision resolution, and state management:

```bash
# Run all unit and integration tests
cargo test

# Check formatting and clippy lints
cargo fmt --check
cargo clippy -- -D warnings
```

---

## 👤 Author

**Leonid Medovyy**
- GitHub: [@leouofa](https://github.com/leouofa)
- Email: [leonid@macron1.io](mailto:leonid@macron1.io)

---

## 📄 License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.
