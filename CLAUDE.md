# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Use basic skill karpathy-guidelines 

---

## Project Overview

**hermes_tail** is a Total Commander-style dual-pane TUI (Terminal User Interface) file manager written in Rust using ratatui and crossterm.

**Core Features**:
- Dual-pane side-by-side file browsing
- Keyboard-centric navigation (Total Commander compatible keybindings)
- Multi-select file operations
- Archive navigation and operations (ZIP/TAR/TAR.GZ)
- Async background operations (folder size calculation)
- Cross-platform Linux/macOS/Windows support

**Current Status**: Phase 5 complete - Core functionality, archive support, configuration, and UI polishing implemented.

---

## Technology Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| UI Framework | ratatui | 0.28+ |
| Terminal Backend | crossterm | 0.28+ |
| Async Runtime | tokio | 1.0+ (with full features) |
| Archive Support | zip, tar, flate2 | Latest |
| Config | serde, toml, dirs | Latest |
| Search Support | walkdir, glob | Latest |
| Error Handling | anyhow, thiserror | Latest |

---

## Architecture Overview

### Layered Design

```
┌─────────────────────────────────────────────────┐
│         UI Rendering Layer (ratatui)            │
│  TitleBar │ LeftPanel │ RightPanel │ CmdBar    │
├─────────────────────────────────────────────────┤
│        Application State (App struct)            │
│  - left_panel: PanelState                       │
│  - right_panel: PanelState                      │
│  - active_panel: bool (left=true, right=false)  │
│  - mode: Normal|Dialog|Search|Filter            │
│  - tx/rx: Async event channel                   │
├─────────────────────────────────────────────────┤
│    Event Loop & Command Dispatch                │
│  KeyEvent → Command enum → handle_command()     │
├─────────────────────────────────────────────────┤
│   Virtual Filesystem Abstraction (VFS Trait)    │
│  - LocalFs (standard OS filesystem)             │
│  - ArchiveFs (ZIP/TAR virtual filesystem)       │
└─────────────────────────────────────────────────┘
```

### Module Structure

