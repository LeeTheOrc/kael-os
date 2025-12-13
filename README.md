# Kael-OS

AI-native desktop forge built in Rust with Dioxus + Tauri. No Node/npm/pnpm required.

## Features

- **Rust + Dioxus Desktop**: Native UI with forge palette and Kael sigil
- **Kael Personality**: System context + laws baked into Rust backend
- **Terminal Bridge**: Local shell exec with Arch/paru translator text
- **Offline-First**: SQLite local store; Firebase sync remains optional
- **Modular Panels**: Header, chat, left/right sidebars, settings
- **Pure Rust Build**: No Node toolchain needed

## Quick Start

### Prerequisites

- Rust (via rustup)
- Platform libs for Tauri/WebKit (see SETUP.md)

### Run (dev)

```bash
cd src-tauri
cargo run
```

### Firebase OAuth Setup (Optional)

For Google and GitHub sign-in support:

1. Copy `.env.example` to `.env.local`
2. Add your Firebase project credentials
3. See [FIREBASE.md](./FIREBASE.md) for integration notes

Without Firebase configured, the app stays entirely local.

### Build

```bash
cd src-tauri
cargo build --release
```

## Project Structure

- `src-tauri/`: Rust app (Dioxus Desktop + Tauri + SQLite)
- `sql/`: Database migrations

## Architecture

### UI (Dioxus)

- Component-based UI in Rust (`src-tauri/src/components`)
- Forge palette + Kael sigil baked into components

### Backend (Rust/Tauri)

- IPC commands for UI
- SQLite migrations
- Modules: db, terminal, kael, firebase, api
- Async/await with Tokio

## Technologies

- **UI**: Dioxus Desktop
- **Core**: Tauri 2.x, Rust, SQLite, Tokio

## Roadmap

- **Chat/LLM**: Wire streaming + history (SQLite)
- **Terminal**: Stream PTY output to UI; keep Arch translations
- **Settings**: Provider toggles + theme slots
- **Sync**: Optional Firebase; stays local by default
- **Packaging**: Cross-platform bundles via `cargo build --release`

## License

MIT
