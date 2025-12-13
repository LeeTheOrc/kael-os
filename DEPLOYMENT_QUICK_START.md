# Kael-OS Quick Deployment Reference

## 🎯 The Big Picture

Your app will work across 4 platforms using 4 mirrors:

```
┌─────────────────────────────────────────────┐
│         YOUR INFRASTRUCTURE                 │
├─────────────────────────────────────────────┤
│                                              │
│  Desktop (Windows/Linux/macOS)              │
│  ↓ Auto-update from GitHub                 │
│                                              │
│  Android (APK/AAB)                         │
│  ↓ Auto-update from Firebase               │
│                                              │
│  Repositories (Arch packages)              │
│  ↓ Synced from 3 mirrors                   │
│                                              │
│  Cloud Services:                            │
│  ├─ GitHub (primary distribution CDN)      │
│  ├─ Firebase (backup + Android delivery)   │
│  ├─ cPanel (webhosting + Let's Encrypt)   │
│  └─ Google Cloud (AUR fallback)            │
│                                              │
└─────────────────────────────────────────────┘
```

## 🚀 Implementation Order (Next 8 weeks)

### Week 1-2: Update Server (cPanel)
**Goal**: Get auto-update working for desktop

```bash
1. Create /public_html/kael-os/api/ on cPanel
2. Upload: config.php, check.php, manifest.json
3. Add .htaccess for CORS
4. Test: curl https://yourdomain.com/kael-os/api/check.php
```

**Deliverable**: Desktop app checks version on startup

### Week 2-3: Build Installers
**Goal**: Package app for Windows, Linux, macOS

```bash
1. Windows: `cargo build --release` → .msi via WiX
2. Linux: AppImage or deb package
3. macOS: .dmg via create-dmg
4. Get SHA256 hashes for each
```

**Deliverable**: 3 installer files + hashes → update in manifest.json

### Week 3-4: GitHub Releases + Firebase Hosting
**Goal**: Deploy to 2 mirrors

```bash
1. GitHub:
   - Create release v0.2.0
   - Upload .msi, .AppImage, .dmg
   - Auto-CDN via GitHub

2. Firebase:
   - Upload files to Storage
   - Configure CORS
   - Deployed at: kael-os.web.app/releases/v0.2.0/
```

**Deliverable**: All mirrors populated, app downloads from any

### Week 4-5: Repository Sync Setup
**Goal**: Sync Arch repos across mirrors

```bash
1. Create GitHub releases repo: kael-os-repo
2. Add workflow to auto-sync on package build
3. Setup cPanel sync script
4. Test: pacman -S kael-os from custom mirror
```

**Deliverable**: Users can install packages from any mirror

### Week 5-6: Android MVP
**Goal**: Chat + Settings working on Android

```bash
1. Init React Native: npx react-native init
2. Add Firebase integration
3. Implement OAuth login
4. Port ChatScreen component
5. Build APK
```

**Deliverable**: Android beta app for testing

### Week 6-7: Testing & Docs
**Goal**: Make it foolproof for others

```bash
1. Write installation guides (Windows/Linux/macOS)
2. Create FAQ for common issues
3. Setup GitHub Issues template
4. Test on someone else's PC
```

**Deliverable**: Easy install guide + troubleshooting

### Week 7-8: Polish & Distribution
**Goal**: Ready for public release

```bash
1. Setup Google Play Store developer account
2. Build AAB for Google Play
3. Setup F-Droid repo
4. Create changelog & release notes
5. Announce on social media
```

**Deliverable**: v1.0.0 released on all platforms

## 💻 What to Code Right Now (This Week)

### 1. Add Updater Module to App
File: `src-tauri/src/components/app.rs`

```rust
use_effect(move || {
    spawn(async {
        match crate::updater::check_for_updates(
            "0.1.0",
            "https://yourdomain.com/kael-os/api",
        ).await {
            Ok(response) if response.update_available => {
                log::info!("Update available: {}", response.latest_version.unwrap());
                // Show update notification UI
            }
            _ => {}
        }
    });
});
```

