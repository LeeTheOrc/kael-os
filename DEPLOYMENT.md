# Kael-OS Multi-Platform Deployment & Distribution Architecture

## 🌍 Infrastructure Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Global Distribution Network                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │  cPanel Web  │  │   Firebase   │  │ GitHub Pages │           │
│  │   Hosting    │  │   Hosting    │  │  + Releases  │           │
│  │ (Let's Encr) │  │   Storage    │  │   (CDN)      │           │
│  └──────────────┘  └──────────────┘  └──────────────┘           │
│         │                 │                    │                │
│         └─────────────────┴────────────────────┘                │
│                        ▼                                         │
│         ┌────────────────────────────────┐                      │
│         │   Update Server / App Store    │                      │
│         │  (Multi-Mirror Fallback)       │                      │
│         └────────────────────────────────┘                      │
│                        ▼                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │   Desktop    │  │    Mobile    │  │   Repos      │           │
│  │ (Windows,    │  │   (Android   │  │  (Packages   │           │
│  │  Linux, Mac) │  │   iOS later) │  │   & Mirrors) │           │
│  └──────────────┘  └──────────────┘  └──────────────┘           │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## 📦 Mirror Distribution Strategy

### Primary Mirrors (in priority order):
1. **GitHub Releases** (kael-os releases, built-in CDN)
2. **Firebase Hosting** (cloud-native, real-time sync)
3. **cPanel Web Hosting** (your.domain.com, Let's Encrypt SSL, webdisk)
4. **Google Cloud Storage** (Arch AUR mirror fallback)

### Mirror URLs:
```
Primary:   https://github.com/LeeTheOrc/kael-os/releases/download/v{version}/
Secondary: https://kael-os.web.app/releases/v{version}/
Tertiary:  https://yourdomain.com/kael-os/releases/v{version}/
Fallback:  https://storage.googleapis.com/kael-os-releases/v{version}/
```

### Repository Mirrors (Arch Packages):
```
[core]
Server = https://github.com/LeeTheOrc/kael-os-repo/releases/download/core/$arch
Server = https://kael-os.web.app/repos/core/$arch
Server = https://yourdomain.com/repos/core/$arch

[extra]
Server = https://github.com/LeeTheOrc/kael-os-repo/releases/download/extra/$arch
Server = https://kael-os.web.app/repos/extra/$arch
Server = https://yourdomain.com/repos/extra/$arch
```

## 🔐 SSL/TLS Configuration

### Your Current Setup:
- ✅ Let's Encrypt SSL on cPanel (yourdomain.com)
- ✅ Firebase Hosting SSL (kael-os.web.app) - automatic
- ✅ GitHub HTTPS - automatic
- ✅ Google Cloud HTTPS - automatic

### Implementation Steps:
1. Generate certificate pinning for Let's Encrypt:
   ```bash
   openssl x509 -in /path/to/cert.pem -pubkey -noout | openssl pkey -pubin -outform der | openssl dgst -sha256 -binary | base64
   ```

2. Add to app config:
   ```rust
   const PINNED_CERTIFICATES: &[&str] = &[
       // yourdomain.com Let's Encrypt cert
       "YOUR_CERT_PIN_HERE",
       // Firebase Hosting backup pin
       "FIREBASE_CERT_PIN_HERE",
   ];
   ```

3. Implement certificate validation in reqwest client:
   ```rust
   let client = reqwest::Client::builder()
       .danger_accept_invalid_certs(false)
       .build()?;
   ```

## 📱 Platform-Specific Distribution

### Windows
- **Format**: `.msi` (Windows Installer)
- **Distribution**: GitHub Releases + cPanel webdisk
- **Auto-Update**: Check `https://yourdomain.com/kael-os/version.json`
- **Install Location**: `C:\Program Files\Kael-OS`

### Linux
- **Format**: `.AppImage` (portable) + `.rpm` (Fedora) + `.deb` (Debian)
- **Distribution**: GitHub Releases + Arch AUR
- **Auto-Update**: `pacman -S kael-os` (from your repos)
- **Install Location**: `/usr/local/bin/kael-os` or `~/.local/bin/kael-os`

### macOS
- **Format**: `.dmg` (disk image)
- **Distribution**: GitHub Releases + Homebrew tap
- **Auto-Update**: `brew upgrade kael-os`
- **Install Location**: `/Applications/Kael-OS.app`

### Android
- **Format**: `.apk` (sideload) / `.aab` (Google Play)
- **Distribution**: Firebase App Distribution + GitHub Releases
- **Framework**: React Native or Flutter (TBD)
- **Sync**: Firestore for config/chat history

## 🚀 Auto-Update Mechanism

### Version Manifest (stored on all mirrors):
```json
{
  "version": "0.2.0",
  "released": "2025-12-13T10:00:00Z",
  "platforms": {
    "windows": {
      "url": "https://github.com/LeeTheOrc/kael-os/releases/download/v0.2.0/kael-os-0.2.0-x64.msi",
      "sha256": "abc123...",
      "size": 45000000,
      "mirrors": [
        "https://kael-os.web.app/releases/v0.2.0/kael-os-0.2.0-x64.msi",
        "https://yourdomain.com/kael-os/releases/v0.2.0/kael-os-0.2.0-x64.msi"
      ]
    },
    "linux": {
      "url": "https://github.com/LeeTheOrc/kael-os/releases/download/v0.2.0/kael-os-0.2.0-x64.AppImage",
      "sha256": "def456...",
      "size": 50000000,
      "mirrors": [...]
    }
  },
  "changelog": "Fixed encryption, added GPG keys, SSL certs..."
}
```

### App Update Flow:
1. App checks version on startup (async, non-blocking)
2. Compares local version vs. remote manifest
3. If newer available:
   - Show notification: "Update available (0.2.0)"
   - Download in background from primary mirror
   - Fallback to secondary mirrors if primary fails
4. When download complete:
   - Verify SHA256 checksum
   - Create backup of current version
   - Install new version
   - Restart app

### Update Server Endpoints:
```
GET /api/update/check?platform=linux&arch=x86_64&version=0.1.0
→ Returns: { newer: true, version: "0.2.0", manifest_url: "..." }

GET /manifest.json
→ Returns: Full version manifest with all mirrors

GET /releases/{version}/{filename}
→ Redirects to primary/secondary mirror based on availability
```

## 🔄 Repository Mirroring System

### Repo Structure (3 mirrors):
```
GitHub (repo server):
  releases/
    core/
      os/x86_64/
        core.db.tar.gz
        core.db.tar.gz.sig
        package1-1.0-1-x86_64.pkg.tar.zst
        ...
    extra/
      os/x86_64/
        ...

Firebase Hosting:
  repos/core/os/x86_64/
    core.db.tar.gz
    (same sync from GitHub)

cPanel webdisk:
  /home/youruser/public_html/repos/core/os/x86_64/
    (same content, synced daily via webhook)
```

### Auto-Sync Workflow (when you add new package):
1. Run: `makepkg -si --sign` (signs with your GPG key)
2. Run: `repo-add --sign /srv/repo/core/os/x86_64/core.db.tar.gz pkg.tar.zst`
3. GitHub Actions workflow:
   - Detects new release tag
   - Uploads to GitHub Releases
   - Triggers Firebase deploy
   - Triggers cPanel sync (webhook)

## 🛠️ Implementation Plan

### Phase 1: Auto-Update (Week 1-2)
- [ ] Create version manifest API
- [ ] Implement update checker in app
- [ ] Add download manager with fallback mirrors
- [ ] Package installers for Windows/Linux/macOS

### Phase 2: Repository Mirroring (Week 3-4)
- [ ] Setup GitHub repo mirror
- [ ] Configure Firebase Hosting
- [ ] Create cPanel sync script
- [ ] Generate and pin SSL certificates

### Phase 3: Android Port (Week 5+)
- [ ] Choose framework (React Native or Flutter)
- [ ] Port Dioxus UI to platform
- [ ] Implement Firebase sync
- [ ] Build APK/AAB distribution

### Phase 4: CI/CD Automation (Ongoing)
- [ ] GitHub Actions for releases
- [ ] Automated installer builds
- [ ] Mirror sync on schedule
- [ ] Signature verification

## 📋 Configuration Files

### .env.local (app config):
```env
# Update Server
UPDATE_SERVER=https://yourdomain.com/api/update
PRIMARY_MIRROR=https://github.com/LeeTheOrc/kael-os/releases
SECONDARY_MIRROR=https://kael-os.web.app/releases
TERTIARY_MIRROR=https://yourdomain.com/kael-os/releases

# SSL/TLS
CERTIFICATE_PINNING_ENABLED=true
PINNED_CERT_HASHES=["sha256/abc123...","sha256/def456..."]

# Repo Mirrors
REPO_MIRRORS=[
  "https://github.com/LeeTheOrc/kael-os-repo",
  "https://kael-os.web.app/repos",
  "https://yourdomain.com/repos"
]

# Platform Detection
PLATFORM=linux
ARCH=x86_64
VERSION=0.1.0
```

### pacman.conf (for users):
```ini
[core]
SigLevel = Required DatabaseNever
Server = https://github.com/LeeTheOrc/kael-os-repo/releases/download/core/$arch
Server = https://kael-os.web.app/repos/core/$arch
Server = https://yourdomain.com/repos/core/$arch

[kael-extra]
Server = https://github.com/LeeTheOrc/kael-os-repo/releases/download/extra/$arch
Server = https://kael-os.web.app/repos/extra/$arch
Server = https://yourdomain.com/repos/extra/$arch
```

## 🔗 User Installation Flow

### First-Time User (Windows/Linux/macOS):
1. Download installer from https://kael-os.com (or GitHub releases)
2. Run installer (auto-downloads from preferred mirror)
3. Launch app → first-run setup wizard
4. Configure:
   - Auth (Google/GitHub login)
   - Repo mirrors (auto-detected best mirror)
   - AI provider keys (optional)
   - Update frequency
5. Create desktop shortcut
6. Auto-check for updates weekly

### Existing Users:
1. App auto-checks for updates daily
2. Downloads in background
3. Shows notification: "Kael-OS 0.2.0 available"
4. One-click update or auto-update on next restart

### Linux (from repo):
```bash
# Add Kael-OS repos to pacman.conf
sudo pacman -S kael-os

# Auto-updates with: pacman -Syu
```

## 🌐 Web Dashboard (Future)

Will provide:
- Download statistics
- Mirror health status
- Release notes
- User guide
- Community repos

## 🔒 Security Checklist

- [ ] GPG sign all releases
- [ ] Implement certificate pinning
- [ ] Verify SHA256 checksums on download
- [ ] Store signing keys encrypted (AES-256-GCM in Firebase)
- [ ] Rate limit update checks (avoid hammering)
- [ ] Implement rollback capability
- [ ] Log update attempts for audit
- [ ] Validate all downloaded files before execution

## 📊 Mirror Failover Logic

```rust
async fn download_from_mirrors(filename: &str) -> Result<Vec<u8>> {
    let mirrors = vec![
        "https://github.com/LeeTheOrc/kael-os/releases/...",
        "https://kael-os.web.app/releases/...",
        "https://yourdomain.com/kael-os/releases/...",
    ];
    
    for mirror in mirrors {
        match reqwest::get(&format!("{}/{}", mirror, filename)).await {
            Ok(resp) if resp.status().is_success() => return Ok(resp.bytes().await?.to_vec()),
            _ => continue, // Try next mirror
        }
    }
    
    Err("All mirrors failed".into())
}
```

## 🎯 Next Actions

1. **Immediate** (this week):
   - [ ] Setup version.json API on cPanel
   - [ ] Create GitHub release process
   - [ ] Add update checker to app

2. **This month**:
   - [ ] Build platform-specific installers
   - [ ] Setup Firebase Hosting mirror
   - [ ] Configure SSL certificate pinning

3. **Next month**:
   - [ ] Implement repository mirroring
   - [ ] Build Arch AUR package
   - [ ] Create Homebrew tap (macOS)

4. **Future**:
   - [ ] Plan Android architecture
   - [ ] Setup Google Play / F-Droid distribution
   - [ ] Build user dashboard
