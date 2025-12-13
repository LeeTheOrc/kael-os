# Publishing System (Rust-Native, No External Dependencies)

Kael-OS now includes a **fully self-contained publishing system** using only Rust-native code. No Python, no `gh` CLI, no external upload tools needed.

## Architecture

### Traditional Approach (Old)
```
App → bash scripts → Python (google-cloud-storage) → Firebase
                  → gh CLI → GitHub
                  → curl → WebDAV
```

Problems: Large dependencies, slow startup, external tool requirements.

### New Rust-Native Approach
```
App → Tauri commands → Rust native code → Firebase (REST API + JWT)
                                      → GitHub (REST API v3)
                                      → WebDAV (HTTP PUT)
```

Benefits:
- ✅ Single compiled binary
- ✅ No external dependencies
- ✅ Faster execution
- ✅ Smaller download size
- ✅ Works on fresh Arch install with zero setup

## Uploaders

### 1. WebDAV (Rust Native)
**Module**: `src-tauri/src/webdav/mod.rs`

**Features**:
- HTTP PUT with basic auth
- Folder creation (MKCOL)
- File deletion
- Size-based change detection

**Tauri Command**:
```rust
webdav_upload_file(
    base_url: String,        // "https://leroyonline.co.za:2078/public_html/kael"
    username: String,        // "leetheorc"
    password: String,        // "***"
    local_path: String,      // "dist/kael-os-0.0.1-x86_64.tar.gz"
    remote_path: String      // "downloads/desktop/kael-os-0.0.1-x86_64.tar.gz"
) -> Result<String, String>
```

### 2. Firebase Storage (Rust Native)
**Module**: `src-tauri/src/firebase/uploader.rs`

**Features**:
- Google Cloud Storage REST API
- OAuth2 authentication via service account JWT
- MD5 checksum-based skip (avoids re-uploading unchanged files)
- No `google-cloud-storage` Python package needed

**Tauri Command**:
```rust
firebase_upload_file(
    bucket: String,              // "kael-os.firebasestorage.app"
    sa_json_path: String,        // "firebase-service-account.json"
    local_path: String,          // "dist/kael-os-0.0.1-x86_64.tar.gz"
    remote_path: String          // "releases/desktop/kael-os-0.0.1-x86_64.tar.gz"
) -> Result<String, String>  // Returns public HTTPS URL
```

**Authentication**: Uses service account JSON to mint a JWT token, exchanges it for an OAuth2 access token, and uploads directly to Google Cloud Storage.

### 3. GitHub Releases (Rust Native)
**Module**: `src-tauri/src/github/uploader.rs`

**Features**:
- GitHub REST API v3
- Create releases (or fetch existing)
- Upload assets with size-based skip
- Auto-detect prerelease (tags with "alpha" or "beta")
- No `gh` CLI required

**Tauri Commands**:
```rust
github_create_release(
    owner: String,               // "LeeTheOrc"
    repo: String,                // "kael-os"
    token: String,               // GitHub PAT
    tag: String,                 // "v0.0.1-alpha.1"
    name: String,                // "Kael-OS v0.0.1-alpha.1"
    body: String                 // Release notes
) -> Result<u64, String>  // Returns release ID

github_upload_asset(
    owner: String,
    repo: String,
    token: String,
    release_id: u64,
    file_path: String,           // "dist/kael-os-0.0.1-x86_64.tar.gz"
    file_name: String            // "kael-os-0.0.1-x86_64.tar.gz"
) -> Result<String, String>  // Returns public download URL
```

## Configuration

### Environment Variables
```bash
# WebDAV
WEBDAV_SERVER=leroyonline.co.za
WEBDAV_PORT=2078
WEBDAV_USERNAME=leetheorc
WEBDAV_PASSWORD=***
WEBDAV_BASE_PATH=/public_html/kael

# Firebase
FIREBASE_BUCKET=kael-os.firebasestorage.app
FIREBASE_SA_JSON=firebase-service-account.json

# GitHub
GITHUB_TOKEN=ghp_***
GITHUB_OWNER=LeeTheOrc
GITHUB_REPO=kael-os
```

### Files Required
- `firebase-service-account.json` (from Firebase Console) — must exist for Firebase uploads
- `~/.ssh/id_rsa` (optional) — for Git signing
- `gpg` binary (optional) — for detached signatures

## Usage

