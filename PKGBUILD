# Maintainer: Kael-OS Team <team@kael-os.dev>
pkgname=kael-os
pkgver=0.1.0
pkgrel=1
pkgdesc="Kael-OS - A self-contained forge for building and publishing Arch apps"
arch=('x86_64')
url="https://github.com/yourusername/kael-os"
license=('MIT')
depends=('glibc' 'fontconfig' 'xorg-libs' 'libxkbcommon' 'libxcb' 'dbus')
makedepends=('rust' 'cargo' 'paru' 'pkg-config')
provides=("${pkgname}")
conflicts=("${pkgname}-bin" "${pkgname}-git")

source=("$pkgname-$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$srcdir/$pkgname-$pkgver"
    cargo build --release 2>&1
}

package() {
    cd "$srcdir/$pkgname-$pkgver"
    
    # Install binary
    install -Dm755 "target/release/kael-os" "$pkgdir/usr/bin/kael-os"
    
    # Install desktop file for menu shortcut
    install -Dm644 /dev/stdin "$pkgdir/usr/share/applications/kael-os.desktop" << EOF
[Desktop Entry]
Type=Application
Name=Kael-OS
Comment=Self-contained forge for building and publishing Arch apps
Exec=/usr/bin/kael-os
Icon=kael-os
Terminal=false
Categories=Development;
StartupNotify=true
MimeType=text/plain;
EOF
    
    # Install icon
    install -Dm644 "src-tauri/icons/icon.png" "$pkgdir/usr/share/pixmaps/kael-os.png"
    install -Dm644 "src-tauri/icons/128x128.png" "$pkgdir/usr/share/icons/hicolor/128x128/apps/kael-os.png"
    install -Dm644 "src-tauri/icons/128x128@2x.png" "$pkgdir/usr/share/icons/hicolor/256x256/apps/kael-os.png"
    
    # Install documentation
    install -Dm644 "README.md" "$pkgdir/usr/share/doc/kael-os/README.md"
    install -Dm644 "PUBLISHING.md" "$pkgdir/usr/share/doc/kael-os/PUBLISHING.md"
}