### 2. Create .env Configuration
File: `.env.local`

```env
# Update Server
UPDATE_SERVER=https://yourdomain.com/kael-os/api
PRIMARY_MIRROR=https://github.com/LeeTheOrc/kael-os/releases
SECONDARY_MIRROR=https://kael-os.web.app/releases
TERTIARY_MIRROR=https://yourdomain.com/kael-os/releases

# Current Version
APP_VERSION=0.1.0
PLATFORM=linux
ARCH=x86_64
```

### 3. Build & Get Hashes
```bash
cd src-tauri && cargo build --release
sha256sum target/release/kael-os

# Use this hash in manifest.json
```

### 4. Deploy to cPanel
```bash
# Connect via SFTP
sftp user@yourdomain.com

# Create directory
mkdir /public_html/kael-os/api

# Upload files
put config.php check.php manifest.json .htaccess

# Test
curl https://yourdomain.com/kael-os/api/check.php?platform=linux
```

## 🔐 Security Checklist

- [ ] All API calls use HTTPS (check cPanel SSL)
- [ ] Version manifest has Let's Encrypt SSL certificate pinning
- [ ] API keys encrypted with AES-256-GCM before Firebase
- [ ] GPG signatures on all releases
- [ ] SHA256 verification before installation
- [ ] Rate limiting on update checks (max 1/day per device)
- [ ] Rollback capability (keep 2 previous versions)

## 📊 Multi-Mirror Reliability

Your fallback strategy:
```
App tries to download from:
  1. GitHub (99.9% uptime, CDN)
  2. Firebase (99.9% uptime, Google infrastructure)
  3. cPanel (Your webhosting, Let's Encrypt)
  4. Google Cloud (AUR fallback)

If any fails → try next
All have same file hashes → safe to use any
```

## 🎁 What You'll Have After Week 8

✅ **Desktop**: Windows, Linux, macOS installers with auto-update
✅ **Mobile**: Android app with chat + settings
✅ **Repos**: Arch packages on 3 mirrors with auto-sync
✅ **Distribution**: Auto-update server on your domain
✅ **Security**: AES-256-GCM encryption + GPG signing
✅ **Reliability**: Multi-mirror fallback system
✅ **Documentation**: Installation guides for users
✅ **Automation**: CI/CD pipeline for releases

## 📞 User Experience

**User installs app:**
1. Downloads from yourdomain.com or GitHub
2. Runs installer (auto-verifies hash)
3. Creates account (Google/GitHub OAuth)
4. Opens app
5. App auto-checks for updates weekly
6. Notifications for new versions
7. One-click update or auto-update on restart

**For Arch Linux users:**
```bash
# Add to /etc/pacman.conf
[kael-os]
Server = https://github.com/LeeTheOrc/kael-os-repo/releases/$arch

# Install
sudo pacman -S kael-os
```

**For Android users:**
1. Download APK from GitHub or Firebase
2. Install (one tap)
3. Same login + auto-update experience

## 🏁 Success Criteria

- [ ] App installs on Windows 10/11
- [ ] App installs on Ubuntu 22.04
- [ ] App installs on macOS 12+
- [ ] App installs on Android 8+
- [ ] Auto-update works from all 3 mirrors
- [ ] Package sync works across mirrors
- [ ] Someone else installs + uses without issues
- [ ] ChatGPT can install it from your guide alone

## 🆘 Common Issues & Solutions

| Issue | Cause | Solution |
|-------|-------|----------|
| Update check times out | Mirror down | Try next mirror |
| Hash mismatch | Corrupted download | Retry or use different mirror |
| App won't start | First run no network | Show offline mode message |
| Can't login | Firebase offline | Cache last auth token |
| Repo sync fails | Mirror unavailable | Queue for next sync |

## 🚀 Launch Timeline

```
Week 8:  v0.1.0 on all platforms ✅
Month 2: v1.0.0 with all features
Month 3: Promote as GPL3 distro base
Month 6: Android on Play Store + F-Droid
Month 12: 10K+ users?
```

Ready to start Week 1? Begin with the cPanel update server setup!
