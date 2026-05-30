# Maintainer: Your Name <you@example.com>
pkgname=bludio
pkgver=0.1.0
pkgrel=1
pkgdesc="GPUI Bluetooth and audio manager"
arch=('x86_64')
url="https://github.com/toomosin/bludio"
license=('MIT')
depends=('bluez' 'pulseaudio' 'polkit')
makedepends=('cargo' 'rust')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$pkgname-$pkgver"
  cargo build --release --locked
}

package() {
  cd "$pkgname-$pkgver"

  # Binary
  install -Dm755 "target/release/bludio" "$pkgdir/usr/bin/bludio"

  # Polkit policy file
  install -Dm644 "policy/dev.toomosin.bludio.policy" \
    "$pkgdir/usr/share/polkit-1/actions/dev.toomosin.bludio.policy"
}
