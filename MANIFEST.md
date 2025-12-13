# 🔥 Kael-OS Manifest (Rust/Dioxus)

## Stack

- Rust + Tauri + Dioxus Desktop UI (forge palette + Kael sigil)
- SQLite with migrations
- Optional Firebase sync (stub/off by default)
- No Node/npm/pnpm required

## Run

```bash
cd /home/leetheorc/Kael-os/kael-os
./setup-deps.sh     # platform libs
cd src-tauri
cargo run           # dev
# cargo build --release  # production
```

## Key Paths

- src-tauri/src/main.rs — Tauri/Dioxus entry
- src-tauri/src/commands.rs — IPC/handlers
- src-tauri/src/state.rs — shared types
- src-tauri/src/components/ — UI (header, chat, panels, settings)
- src-tauri/src/terminal/ — shell bridge (sync exec)
- src-tauri/src/kael/ — personality/system context
- src-tauri/src/firebase/ — optional sync stub
- sql/ — database migrations

## UI Shape

- Header: forge bar with settings dropdown
- Left panel: sigil tile, quick actions, pinned panels
- Chat: Kael + terminal-aware bubbles with shell chrome
- Right panel: status cards (providers/build/runtime)
- Settings: provider toggles, system info

## Roadmap

- Stream PTY output to UI; preserve Arch/paru translation
- Wire LLM providers (Ollama/Mistral/Gemini) + history persistence
- Add settings toggles for providers/sync
- Keep Firebase optional; remain offline-first
- Package via `cargo tauri build --release`

## Notes

- Legacy web assets remain but are unused; build uses only `src-tauri`
- Keep secrets in `.env.local`; never commit

Forge stays Rust-native. Keep building. 🔥