- **main.rs** - Application entry point and terminal setup
- **app.rs** - Main application state, event loop, command dispatch, async task management
- **fs/** - Virtual filesystem trait and implementations
  - `fs/mod.rs` - FileEntry struct, FileSystem trait, permission handling
  - `fs/local.rs` - LocalFs implementation for OS filesystem
  - `fs/archive.rs` - ArchiveFs for ZIP/TAR navigation and extraction
- **panel/** - Panel state management
  - `panel/mod.rs` - PanelState struct (path, entries, cursor, selection, sort, size tracking, filtering)
- **ui/** - Ratatui rendering and UI components
  - `ui/mod.rs` - Main render() function, layout orchestration
  - `ui/panel.rs` - Renders individual file panel with list and icons
  - `ui/cmdbar.rs` - F-key command hints bar and status information
  - `ui/statusbar.rs` - Selection info, path, and total size display
  - `ui/dialog.rs` - Input and confirmation dialogs for file operations
  - `ui/help.rs` - Interactive help popup with 2-column layout (F1 key)
  - `ui/theme.rs` - Color scheme management (Classic/Dark themes)
  - `ui/viewer.rs` - File viewer with markdown and code syntax highlighting (F3 key)
  - `ui/searchbar.rs` - Quick search and filter UI components
  - `ui/ai.rs` - AI command UI and status display
  - `ui/ai_command.rs` - AI command dialog and input handling
- **commands.rs** - Command enum for all possible actions (including async updates)
- **events.rs** - KeyEvent → Command mapping and keybinding logic
- **ops/** - File operations and search
  - `ops/mod.rs` - Module registration and operation dispatching
  - `ops/search.rs` - Recursive file search implementation
  - `ops/archive.rs` - Archive compression and extraction logic (Pack/Unpack)
- **config.rs** - Configuration management (`config.toml`, `notes.json`)
- **ai/** - AI Commander integration (llama.cpp based)
  - `ai/mod.rs` - AI feature definitions and command handling
  - `ai/client.rs` - OpenAI-compatible API client for LLM interaction
  - `ai/state.rs` - AI session state and batch operation tracking

### Key Data Structures

**FileEntry**
```rust
struct FileEntry {
    name: String,
    path: PathBuf,
    is_dir: bool,
    size: u64,
    modified: DateTime<Local>,
    permissions: u32,
}
```

**PanelState**
```rust
struct PanelState {
    path: PathBuf,
    entries: Vec<FileEntry>,
    list_total_size: u64,         // Immediate sum of current list
    recursive_total_size: Option<u64>, // Async background sum
    is_calculating: bool,         // Background task status
    // ... selection, sort, filter, fs
}
```

---

## Build & Run

```bash
# Build (debug)
cargo build

# Run
cargo run
```

### Keyboard Controls

| Key | Action |
|-----|--------|
| ↑↓ | Move cursor up/down |
| Enter | Navigate into directory / archive |
| Backspace | Go to parent directory / exit archive |
| Tab | Switch active panel (left ↔ right) |
| Insert | Toggle select current file |
| Ctrl+A | Select all files |
| Ctrl+L | Refresh current panel & recalculate size |
| Ctrl+H | Toggle hidden files |
| Ctrl+Q or Esc | Exit application |
| `/` | Quick search (type-to-filter) |
| `=` | Wildcard filter |
| Ctrl+F | Find files (recursive) |
| Alt+U | Find duplicate files |
| F5 | Copy / Extract (from archive) |
| Alt+F5 | Pack files (into archive) |
| F6 | Move |
| F7 | Create directory |
| F8 | Delete |
| F2 / Shift+F6 | Rename |

---

## Development Phases

### ✅ Phase 1: Foundation (Complete)
- [x] Dual-pane layout with borders
- [x] File listing with icons, size, date columns
- [x] Keyboard navigation, multi-select
- [x] Automatic sorting

### ✅ Phase 2: File Operations (Complete)
- [x] F5 Copy, F6 Move, F7 Mkdir, F8 Delete
- [x] F2/ShiftF6 Rename
- [x] VFS-based file operation trait methods

### ✅ Phase 3: Search & Filter (Complete)
- [x] `/` quick search, `=` wildcard filter
- [x] Ctrl+F Find files (recursive)

### ✅ Phase 4: Archive Support (Complete)
- [x] ZIP/TAR/TAR.GZ reading and navigation
- [x] F5 Extract from archive
- [x] Alt+F5 Pack files into archive

### ✅ Phase 5: Polish (Complete)
- [x] Config file: `~/.config/hermes_tail/config.toml`
- [x] Hidden file toggle (Ctrl+H)
- [x] Human-readable file sizes (K, M, G, T)
- [x] Async background folder size calculation
- [x] Interactive help (F1) with 2-column layout
- [x] Theme system (Classic/Dark)
- [x] File viewer with syntax highlighting (F3)

### 🚀 Phase 6: AI Commander (In Progress - 13/20 Features)
- [x] Ctrl+G Natural language commands
- [x] Alt+B Batch rename with AI suggestions
- [x] Alt+R README auto-generation
- [x] Alt+N File notes and tagging system
- [x] Alt+X Script generation from natural language
- [x] Alt+G File summarization
- [x] Alt+S Security scanning (sensitive info detection)
- [x] Alt+I Image metadata analysis
- [x] Alt+C Code structure analysis
- [x] Alt+D File comparison and diff
- [x] Alt+A Folder structure analysis
- [x] Ctrl+F Smart search with natural language
- [x] Alt+U Duplicate file detection
- [ ] Auto file classification
- [ ] Old file cleanup recommendations
- [ ] Storage optimization analysis
- [ ] Git change history analysis
- [ ] File operation macro learning
- [ ] Next action prediction
- [ ] Folder sync analysis

---

## AI Commander Integration

**AI Commander** is an intelligent file management system powered by llama.cpp (OpenAI-compatible API).

### Setup Requirements
1. **LLM Server**: Start llama.cpp on port 8080 (or configure in `config.toml`)
2. **Recommended Model**: Qwen 35B or similar CoT (Chain-of-Thought) models
3. **Docker Example**:
   ```bash
   docker run -d --name llama-server \
   --gpus all \
   -p 8080:8080 \
   -v ~/Programming/models:/models \
   ghcr.io/ggml-org/llama.cpp:server-cuda \
   -m /models/Qwen_Qwen3.6-35B-A3B-Q4_0.gguf \
   --n-cpu-moe 20 -c 70000
   ```

### AI Features (12/20 Completed)
- **Natural Language Commands** (Ctrl+G): Execute file operations via natural language
- **Batch Rename** (Alt+B): AI-suggested rename patterns for multiple files
- **README Generation** (Alt+R): Auto-generate project documentation
- **File Notes** (Alt+N): Persistent notes and tags (`notes.json`)
- **Script Generation** (Alt+X): Create bash scripts from instructions
- **File Summarization** (Alt+G): Quick 2-3 line content summary
- **Security Scanning** (Alt+S): Detect API keys, credentials, PII
- **Image Analysis** (Alt+I): Extract EXIF and image metadata
- **Code Analysis** (Alt+C): Analyze code structure and dependencies
- **File Comparison** (Alt+D): Explain differences between two files
- **Folder Analysis** (Alt+A): Analyze project structure and usage patterns
- **Smart Search** (Ctrl+F): Natural language based recursive search

### Implementation Notes
- All AI features are **opt-in** and degrade gracefully if LLM unavailable
- Uses OpenAI-compatible API (compatible with any llama.cpp server)
- Async execution prevents UI blocking
- Batch operations are logged in `notes.json` for persistence

---

## Configuration

Settings are stored in `~/.config/hermes_tail/config.toml`:

```toml
[ui]
color_scheme = "classic"  # or "dark"
show_hidden = false
default_sort = "name"     # "name", "size", "date", "ext"
confirm_delete = true

[ai]
enabled = true
server_url = "http://localhost:8080"
model = "default"
timeout_seconds = 60

[behavior]
auto_refresh = true
```

Notes and file metadata are stored in `~/.config/hermes_tail/notes.json`.

---

## Performance Notes

- **Background Calculation**: Folder sizes are calculated in a separate tokio task using `WalkDir`. It uses `yield_now()` to prevent UI stuttering.
- **Refresh Optimization**: Immediate size sum uses memory-cached entries for zero-latency feedback.
- **Archive Navigation**: Uses in-memory index for fast browsing; extraction is streamed.

---

## References

- ratatui examples: https://github.com/ratatui-org/ratatui/tree/main/examples
- Total Commander keybindings: https://www.ghisler.com/history.htm
