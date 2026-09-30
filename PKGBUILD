# Maintainer: Andre <andre@example.com>
pkgname=ruwall
pkgver=3.3.1
pkgrel=1
pkgdesc="Generate and change color-schemes on the fly (Rust port of pywal)"
arch=('x86_64' 'aarch64')
url="https://github.com/souandresouza/ruwall"
license=('MIT')
makedepends=('cargo' 'rust')
depends=('imagemagick' 'feh' 'nitrogen' 'xrdb')
optdepends=(
    'python2: GTK2 reload support'
    'i3-wm: i3 reload support'
    'sway: sway reload support'
    'polybar: polybar reload support'
    'kitty: kitty terminal support'
    'bspwm: bspwm reload support'
)
conflicts=('python-pywal' 'python-pywal16')
provides=('wal')
source=("${pkgname}-${pkgver}.tar.gz::https://github.com/souandresouza/ruwall/archive/v${pkgver}.tar.gz")
sha256sums=('SKIP')

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
    install -Dm644 "LICENSE" "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"
    install -Dm644 "README.md" "${pkgdir}/usr/share/doc/${pkgname}/README.md"
}
