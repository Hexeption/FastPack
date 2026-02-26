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
  <key>CFBundleIdentifier</key><string>uk.co.hexeption.fastpack</string>
  <key>CFBundleName</key><string>FastPack</string>
  <key>CFBundleDisplayName</key><string>FastPack</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
</dict>
</plist>
PLIST

ICO=$(find assets -name 'icon.svg' -print -quit)
if [ -n "$ICO" ]; then
  mkdir -p icon.iconset

  SIZES="
16,16x16
32,16x16@2x
32,32x32
64,32x32@2x
128,128x128
256,128x128@2x
256,256x256
512,256x256@2x
512,512x512
1024,512x512@2x
"

  for PARAMS in $SIZES; do
    SIZE=$(echo "$PARAMS" | cut -d, -f1)
    LABEL=$(echo "$PARAMS" | cut -d, -f2)
    svg2png -w "$SIZE" -h "$SIZE" "$ICO" "icon.iconset/icon_${LABEL}.png"
  done

  iconutil -c icns icon.iconset -o FastPack.app/Contents/Resources/AppIcon.icns
fi

# Convert DMG background SVG to PNG
BG=$(find assets -name 'dmg-background.svg' -print -quit)
if [ -n "$BG" ]; then
  mkdir -p dmg-bg
  svg2png -w 540 -h 380 "$BG" dmg-bg/background.png
fi

npm install -g appdmg

cat > appdmg_config.json << 'CONFIG'
{
  "title": "FastPack",
  "icon": "FastPack.app/Contents/Resources/AppIcon.icns",
  "background": "dmg-bg/background.png",
  "background-color": "#000000",
  "icon-size": 64,
  "window": {
    "position": { "x": 400, "y": 100 },
    "size": { "width": 540, "height": 380 }
  },
  "contents": [
    { "x": 160, "y": 195, "type": "file", "path": "FastPack.app" },
    { "x": 380, "y": 195, "type": "link", "path": "/Applications" }
  ]
}
CONFIG

appdmg appdmg_config.json "$ARTIFACT"

rm -rf FastPack.app dmg-bg appdmg_config.json 
