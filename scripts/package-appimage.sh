#!/usr/bin/env bash
# Build the Linux package: an AppImage holding the release binary, a desktop entry and an icon.
#
#   scripts/package-appimage.sh                # builds target/appimage/StudyFrog-<version>-x86_64.AppImage and .tar.gz, each with a .sha256
#   scripts/package-appimage.sh --appdir-only  # stops after assembling the AppDir (no download)
#
# appimagetool is downloaded once (pinned release) into target/tools unless $APPIMAGETOOL points at one.
# The image carries the application only: the binary links just libc, and loads OpenGL, X11, Wayland and
# xkbcommon from the host at run time, like any egui application. It contains no language model (see docs/specs/scoring.md).
set -euo pipefail
cd "$(dirname "$0")/.."

TOOL_VERSION="1.9.0"
TOOL_URL="https://github.com/AppImage/appimagetool/releases/download/${TOOL_VERSION}/appimagetool-x86_64.AppImage"
VERSION=$(cargo metadata --no-deps --format-version 1 \
  | python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="studyfrog-app"))')
OUT="target/appimage"
APPDIR="$OUT/StudyFrog.AppDir"

echo "==> building studyfrog $VERSION (release)"
cargo build --release --locked -p studyfrog-app

echo "==> assembling $APPDIR"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
# The release profile keeps line tables for profiling; the package does not need them.
strip --strip-all -o "$APPDIR/usr/bin/studyfrog" target/release/studyfrog
chmod 755 "$APPDIR/usr/bin/studyfrog"
install -m 644 packaging/studyfrog.desktop "$APPDIR/studyfrog.desktop"
install -m 644 packaging/studyfrog.png "$APPDIR/studyfrog.png"
ln -s usr/bin/studyfrog "$APPDIR/AppRun"

# The same binary as a plain archive, for people who do not use AppImages.
TARBALL="$OUT/StudyFrog-$VERSION-x86_64.tar.gz"
STAGE="$OUT/StudyFrog-$VERSION"
echo "==> archiving $TARBALL"
rm -rf "$STAGE"
mkdir -p "$STAGE/bin" "$STAGE/share/applications" "$STAGE/share/icons"
install -m 755 "$APPDIR/usr/bin/studyfrog" "$STAGE/bin/studyfrog"
install -m 644 packaging/studyfrog.desktop "$STAGE/share/applications/studyfrog.desktop"
install -m 644 packaging/studyfrog.png "$STAGE/share/icons/studyfrog.png"
install -m 644 README.md LICENSE-APACHE LICENSE-MIT CHANGELOG.md "$STAGE/"
install -m 644 "docs/RELEASE_NOTES_$VERSION.md" "$STAGE/RELEASE_NOTES.md" 2>/dev/null || true
tar --sort=name --owner=0 --group=0 --numeric-owner -C "$OUT" -czf "$TARBALL" "StudyFrog-$VERSION"
(cd "$OUT" && sha256sum "$(basename "$TARBALL")" > "$(basename "$TARBALL").sha256")
rm -rf "$STAGE"

if [ "${1:-}" = "--appdir-only" ]; then
  echo "package: AppDir and tarball ready ($APPDIR, $TARBALL); run without --appdir-only to pack the AppImage"
  exit 0
fi

TOOL="${APPIMAGETOOL:-target/tools/appimagetool-$TOOL_VERSION.AppImage}"
if [ ! -x "$TOOL" ]; then
  echo "==> fetching appimagetool $TOOL_VERSION"
  mkdir -p "$(dirname "$TOOL")"
  curl -fsSL -o "$TOOL" "$TOOL_URL"
  chmod +x "$TOOL"
fi

IMAGE="$OUT/StudyFrog-$VERSION-x86_64.AppImage"
echo "==> packing $IMAGE"
ARCH=x86_64 "$TOOL" --appimage-extract-and-run "$APPDIR" "$IMAGE"
(cd "$OUT" && sha256sum "$(basename "$IMAGE")" > "$(basename "$IMAGE").sha256")
echo "package: ok ($IMAGE and $TARBALL, checksums beside them)"
