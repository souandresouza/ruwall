# Maintainer: Andre <andre@example.com>
pkgname=ruwall
pkgver=3.3.1
pkgrel=1
pkgdesc="Generate and change color-schemes on the fly (Rust port of pywal)"
arch=('x86_64' 'aarch64')
url="https://github.com/souandresouza/ruwall"
license=('MIT')
makedepends=('cargo' 'rust')
# The binary is self-contained: no ImageMagick or Python is needed to
# generate a palette. Wallpaper setters and xrdb are optional and only used
# when applying the generated scheme.
depends=()
optdepends=(
    'feh: wallpaper support (X11)'
    'nitrogen: wallpaper support (X11)'
    'imagemagick: fallback wallpaper support (display)'
    'xorg-xrdb: merge colors.Xresources into the X resource database'
    'python2: GTK2 reload support'
    'i3-wm: i3 reload support'
    'sway: sway reload support'
    'polybar: polybar reload support'
    'kitty: kitty terminal support'
    'bspwm: bspwm reload support'
)
conflicts=('python-pywal' 'python-pywal16')
provides=('wal')
source=("${pkgname}-${pkgver}.tar.gz::https://github.com/souandresouza/ruwall/archive/refs/tags/v${pkgver}.tar.gz")
sha256sums=('7fd5cc03ca435fe1af2aadfe44027142da6797cef81359b33f0af4676c723ef9')

prepare() {
    cd "${pkgname}-${pkgver}"
    cargo fetch --locked
}

build() {
    cd "${pkgname}-${pkgver}"
    cargo build --release --locked
}

check() {
    cd "${pkgname}-${pkgver}"
    cargo test --release --locked
}

package() {
    cd "${pkgname}-${pkgver}"
    
    install -Dm755 "target/release/ruwall" "${pkgdir}/usr/bin/ruwall"
    # Keep pywal's executable name working for existing scripts/keybindings.
    ln -s ruwall "${pkgdir}/usr/bin/wal"
    install -Dm644 "LICENSE" "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"
    install -Dm644 "README.md" "${pkgdir}/usr/share/doc/${pkgname}/README.md"
}
