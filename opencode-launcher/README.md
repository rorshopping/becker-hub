# OpenCode Launcher

A Tauri-based desktop application combining AI assistance with terminal emulation and voice input.

## Features

- Terminal emulation via PTY
- Voice input with speech-to-text
- AI-powered assistance

## Stack

- **Framework**: Tauri 1.5 (Rust backend + Svelte frontend)
- **Frontend**: Svelte 4 + Vite 5 + xterm.js
- **Backend**: Rust with tokio, whisper-rs, portable-pty

## Development

```bash
npm install
npm run dev        # Start Vite dev server (port 5173)
npm run tauri dev  # Run Tauri in dev mode
npm run build      # Build frontend for production
npm run tauri build  # Build desktop app (exe)
```

## Window Config

- Size: 1400×900, resizable
- Output: `src-tauri/target/release/` (Windows: `.exe` in bundle)

## License

MIT