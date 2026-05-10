# hermes_tail 🗂️

A Total Commander-style dual-pane TUI file manager written in Rust, supercharged with AI automation.

## Features

- **Dual-pane layout**: Side-by-side file browsing like Total Commander
- **Keyboard-centric**: Fast navigation and operations without mouse dependency
- **AI Commander**: Intelligent file analysis, automation, and search with local LLM (llama.cpp)
- **Archive support**: Seamlessly navigate and extract ZIP/TAR/TAR.GZ files
- **Customizable**: Built-in Theme system (Classic/Dark) and rich configuration
- **Optimized**: High performance with async operations and event-driven rendering

## Status

🚀 **Phase 5: Complete** - All core features implemented
🤖 **AI Commander: In Progress** - **12/20** features implemented (**60%**)

**Key Milestones (2026-05-10):**
- ✅ **Smart Search**: Natural language based recursive file searching
- ✅ **README Generation**: Auto-generate project documentation using AI
- ✅ **File Notes**: Persistent AI-powered notes and tagging for files
- ✅ **Scripting**: Generate executable bash scripts from natural language
- ✅ **Resource Polish**: CPU usage optimized (~0% idle), Terminal recovery improved

## Keybindings (Shortcuts)

Press **F1** anytime within the app to see the interactive help.

### Navigation & Selection
| Key | Action |
|-----|--------|
| **F1** | 💡 **Show Help (Help Popup)** |
| ↑ / ↓ | Move cursor up/down |
| Enter | Navigate into directory / Open archive |
| Backspace | Go to parent directory / Exit archive |
| Tab | Switch active panel (Left ↔ Right) |
| PgUp / PgDn | Page up / Page down |
| **Insert** | Toggle select current item |
| **Ctrl + A** | Select all items |
| **Esc** | Clear selection / Cancel search |

### File Operations
| Key | Action |
|-----|--------|
| **F3** | View file (with Markdown & Code highlighting) |
| **F5** | Copy / Extract from archive |
| **Alt + F5** | Pack files into ZIP archive |
| **F6** | Move |
| **F2 / S+F6** | Rename current item |
| **F7** | Create new directory (Mkdir) |
| **F8** | Delete (with confirmation) |

### Search & Filter
| Key | Action |
|-----|--------|
| **/** | Quick search (Type-to-filter active panel) |
| **=** | Wildcard filter |
| **Ctrl + F** | 🔍 **Find files** (Recursive search, AI Smart Search supported) |

### AI Commander (The Intelligent Core)
| Key | Action | Description |
|-----|--------|-------------|
| **Ctrl + G** | 🤖 **Natural Command** | Interpret natural language and execute operations (e.g., "Delete logs older than 3 months") |
| **Alt + B** | ✏️ **Batch Rename** | Rename multiple files with AI-suggested patterns |
| **Alt + R** | 📝 **README Gen** | Auto-generate README.md by analyzing project structure & code |
| **Alt + N** | 🔖 **File Note** | Add persistent notes/tags to files (stored in `notes.json`, marked with `[N]`) |
| **Alt + X** | 📜 **Script Gen** | Generate executable bash scripts from natural language instructions |
| **Alt + G** | 📄 **Summarize** | Brief 2-3 line summary of file content |
| **Alt + S** | 🔒 **Security Scan** | Detect sensitive info (API keys, secrets, PII) |
| **Alt + I** | 🖼️ **Image Info** | Analyze image properties and EXIF metadata |
| **Alt + C** | 💻 **Code Analysis** | Analyze code structure, APIs, and dependencies |
| **Alt + D** | 📊 **File Diff** | Compare two selected files and explain differences |
| **Alt + A** | 📁 **Folder Analysis** | Analyze folder structure and project usage |

### System
| Key | Action |
|-----|--------|
| **Ctrl + Q** / **F10** | Exit application |
| **Ctrl + H** | Toggle hidden files |
| **Ctrl + L** | Force refresh panels |

## Quick Start

```bash
# Build
cargo build

# Run
cargo run

# Create release version
cargo build --release
./target/release/hermes_tail
```

### AI Commander Requirements

To use the AI features:
1. Start a `llama.cpp` server (or any OpenAI-compatible server) on port 8080.
2. Recommended model: **Qwen 35B** or similar CoT (Chain-of-Thought) models.

```bash
docker run -d --name llama-server \
--gpus all \
--cap-add IPC_LOCK \
--ulimit memlock=-1:-1 \
-p 8080:8080 \
-v ~/Programming/models:/models \
ghcr.io/ggml-org/llama.cpp:server-cuda \
-m /models/Qwen_Qwen3.6-35B-A3B-Q4_0.gguf \
--n-cpu-moe 20 \
--no-mmap \
--cache-type-k q4_0 \
--cache-type-v q4_0 \
--mlock \
-c 70000

```

## Configuration

Settings are stored in `~/.config/hermes_tail/config.toml`:
- **Theme**: `color_scheme = "classic"` or `"dark"`
- **Behavior**: Confirm delete, default sort, etc.
- **Notes**: File annotations are stored in `notes.json`.

## License

MIT
