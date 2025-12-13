# 🎉 Welcome to Kael-OS, Architect!

## Your Forge Is Ready (Rust Only)

Root:

```
/home/leetheorc/Kael-os/kael-os/
```

## ✨ What’s Ready

**Rust/Dioxus Desktop**

- Tauri config + Dioxus UI with forge palette
- SQLite + migrations
- IPC commands, terminal bridge (sync), kael personality context
- Settings/chat/panels themed with Kael sigil

**No Node Tooling**

- Build and run entirely with Cargo
- Optional Firebase sync stays stubbed/off by default

## 🚀 Quick Start

```bash
cd /home/leetheorc/Kael-os/kael-os
./setup-deps.sh   # installs platform libs
cd src-tauri
cargo run         # dev
# cargo build --release  # production
```

## 📚 Docs

- README.md – overview
- SETUP.md – system deps + run steps (Rust-only)
- MANIFEST.md – file/architecture overview
- FIREBASE.md – optional sync notes

## 🎯 Next Steps

- Wire LLM providers (Ollama/Mistral/Gemini) and chat history
- Stream PTY output to the new shell chrome
- Add settings toggles for providers/sync
- Keep Firebase optional; remain offline-first

## 💡 Architecture (Current)

```
┌────────────────────────────┐
│ Dioxus Desktop (Rust UI)   │
│ - Header / Panels / Chat   │
└────────────┬───────────────┘
             │ IPC
┌────────────▼───────────────┐
│ Tauri Runtime (Rust)       │
│ - Commands / Terminal      │
│ - Kael context / SQLite    │
│ - Firebase stub (optional) │
└────────────┬───────────────┘
      SQLite (local)   [Firebase optional]
```

## 🎨 Forge Theme

- Background `#120e1a`, Fire `#ffcc00`, Magic `#e040fb`, Steel `#3a2d56`, Text `#f7f2ff`

Welcome home, Architect. Keep forging. 🔥
