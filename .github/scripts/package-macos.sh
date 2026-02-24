#!/usr/bin/env bash
# Usage: package-macos.sh <bin-path> <artifact-name>
set -euo pipefail

BIN="$1"
ARTIFACT="$2"

strip "$BIN"

mkdir -p FastPack.app/Contents/MacOS
mkdir -p FastPack.app/Contents/Resources
cp "$BIN" FastPack.app/Contents/MacOS/fastpack
chmod +x FastPack.app/Contents/MacOS/fastpack

cat > FastPack.app/Contents/Info.plist << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key><string>fastpack</string>
  <key>CFBundleIdentifier</key><string>com.hexeption.fastpack</string>
  <key>CFBundleName</key><string>FastPack</string>
  <key>CFBundleDisplayName</key><string>FastPack</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
</dict>
</plist>
PLIST

ICO=$(find target -name 'icon.ico' -print -quit)
if [ -n "$ICO" ]; then
  mkdir -p icon.iconset
  for sz in 16 32 64 128 256 512; do
    sips -z $sz $sz "$ICO" --out "icon.iconset/icon_${sz}x${sz}.png" 2>/dev/null || true
  done
  for sz in 16 32 64 128 256; do
    sz2=$((sz * 2))
    cp "icon.iconset/icon_${sz2}x${sz2}.png" \
       "icon.iconset/icon_${sz}x${sz}@2x.png" 2>/dev/null || true
  done
  iconutil -c icns icon.iconset -o FastPack.app/Contents/Resources/AppIcon.icns
fi

hdiutil create \
  -volname "FastPack" \
  -srcfolder FastPack.app \
  -ov -format UDZO \
  "$ARTIFACT"
