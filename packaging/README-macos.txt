Port Monitor for macOS (Apple Silicon and Intel).

Move "Port Monitor.app" to Applications. The app is not signed. On first launch
macOS blocks it: open System Settings → Privacy & Security → "Open Anyway".
Or run: xattr -dr com.apple.quarantine "/Applications/Port Monitor.app"