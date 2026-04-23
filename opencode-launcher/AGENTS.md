# AGENTS.md

## Stack

- **Framework**: Tauri 1.5 (Rust backend + Svelte frontend)
- **Frontend**: Svelte 4 + Vite 5 + xterm.js
- **Backend**: Rust with tokio, whisper-rs, portable-pty

## Developer Commands

```bash
npm run dev        # Start Vite dev server (port 5173)
npm run tauri dev  # Run Tauri in dev mode
npm run build      # Build frontend for production
npm run tauri build  # Build desktop app (exe)
```

## Project Structure

| Directory | Purpose |
|-----------|---------|
| `src/` | Svelte UI components (`lib/`) and stores |
| `src-tauri/src/` | Rust backend: `main.rs`, `whisper_engine.rs`, `pty_manager.rs`, `ai_dictionary.rs` |

## Architecture Notes

- Frontend runs on Vite dev server (port 5173), embedded in Tauri WebView window
- PTY management via `portable-pty` crate for terminal emulation
- Voice input piped to `whisper-rs` for speech-to-text
- Tauri window config: 1400×900, resizable

## Build Output

- Desktop exe built via `npm run tauri build`
- Output in `src-tauri/target/release/` (Windows: `.exe` in bundle)

## Setup (Windows)

First-time setup requires installing these tools:

| Tool | Purpose | Install |
|------|---------|---------|
| Rust | Language runtime | `iwr https://win.rustup.rs -OutFile rustup-init.exe; .\rustup-init.exe` |
| VS Build Tools | C++ linker (`link.exe`) | [Download](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio), select "Desktop development with C++" |
| LLVM | `libclang.dll` for bindgen | `winget install LLVM.LLVM` |
| CMake | Build whisper.cpp | `winget install Kitware.CMake` |

After installing, restart PowerShell. Icons can be auto-generated:

```bash
npx tauri icon src-tauri/icons/128x128.png  # creates all required icon formats
```