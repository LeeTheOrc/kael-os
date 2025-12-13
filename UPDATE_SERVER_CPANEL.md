# Kael-OS Update Server (PHP/cPanel)

Deploy this on your cPanel hosting at: yourdomain.com/kael-os/api/

## Files to create:

### 1. config.php
```php
<?php
// config.php - Update server configuration

define('CURRENT_VERSION', '0.1.0');
define('APP_NAME', 'kael-os');
define('RELEASE_DATE', '2025-12-13');

// Mirror URLs (update these with your actual mirrors)
$mirrors = [
    'windows' => [
        'primary' => 'https://github.com/LeeTheOrc/kael-os/releases/download/v0.1.0/',
        'secondary' => 'https://kael-os.web.app/releases/v0.1.0/',
        'tertiary' => 'https://yourdomain.com/kael-os/releases/v0.1.0/',
    ],
    'linux' => [
        'primary' => 'https://github.com/LeeTheOrc/kael-os/releases/download/v0.1.0/',
        'secondary' => 'https://kael-os.web.app/releases/v0.1.0/',
        'tertiary' => 'https://yourdomain.com/kael-os/releases/v0.1.0/',
    ],
    'macos' => [
        'primary' => 'https://github.com/LeeTheOrc/kael-os/releases/download/v0.1.0/',
        'secondary' => 'https://kael-os.web.app/releases/v0.1.0/',
    ],
];

// Release info
$releases = [
    '0.1.0' => [
        'windows' => [
            'filename' => 'kael-os-0.1.0-x64.msi',
            'sha256' => 'abc123def456...', // Update after building
            'size' => 45000000,
        ],
        'linux' => [
            'filename' => 'kael-os-0.1.0-x64.AppImage',
            'sha256' => 'def456abc123...',
            'size' => 50000000,
        ],
        'macos' => [
            'filename' => 'kael-os-0.1.0.dmg',
            'sha256' => 'ghi789jkl012...',
            'size' => 55000000,
        ],
    ],
];

$changelog = [
    '0.1.0' => 'Initial release with AES-256-GCM encryption, GPG key management, SSL/TLS support',
];
?>
```

### 2. check.php
```php
<?php
header('Content-Type: application/json');
require 'config.php';

$platform = $_GET['platform'] ?? 'unknown';
$arch = $_GET['arch'] ?? 'x86_64';
$version = $_GET['version'] ?? '0.0.0';

// Version comparison function
function compare_versions($v1, $v2) {
    $parts1 = array_map('intval', explode('.', $v1));
    $parts2 = array_map('intval', explode('.', $v2));
    
    for ($i = 0; $i < 3; $i++) {
        $p1 = $parts1[$i] ?? 0;
        $p2 = $parts2[$i] ?? 0;
        if ($p2 > $p1) return 1;
        if ($p2 < $p1) return -1;
    }
    return 0;
}

$update_available = compare_versions($version, CURRENT_VERSION) < 0;

$response = [
    'update_available' => $update_available,
    'current_version' => $version,
    'latest_version' => CURRENT_VERSION,
    'error' => null,
];

if ($update_available) {
    $response['version_info_url'] = 'https://yourdomain.com/kael-os/api/manifest.json';
}

echo json_encode($response, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES);
?>
```

