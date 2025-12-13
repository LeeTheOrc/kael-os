# Firebase Integration (Condensed)

This app supports optional cloud sync with Firebase while remaining fully functional offline.

## What You Need

- A Firebase project (Firestore + Authentication if desired)
- Local env file: `.env.local` with your keys

Example `.env.local` entries:

```
FIREBASE_API_KEY=your_api_key
FIREBASE_PROJECT_ID=your_project_id
FIREBASE_DB_URL=your_db_url
FIREBASE_AUTH_DOMAIN=your_auth_domain
```

## How It Works

- Local-first data persists in SQLite.
- When enabled, a background sync process mirrors selected tables to Firestore.
- Auth can be Google/GitHub; you remain the owner/operator.

## Enable Sync (Roadmap)

1. Put keys in `.env.local`.
2. Implement wiring in `src-tauri/src/firebase/`.
3. Add a toggle in the settings panel to enable/disable sync.

## Security Notes

- You are in full control; nothing syncs unless enabled.
- System/terminal commands run locally; never sent to Firebase.
- Credentials and secrets should be stored securely and never committed.

## Troubleshooting

- Check network connectivity and Firestore rules.
- Review Tauri logs for sync status.
- Validate `.env.local` keys are correct and loaded.

## Next Steps

- Add selective sync per table.
- Add conflict resolution and merge strategies.
- Provide export/import tools (JSON/SQL) for backups.
