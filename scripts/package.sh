#!/bin/sh
# Собирает Vibe Timer и складывает готовые пакеты в release/.
set -eu

cd "$(dirname "$0")/.."

VERSION=$(node -p "require('./package.json').version")
PRODUCT="Vibe Timer"
BUNDLE="src-tauri/target/release/bundle"
OUT="release"

npm run tauri build

rm -rf "$OUT"
mkdir -p "$OUT"

cp "$BUNDLE/appimage/${PRODUCT}_${VERSION}_amd64.AppImage" \
   "$OUT/vibe-timer-${VERSION}-x86_64.AppImage"
cp "$BUNDLE/rpm/${PRODUCT}-${VERSION}-1.x86_64.rpm" \
   "$OUT/vibe-timer-${VERSION}-1.x86_64.rpm"
cp "$BUNDLE/deb/${PRODUCT}_${VERSION}_amd64.deb" \
   "$OUT/vibe-timer_${VERSION}_amd64.deb"
cp src-tauri/icons/128x128.png "$OUT/vibe-timer.png"
cp scripts/install.sh "$OUT/install.sh"
cp scripts/INSTALL.md "$OUT/INSTALL.md"

chmod +x "$OUT/install.sh" "$OUT/vibe-timer-${VERSION}-x86_64.AppImage"

(
    cd "$OUT"
    sha256sum \
        "vibe-timer-${VERSION}-x86_64.AppImage" \
        "vibe-timer-${VERSION}-1.x86_64.rpm" \
        "vibe-timer_${VERSION}_amd64.deb" > SHA256SUMS
)

echo
echo "Готово. Пакеты и установщик: $OUT/"
