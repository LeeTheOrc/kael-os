# Kael-OS OAuth Notes (Optional)

Current state: OAuth is **not wired** into the Rust/Dioxus app. Kael-OS runs offline-first with no login flow by default. Firebase remains optional.

## If/when enabling Firebase OAuth

- Implement provider wiring in `src-tauri/src/firebase/` and surface a toggle in the settings panel.
- Keep all secrets in `.env.local` (see FIREBASE.md for variable names).
- Preferred providers: Google, GitHub.

## Guidelines

- Stay offline-first: never block the app if Firebase is absent.
- Cache sessions locally if implemented; expire gracefully.
- Do not ship any Node-based auth UI; keep everything Rust/Dioxus.

## Next steps (when prioritized)

1. Add OAuth command handlers in Rust and expose to the Dioxus UI.
2. Store minimal profile/cache locally; guard secrets.
3. Make provider toggles optional and default-off.
