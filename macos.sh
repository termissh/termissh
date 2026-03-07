#!/usr/bin/env bash
set -euo pipefail

APP_NAME="TermiSSH"
BINARY_NAME="termissh"
APP_BUNDLE="$APP_NAME.app"
CONTENTS_DIR="$APP_BUNDLE/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"
INFO_PLIST="$CONTENTS_DIR/Info.plist"
ICON_SRC="src/icons/mini-icon.png"
ICONSET_DIR="$APP_NAME.iconset"
ICON_ICNS="$APP_NAME.icns"

if ! command -v cargo >/dev/null 2>&1; then
  echo "[ERROR] cargo bulunamadi. Rust kur: https://rustup.rs"
  exit 1
fi

if [[ ! -f "Cargo.toml" ]]; then
  echo "[ERROR] Bu script proje kok dizininde calismali (Cargo.toml bulunamadi)."
  exit 1
fi

VERSION="$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' Cargo.toml | head -n1)"
if [[ -z "$VERSION" ]]; then
  VERSION="0.1.0"
fi

echo "[*] Release build aliniyor..."
cargo build --release --bin "$BINARY_NAME"

if [[ ! -f "target/release/$BINARY_NAME" ]]; then
  echo "[ERROR] Binary uretilmedi: target/release/$BINARY_NAME"
  exit 1
fi

echo "[*] .app klasor yapisi olusturuluyor..."
rm -rf "$APP_BUNDLE" "$ICONSET_DIR" "$ICON_ICNS"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

cp "target/release/$BINARY_NAME" "$MACOS_DIR/$BINARY_NAME"
chmod +x "$MACOS_DIR/$BINARY_NAME"

cat > "$INFO_PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>$APP_NAME</string>
  <key>CFBundleDisplayName</key><string>$APP_NAME</string>
  <key>CFBundleIdentifier</key><string>org.termissh.app</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleExecutable</key><string>$BINARY_NAME</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
</dict>
</plist>
EOF

if [[ -f "$ICON_SRC" ]] && command -v sips >/dev/null 2>&1 && command -v iconutil >/dev/null 2>&1; then
  echo "[*] Uygulama iconu olusturuluyor..."
  mkdir -p "$ICONSET_DIR"

  sips -z 16 16   "$ICON_SRC" --out "$ICONSET_DIR/icon_16x16.png" >/dev/null
  sips -z 32 32   "$ICON_SRC" --out "$ICONSET_DIR/icon_16x16@2x.png" >/dev/null
  sips -z 32 32   "$ICON_SRC" --out "$ICONSET_DIR/icon_32x32.png" >/dev/null
  sips -z 64 64   "$ICON_SRC" --out "$ICONSET_DIR/icon_32x32@2x.png" >/dev/null
  sips -z 128 128 "$ICON_SRC" --out "$ICONSET_DIR/icon_128x128.png" >/dev/null
  sips -z 256 256 "$ICON_SRC" --out "$ICONSET_DIR/icon_128x128@2x.png" >/dev/null
  sips -z 256 256 "$ICON_SRC" --out "$ICONSET_DIR/icon_256x256.png" >/dev/null
  sips -z 512 512 "$ICON_SRC" --out "$ICONSET_DIR/icon_256x256@2x.png" >/dev/null
  sips -z 512 512 "$ICON_SRC" --out "$ICONSET_DIR/icon_512x512.png" >/dev/null
  cp "$ICON_SRC" "$ICONSET_DIR/icon_512x512@2x.png"

  iconutil -c icns "$ICONSET_DIR" -o "$ICON_ICNS"
  cp "$ICON_ICNS" "$RESOURCES_DIR/"

  /usr/libexec/PlistBuddy -c "Add :CFBundleIconFile string $APP_NAME" "$INFO_PLIST" >/dev/null
else
  echo "[WARN] Icon olusturma atlandi (icon dosyasi/sips/iconutil eksik)."
fi

if command -v codesign >/dev/null 2>&1; then
  echo "[*] Lokal codesign uygulaniyor..."
  codesign --force --deep --sign - "$APP_BUNDLE" >/dev/null 2>&1 || true
fi

echo "[*] /Applications klasorune kopyalaniyor..."
rm -rf "/Applications/$APP_BUNDLE"
cp -R "$APP_BUNDLE" "/Applications/$APP_BUNDLE"

echo "[*] Uygulama aciliyor..."
open "/Applications/$APP_BUNDLE"

echo "[OK] Tamamlandi: /Applications/$APP_BUNDLE"
echo "[NOT] Ilk acilista guvenlik uyarisi olursa sag tik > Open ile ac." 
