# Kael-OS Setup Guide

Welcome to the Kael-OS Forge, Architect! Pure Rust/Dioxus desktop—no npm/pnpm required.

## ⚙️ Prerequisites

### System Requirements

- **Rust**: latest stable (https://rustup.rs/)

### On Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install -y libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

### On Fedora

```bash
sudo dnf install openssl-devel gtk3-devel appindicator-gtk3-devel librsvg2-devel
```

### On Arch

```bash
sudo pacman -S openssl gtk3 libappindicator-gtk3 librsvg
```

### On macOS

```bash
brew install openssl
```

## 🚀 Quick Start

### 1. Navigate to Project

```bash
cd /home/leetheorc/Kael-os/kael-os
```

### 2. Run in Development (Rust-only)

```bash
cd src-tauri
cargo run
```

### 3. Build for Production

```bash
cd src-tauri
cargo build --release
```

Built executables will be in `src-tauri/target/release/bundle/`

## 📁 Project Structure

```
kael-os/
├── src-tauri/              # Rust app (Dioxus + Tauri)
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── commands.rs     # IPC commands
│   │   ├── state.rs        # Type definitions
│   │   ├── db/             # Database module
│   │   ├── terminal/       # Terminal module
│   │   ├── kael/           # Kael-AI module
│   │   └── firebase/       # Firebase module
│   ├── Cargo.toml
│   └── tauri.conf.json
├── sql/                    # Database migrations
```

## 🔧 Key Features

### Desktop Components (Dioxus)

- **Header**: Forge bar + settings dropdown
- **Chat**: Kael convo with terminal-aware output chrome
- **Panels**: Left/Right forge cards and status blocks

### Backend Modules

- **db**: SQLite database with migrations
- **commands**: IPC handlers for UI
- **state**: Shared types (messages/config)
- **terminal**: Shell exec (sync for now)
- **kael**: AI personality/system context
- **firebase**: Optional cloud sync (stub)

### Services

- Rust-native modules; legacy TS services are deprecated

### Styling

- Forge palette embedded via inline styles in Rust components

## 📡 IPC Commands

The Dioxus desktop app communicates directly; IPC remains available for modularity.

## 🗄️ Database

SQLite database located at (platform-specific):

- **Linux**: `~/.config/kael-os/kael.db`
- **macOS**: `~/Library/Application Support/kael-os/kael.db`
- **Windows**: `%APPDATA%\kael-os\kael.db`

Tables:

- `chat_messages`: id, role, text, timestamp, synced
- `scripts`: id, name, content, created_at, updated_at
- `kael_config`: key, value pairs

## 🔐 Firebase (Optional)

We support optional sync via Firebase. Minimal steps:

1. Create a Firebase project.
2. Keep `.env.local` with your keys. Use the docs in the repo for provider setup.
3. When ready, wire the Firebase module in `src-tauri/src/firebase/` and enable the toggle in the settings.

## 🐛 Troubleshooting

### Build Fails with WebKit Errors

Ensure you have all system dependencies installed (see Prerequisites section)

### Database Permission Errors

- Check directory permissions
- Ensure database directory exists

### IPC Command Not Found

- Verify command is registered in `src-tauri/src/main.rs`
- Check command name matches frontend invoke call

## 📚 Development Tips

### Hot Reload

- Dioxus desktop supports fast rebuilds; Rust changes recompile

### Debugging

- Desktop: Use DevTools if enabled; otherwise check Tauri logs

### Adding New Commands

1. Create function in `src-tauri/src/commands.rs`
2. Register it in `main.rs`
3. Call it internally from Dioxus components or via IPC

### Adding New Dependencies

- **Backend**: Update `src-tauri/Cargo.toml`

## 🚢 Deployment

### Creating Release

```bash
cd src-tauri
cargo tauri build --release
```

### Distribution

Built binaries are in:

- Linux: `src-tauri/target/release/bundle/deb/`
- macOS: `src-tauri/target/release/bundle/dmg/`
- Windows: `src-tauri/target/release/bundle/msi/`

## 📖 Further Reading

- [Tauri Docs](https://tauri.app/docs/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [SQLite Docs](https://www.sqlite.org/docs.html)

## 🎯 Next Steps

1. ✅ Set up local development environment
2. ⬜ Integrate Firebase authentication (optional)
3. ⬜ Stream terminal PTY output to UI
4. ⬜ Wire LLM providers (Ollama/Mistral/Gemini)
5. ⬜ Build VM integration
6. ⬜ Create differential updater

---

**Welcome to the Forge, Architect!** The foundation is set. Now let's build something legendary! 🔥
