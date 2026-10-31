#!/usr/bin/env bash
# Build KadeeStudio for Linux into dist/.
#
#   ./scripts/build-linux.sh              # release build → .tar.gz, plus .deb / .rpm / AppImage
#                                         #   when dpkg-deb / rpmbuild / appimagetool are installed
#   ./scripts/build-linux.sh --run        # build and start the app
#   ./scripts/build-linux.sh --tar-only   # only the .tar.gz
#   ./scripts/build-linux.sh --target aarch64-unknown-linux-gnu
#   ./scripts/build-linux.sh --clean
#
# Build dependencies (Debian/Ubuntu):
#   sudo apt install build-essential pkg-config libxkbcommon-dev libwayland-dev \
#        libx11-dev libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev libgtk-3-dev
# Fedora:
#   sudo dnf install gcc pkgconf libxkbcommon-devel wayland-devel libX11-devel \
#        libXcursor-devel libXrandr-devel libXi-devel mesa-libGL-devel gtk3-devel rpm-build
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

APP_NAME="KadeeStudio"
BIN_NAME="kadeestudio"
APP_ID="be.kadee.KadeeStudio"
DIST="$ROOT/dist"

RUN=0 TAR_ONLY=0 CLEAN=0 TARGET=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --run) RUN=1 ;;
    --tar-only) TAR_ONLY=1 ;;
    --clean) CLEAN=1 ;;
    --target) TARGET="$2"; shift ;;
    -h|--help) sed -n '2,17p' "$0"; exit 0 ;;
    *) echo "Unknown option: $1 (see --help)" >&2; exit 2 ;;
  esac
  shift
done

step() { printf '\n\033[1;34m==> %s\033[0m\n' "$*"; }
die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

[[ "$(uname -s)" == "Linux" ]] || die "this script runs on Linux"
command -v cargo >/dev/null || die "Rust is not installed: https://rustup.rs"

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)"
[[ -n "$VERSION" ]] || die "could not read the version from Cargo.toml"

if [[ $CLEAN -eq 1 ]]; then step "cargo clean"; cargo clean; fi

if [[ -n "$TARGET" ]]; then
  command -v rustup >/dev/null && rustup target add "$TARGET" >/dev/null
  step "Building $TARGET (release)"
  cargo build --release --locked -p "$BIN_NAME" --target "$TARGET"
  BIN="target/$TARGET/release/$BIN_NAME"
  ARCH="${TARGET%%-*}"
else
  step "Building $(uname -m) (release)"
  cargo build --release --locked -p "$BIN_NAME"
  BIN="target/release/$BIN_NAME"
  ARCH="$(uname -m)"
fi
case "$ARCH" in x86_64) DEB_ARCH=amd64 ;; aarch64) DEB_ARCH=arm64 ;; *) DEB_ARCH="$ARCH" ;; esac

# ---------------------------------------------------------------- install tree
# A FHS tree (usr/bin, usr/share/…) reused by every package format.
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
TREE="$STAGE/tree"
install -Dm755 "$BIN" "$TREE/usr/bin/$BIN_NAME"
for s in 256 512; do
  install -Dm644 "assets/app-icon/kadeestudio-$s.png" "$TREE/usr/share/icons/hicolor/${s}x${s}/apps/$APP_ID.png"
done
install -Dm644 "assets/app-icon/kadeestudio-1024.png" "$TREE/usr/share/pixmaps/$APP_ID.png"
install -Dm644 packaging/linux/kadeestudio.desktop "$TREE/usr/share/applications/$APP_ID.desktop"
sed "s/@VERSION@/$VERSION/; s/@DATE@/$(date +%F)/" packaging/linux/kadeestudio.metainfo.xml \
  | install -Dm644 /dev/stdin "$TREE/usr/share/metainfo/$APP_ID.metainfo.xml"
install -Dm644 LICENSE-MIT "$TREE/usr/share/licenses/$BIN_NAME/LICENSE-MIT"
install -Dm644 LICENSE-APACHE "$TREE/usr/share/licenses/$BIN_NAME/LICENSE-APACHE"
mkdir -p "$DIST"

# ---------------------------------------------------------------- tar.gz
BASE="$APP_NAME-$VERSION-linux-$ARCH"
step "Creating $BASE.tar.gz"
mkdir -p "$STAGE/$BASE"
cp -R "$TREE/usr/." "$STAGE/$BASE/"
cp README.md "$STAGE/$BASE/"
cat > "$STAGE/$BASE/install.sh" <<'SH'
#!/usr/bin/env sh
# Install into ~/.local (no root needed): ./install.sh   or system-wide: sudo ./install.sh /usr/local
set -e
PREFIX="${1:-$HOME/.local}"
HERE="$(cd "$(dirname "$0")" && pwd)"
for d in bin share; do mkdir -p "$PREFIX/$d" && cp -R "$HERE/$d/." "$PREFIX/$d/"; done
command -v update-desktop-database >/dev/null && update-desktop-database "$PREFIX/share/applications" 2>/dev/null || true
echo "KadeeStudio installed in $PREFIX (run: kadeestudio)"
SH
chmod +x "$STAGE/$BASE/install.sh"
tar -C "$STAGE" -czf "$DIST/$BASE.tar.gz" "$BASE"