### Via Tauri Commands (From App UI)
The app can invoke these commands programmatically:

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Upload to WebDAV
await invoke('webdav_upload_file', {
  baseUrl: 'https://leroyonline.co.za:2078/public_html/kael',
  username: 'leetheorc',
  password: 'LeRoy0923!',
  localPath: 'dist/kael-os-0.0.1-x86_64.tar.gz',
  remotePath: 'downloads/desktop/kael-os-0.0.1-x86_64.tar.gz',
});

// Upload to Firebase
await invoke('firebase_upload_file', {
  bucket: 'kael-os.firebasestorage.app',
  saJsonPath: 'firebase-service-account.json',
  localPath: 'dist/kael-os-0.0.1-x86_64.tar.gz',
  remotePath: 'releases/desktop/kael-os-0.0.1-x86_64.tar.gz',
});

// Create GitHub release
const releaseId = await invoke('github_create_release', {
  owner: 'LeeTheOrc',
  repo: 'kael-os',
  token: 'ghp_***',
  tag: 'v0.0.1-alpha.1',
  name: 'Kael-OS v0.0.1-alpha.1',
  body: 'Alpha release',
});

// Upload assets to GitHub
await invoke('github_upload_asset', {
  owner: 'LeeTheOrc',
  repo: 'kael-os',
  token: 'ghp_***',
  releaseId,
  filePath: 'dist/kael-os-0.0.1-x86_64.tar.gz',
  fileName: 'kael-os-0.0.1-x86_64.tar.gz',
});
```

### Via Shell Scripts
```bash
# Sign release
gpg --default-key D0513E222E8EE8D7 --detach-sign dist/kael-os-0.0.1-x86_64.tar.gz

# Prepare version and build
bash scripts/bump-version.sh alpha
cargo build --release

# Publish (currently shows commands; app UI will execute them)
bash scripts/publish-rust-native.sh
```

## Bandwidth Optimization

### Firebase
- **Checksum-based skipping**: Computes MD5 of local file and compares with remote blob's `md5_hash`
- **Skip if unchanged**: Avoids uploading identical files, saving bandwidth and costs
- **Cost**: ~$0.02 per GB for storage + bandwidth

### GitHub
- **Size-based skipping**: Compares local file size with remote asset size
- **Skip if unchanged**: Avoids re-uploading assets that match
- **Cost**: Free (GitHub Releases are unlimited)

### WebDAV
- **Transparent**: Hosting provider handles caching; no app-level optimization needed

## Future Enhancements

1. **In-App UI**: Add buttons to ConfiguratorPanel for one-click publishing
2. **Progress tracking**: Show upload progress for large files
3. **Retry logic**: Handle network failures gracefully
4. **Checksum verification**: Validate uploads with SHA256 hashes
5. **Release notes generator**: Auto-populate from git commit history

## Security Notes

⚠️ **Credentials Handling**:
- Store `GITHUB_TOKEN` securely (not in source code)
- Keep `firebase-service-account.json` out of git (already in `.gitignore`)
- Never commit `WEBDAV_PASSWORD` to version control

✅ **What's Secure**:
- Service account JWT is minted per-request (short-lived)
- Firebase and GitHub APIs use HTTPS with certificate pinning
- No credentials stored in memory longer than needed
- All communication is encrypted in transit

## Dependencies

**None** beyond what's already in Cargo.toml:
- `reqwest` — HTTP client
- `serde_json` — JSON parsing
- `chrono` — timestamps
- `base64` — encoding
- `sha2`, `hmac` — JWT signing

**No external binaries required**:
- ~~`python3`~~ ❌
- ~~`google-cloud-storage`~~ ❌
- ~~`gh` CLI~~ ❌
- ~~`curl`~~ ❌ (only for testing)
- ✅ `gpg` (optional, for detached signatures)

## Troubleshooting

### Firebase 403 Forbidden
**Problem**: "AccessDenied" when uploading to Firebase Storage.

**Solution**: Update Storage Rules in Firebase Console to allow public uploads:
```
rules_version = '2';
service firebase.storage {
  match /b/{bucket}/o {
    match /releases/desktop/{allPaths=**} {
      allow read: if true;
      allow write: if request.auth != null;
    }
  }
}
```

### GitHub 422 Validation Failed
**Problem**: Release already exists or invalid tag format.

**Solution**: The uploader automatically fetches existing releases, so this is harmless. Asset upload will proceed.

### WebDAV 401 Unauthorized
**Problem**: Username or password incorrect.

**Solution**: Verify `WEBDAV_PASSWORD` and `WEBDAV_USERNAME` are set correctly. Note: WebDAV requires SSL on port 2078.
