#!/usr/bin/env bash
# universal "Port Monitor.app" zip from two cargo targets. run on macOS.
set -euo pipefail
cd "$(dirname "$0")/.."

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
for target in aarch64-apple-darwin x86_64-apple-darwin; do
  cargo build --release -p app --target "$target"
done

stage="dist/port-monitor-v$version-macos-universal"
app="$stage/Port Monitor.app"
rm -rf "$stage"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
lipo -create -output "$app/Contents/MacOS/port-monitor" \
  target/aarch64-apple-darwin/release/port-monitor \
  target/x86_64-apple-darwin/release/port-monitor
cp crates/app/assets/app_icon.icns "$app/Contents/Resources/AppIcon.icns"
cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Port Monitor</string>
  <key>CFBundleDisplayName</key><string>Port Monitor</string>
  <key>CFBundleIdentifier</key><string>com.kuhlen.port-monitor</string>
  <key>CFBundleExecutable</key><string>port-monitor</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
EOF
cp packaging/README-macos.txt "$stage/README.txt"
ditto -c -k --keepParent "$stage" "$stage.zip"
echo "$stage.zip"
