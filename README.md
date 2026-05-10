# hermes_tail 🗂️

A Total Commander-style dual-pane TUI file manager written in Rust.

## Features

- **Dual-pane layout**: Side-by-side file browsing like Total Commander
- **Keyboard-centric**: Fast navigation without mouse dependency
- **Multi-select**: Select multiple files for batch operations
- **Archive support**: Navigate and extract ZIP/TAR files
- **AI Commander**: File summarization & analysis with local LLM (llama.cpp)
- **Cross-platform**: Linux, macOS, Windows (terminal support required)

## Status

🚀 **Phase 5: Complete** - All core features implemented

✅ Dual-pane navigation & file operations
✅ Archive support (ZIP/TAR/TAR.GZ)
✅ Search & filter functionality
✅ Configuration system
✅ Async background operations & file watching

## Quick Start

```bash
# Build
cargo build

# Run
cargo run

# Release build
cargo build --release
./target/release/hermes_tail
```

## Keybindings

### Navigation
| Key | Action |
|-----|--------|
| ↑↓ | Move cursor up/down |
| Enter | Navigate into directory / archive |
| Backspace | Go to parent directory / exit archive |
| Tab | Switch active panel (left ↔ right) |

### Selection
| Key | Action |
|-----|--------|
| Insert | Toggle select current file |
| Ctrl+A | Select all files |

### Operations
| Key | Action |
|-----|--------|
| F5 | Copy / Extract from archive |
| Alt+F5 | Pack files into archive |
| F6 | Move |
| F7 | Create directory |
| F8 | Delete |
| F2 / Shift+F6 | Rename |
| Shift+F3 | View file |

### Search & Filter
| Key | Action |
|-----|--------|
| `/` | Quick search (type-to-filter) |
| `=` | Wildcard filter |
| Ctrl+F | Find files (recursive) |

### AI Commander
| Key | Action |
|-----|--------|
| Alt+G | 📝 Summarize - Brief 2-3 line summary of file content |
| Alt+S | 🔒 Security Scan - Detect sensitive info (API keys, passwords) |
| Alt+I | 🖼️ Image Metadata - Analyze image file metadata |
| Alt+C | 💻 Code Structure - Analyze code file structure & APIs |
| Alt+D | 📊 File Diff - Compare two selected files |
| Alt+A | 📁 Folder Analysis - Analyze folder structure & project type |
| ↑↓ | Scroll AI response (in AI mode) |
| PgUp/PgDn | Page up/down AI response |
| T | Toggle thinking process display (in AI mode) |
| Esc / q | Close AI response |

### System
| Key | Action |
|-----|--------|
| Ctrl+H | Toggle hidden files |
| Ctrl+L | Refresh & recalculate size |
| Ctrl+Q / Esc | Exit application |

## Development Roadmap

1. ✅ Phase 1: Dual-pane navigation
2. ✅ Phase 2: File operations (copy, move, delete, mkdir)
3. ✅ Phase 3: Search & filter
4. ✅ Phase 4: Archive support (ZIP/TAR)
5. ✅ Phase 5: Customization & polish

See [CLAUDE.md](CLAUDE.md) for detailed architecture and development guide.

## Requirements

- Rust 1.70+
- Terminal with 24-bit color support (recommended)

### AI Commander Features

AI Commander provides intelligent file analysis using a local LLM (Chain-of-Thought model):

| Feature | Usage | Use Case |
|---------|-------|----------|
| 📝 Summarize | `Alt+G` on any file | Get quick 2-3 line summary of file content |
| 🔒 Security Scan | `Alt+S` on any file | Detect API keys, passwords, PII, secrets |
| 🖼️ Image Metadata | `Alt+I` on image files | Analyze image properties and metadata |
| 💻 Code Structure | `Alt+C` on code files | View functions, APIs, dependencies, structure |
| 📊 File Diff | `Alt+D` on 2 selected files | Understand what changed and why |
| 📁 Folder Analysis | `Alt+A` on a folder | Learn project type, structure, components |

**Thinking Process**: Press `T` to toggle the AI's thinking/reasoning display

### AI Commander Requirements

To use the AI Commander feature:
- [llama.cpp](https://github.com/ggerganov/llama.cpp) running locally on port 8080
- A compatible GGUF model with Chain-of-Thought (CoT) support (tested with Qwen 35B)

#### Setup llama.cpp

```bash
# Start llama.cpp server with a CoT model
./llama-server -m models/Qwen_Qwen3.6-35B-A3B-Q4_0.gguf \
  --port 8080 \
  --ctx-size 2048
```

The AI Commander will connect to `http://localhost:8080/v1/chat/completions`

#### Supported Models

- **Qwen 35B (CoT)** - Recommended for best results
- Other OpenAI-compatible models on llama.cpp server

## License

MIT