# ---------------------------------------------------------------- .deb
if [[ $TAR_ONLY -eq 0 ]] && command -v dpkg-deb >/dev/null; then
  DEB="$DIST/${BIN_NAME}_${VERSION}_${DEB_ARCH}.deb"
  step "Creating $(basename "$DEB")"
  DROOT="$STAGE/deb"
  mkdir -p "$DROOT/DEBIAN"
  cp -R "$TREE/usr" "$DROOT/"
  SIZE_KB="$(du -sk "$DROOT/usr" | cut -f1)"
  cat > "$DROOT/DEBIAN/control" <<CTRL
Package: $BIN_NAME
Version: $VERSION
Section: office
Priority: optional
Architecture: $DEB_ARCH
Installed-Size: $SIZE_KB
Depends: libc6 (>= 2.31), libxkbcommon0, libgtk-3-0
Recommends: libvulkan1 | libgl1
Maintainer: Kris Dugardyn <kris.dugardyn@gmail.com>
Homepage: https://github.com/kdugardyn66/kadeestudio
Description: Native Rust office suite
 KadeeStudio brings AI, Markdown, PDF, conversion, dashboards and reports
 together in one native app, with collaboration through a self-hosted server.
CTRL
  find "$DROOT" -type d -exec chmod 755 {} +
  dpkg-deb --root-owner-group --build "$DROOT" "$DEB" >/dev/null
fi

# ---------------------------------------------------------------- .rpm
if [[ $TAR_ONLY -eq 0 ]] && command -v rpmbuild >/dev/null; then
  step "Creating .rpm"
  RTOP="$STAGE/rpm"
  mkdir -p "$RTOP"/{BUILD,RPMS,SOURCES,SPECS,SRPMS,BUILDROOT}
  cat > "$RTOP/SPECS/$BIN_NAME.spec" <<SPEC
Name:           $BIN_NAME
Version:        $VERSION
Release:        1
Summary:        Native Rust office suite
License:        MIT OR Apache-2.0
URL:            https://github.com/kdugardyn66/kadeestudio
Requires:       libxkbcommon gtk3
%global debug_package %{nil}
%global __strip /bin/true

%description
KadeeStudio brings AI, Markdown, PDF, conversion, dashboards and reports
together in one native app, with collaboration through a self-hosted server.

%install
mkdir -p %{buildroot}
cp -R $TREE/usr %{buildroot}/

%files
/usr/bin/$BIN_NAME
/usr/share/applications/$APP_ID.desktop
/usr/share/metainfo/$APP_ID.metainfo.xml
/usr/share/icons/hicolor/256x256/apps/$APP_ID.png
/usr/share/icons/hicolor/512x512/apps/$APP_ID.png
/usr/share/pixmaps/$APP_ID.png
/usr/share/licenses/$BIN_NAME
SPEC
  rpmbuild --quiet --define "_topdir $RTOP" --target "$ARCH" -bb "$RTOP/SPECS/$BIN_NAME.spec"
  find "$RTOP/RPMS" -name '*.rpm' -exec cp {} "$DIST/" \;
fi

# ---------------------------------------------------------------- AppImage
if [[ $TAR_ONLY -eq 0 ]] && command -v appimagetool >/dev/null; then
  APPIMG="$DIST/$APP_NAME-$VERSION-$ARCH.AppImage"
  step "Creating $(basename "$APPIMG")"
  AD="$STAGE/$APP_NAME.AppDir"
  mkdir -p "$AD"
  cp -R "$TREE/usr" "$AD/"
  cp "$TREE/usr/share/applications/$APP_ID.desktop" "$AD/"
  cp "assets/app-icon/kadeestudio-256.png" "$AD/$APP_ID.png"
  printf '#!/bin/sh\nHERE="$(dirname "$(readlink -f "$0")")"\nexec "$HERE/usr/bin/%s" "$@"\n' "$BIN_NAME" > "$AD/AppRun"
  chmod +x "$AD/AppRun"
  ARCH="$ARCH" appimagetool --no-appstream "$AD" "$APPIMG" >/dev/null
fi

(cd "$DIST" && for f in "$APP_NAME-$VERSION"* "${BIN_NAME}_$VERSION"* "$BIN_NAME-$VERSION"*; do
  [[ -f "$f" && "$f" != *.sha256 ]] && sha256sum "$f" > "$f.sha256"; done; true)

step "Done"
for f in "$DIST"/*; do [[ "$f" == *.sha256 ]] || echo "  ${f#"$DIST"/}"; done

if [[ $RUN -eq 1 ]]; then exec "$BIN"; fi
