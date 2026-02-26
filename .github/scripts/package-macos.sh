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

# Import Developer ID certificate into a temporary keychain
if [ -n "${APPLE_CERT_P12:-}" ]; then
  KEYCHAIN_PATH="$RUNNER_TEMP/signing.keychain-db"
  KEYCHAIN_PASS=$(openssl rand -hex 16)

  security create-keychain -p "$KEYCHAIN_PASS" "$KEYCHAIN_PATH"
  security set-keychain-settings -lut 21600 "$KEYCHAIN_PATH"
  security unlock-keychain -p "$KEYCHAIN_PASS" "$KEYCHAIN_PATH"

  echo "$APPLE_CERT_P12" | base64 --decode -o "$RUNNER_TEMP/cert.p12"
  security import "$RUNNER_TEMP/cert.p12" \
    -k "$KEYCHAIN_PATH" \
    -P "$APPLE_CERT_PASSWORD" \
    -T /usr/bin/codesign
  security set-key-partition-list \
    -S apple-tool:,apple: \
    -s -k "$KEYCHAIN_PASS" "$KEYCHAIN_PATH"
  security list-keychains -d user -s "$KEYCHAIN_PATH" login.keychain

  codesign --force --options runtime \
    --sign "Developer ID Application: $APPLE_TEAM_ID" \
    --entitlements /dev/stdin \
    FastPack.app << 'ENTITLEMENTS'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>com.apple.security.cs.allow-unsigned-executable-memory</key><false/>
</dict></plist>
ENTITLEMENTS
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

if [ -n "${APPLE_CERT_P12:-}" ]; then
  codesign --force --sign "Developer ID Application: $APPLE_TEAM_ID" "$ARTIFACT"

  xcrun notarytool submit "$ARTIFACT" \
    --apple-id "$APPLE_ID" \
    --password "$APPLE_APP_PASSWORD" \
    --team-id "$APPLE_TEAM_ID" \
    --wait

  xcrun stapler staple "$ARTIFACT"
fi

rm -rf FastPack.app dmg-bg appdmg_config.json
