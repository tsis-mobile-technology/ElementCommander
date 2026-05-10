# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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

- **app.rs** - Main application state, event loop, command dispatch, async task management
- **fs/** - Virtual filesystem trait and implementations
  - `fs/mod.rs` - FileEntry struct, FileSystem trait
  - `fs/local.rs` - LocalFs implementation for OS filesystem
  - `fs/archive.rs` - ArchiveFs for ZIP/TAR navigation and extraction
- **panel/** - Panel state management
  - `panel/mod.rs` - PanelState struct (path, entries, cursor, selection, sort, size tracking)
- **ui/** - Ratatui rendering
  - `ui/mod.rs` - Main render() function, layout orchestration
  - `ui/panel.rs` - Renders individual file panel with list
  - `ui/cmdbar.rs` - F-key command hints bar
  - `ui/statusbar.rs` - Selection info, path, and total size display
  - `ui/dialog.rs` - Input and confirmation dialogs for ops
- **commands.rs** - Command enum for all possible actions (including async updates)
- **events.rs** - KeyEvent → Command mapping
- **ops/** - File operations and search
  - `ops/mod.rs` - Module registration
  - `ops/search.rs` - Recursive file search implementation
  - `ops/archive.rs` - Compression logic (Pack)
- **config.rs** - Configuration management (`config.toml`)

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
- [x] **New**: Async background folder size calculation

---

## Performance Notes

- **Background Calculation**: Folder sizes are calculated in a separate tokio task using `WalkDir`. It uses `yield_now()` to prevent UI stuttering.
- **Refresh Optimization**: Immediate size sum uses memory-cached entries for zero-latency feedback.
- **Archive Navigation**: Uses in-memory index for fast browsing; extraction is streamed.

---

## References

- ratatui examples: https://github.com/ratatui-org/ratatui/tree/main/examples
- Total Commander keybindings: https://www.ghisler.com/history.htm