### 3. manifest.json
```json
{
  "version": "0.1.0",
  "released": "2025-12-13T10:00:00Z",
  "changelog": "Initial release with AES-256-GCM encryption, GPG key management, SSL/TLS support",
  "platforms": {
    "windows": {
      "url": "https://github.com/LeeTheOrc/kael-os/releases/download/v0.1.0/kael-os-0.1.0-x64.msi",
      "sha256": "abc123def456...",
      "size": 45000000,
      "mirrors": [
        "https://github.com/LeeTheOrc/kael-os/releases/download/v0.1.0/",
        "https://kael-os.web.app/releases/v0.1.0/",
        "https://yourdomain.com/kael-os/releases/v0.1.0/"
      ],
      "signature_url": "https://yourdomain.com/kael-os/releases/v0.1.0/kael-os-0.1.0-x64.msi.sig"
    },
    "linux": {
      "url": "https://github.com/LeeTheOrc/kael-os/releases/download/v0.1.0/kael-os-0.1.0-x64.AppImage",
      "sha256": "def456abc123...",
      "size": 50000000,
      "mirrors": [
        "https://github.com/LeeTheOrc/kael-os/releases/download/v0.1.0/",
        "https://kael-os.web.app/releases/v0.1.0/",
        "https://yourdomain.com/kael-os/releases/v0.1.0/"
      ],
      "signature_url": "https://yourdomain.com/kael-os/releases/v0.1.0/kael-os-0.1.0-x64.AppImage.sig"
    },
    "macos": {
      "url": "https://github.com/LeeTheOrc/kael-os/releases/download/v0.1.0/kael-os-0.1.0.dmg",
      "sha256": "ghi789jkl012...",
      "size": 55000000,
      "mirrors": [
        "https://github.com/LeeTheOrc/kael-os/releases/download/v0.1.0/",
        "https://kael-os.web.app/releases/v0.1.0/"
      ],
      "signature_url": "https://yourdomain.com/kael-os/releases/v0.1.0/kael-os-0.1.0.dmg.sig"
    }
  }
}
```

### 4. .htaccess (Enable CORS & caching)
```apache
# Enable CORS
<IfModule mod_headers.c>
    Header set Access-Control-Allow-Origin "*"
    Header set Access-Control-Allow-Methods "GET, POST, OPTIONS"
    Header set Access-Control-Allow-Headers "Content-Type"
</IfModule>

# Enable caching for manifest
<FilesMatch "\.json$">
    Header set Cache-Control "public, max-age=3600"
    Header set Content-Type "application/json"
</FilesMatch>

# Enable HTTPS redirect
RewriteEngine On
RewriteCond %{HTTPS} off
RewriteRule ^(.*)$ https://%{HTTP_HOST}%{REQUEST_URI} [L,R=301]
```

## Deployment Instructions:

1. Create `/public_html/kael-os/api/` directory in cPanel
2. Upload config.php, check.php, and manifest.json
3. Upload .htaccess to enable CORS and caching
4. Configure Let's Encrypt SSL (cPanel auto-renews)
5. Test endpoints:
   - `https://yourdomain.com/kael-os/api/check.php?platform=linux&arch=x86_64&version=0.1.0`
   - `https://yourdomain.com/kael-os/api/manifest.json`

## Update Workflow:

When you release v0.2.0:
1. Update `CURRENT_VERSION` in config.php
2. Update `releases` array with new file info
3. Update manifest.json with new hashes and mirrors
4. Add release to GitHub with all 3 files (Windows, Linux, macOS)
5. App auto-detects and notifies users

## Mirror Setup:

### GitHub Releases:
- Create release tag: `v0.2.0`
- Upload: kael-os-0.2.0-x64.msi, .AppImage, .dmg
- GitHub CDN serves automatically

### Firebase Hosting:
- Upload files to Firebase Storage
- Configure Cloud Storage CORS in Firebase Console
- Files at: `https://kael-os.web.app/releases/v0.2.0/`

### cPanel webdisk:
- Use `rsync` or `sftp` to upload from build server:
  ```bash
  rsync -avz ~/releases/v0.2.0/* user@yourdomain.com:public_html/kael-os/releases/v0.2.0/
  ```

## Next Steps:

After this is setup, modify `app.rs` to call the update checker on startup:
```rust
use_effect(move || {
    spawn(async {
        match crate::updater::check_for_updates("0.1.0", "https://yourdomain.com/kael-os/api").await {
            Ok(response) if response.update_available => {
                // Show update notification
                log::info!("Update available: {}", response.latest_version.unwrap_or_default());
            }
            Ok(_) => log::info!("App is up to date"),
            Err(e) => log::warn!("Update check failed: {}", e),
        }
    });
});
```
