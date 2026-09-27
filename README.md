# Port Monitor

A desktop serial port monitor built with **Rust** and **Slint**. Connect to a serial
device, watch incoming lines in real time, filter them, and send data back.

Rewrite of the Tauri 2 + Leptos app (v0.3.2) as a native Slint app. It runs on
Windows 7 through Windows 11, macOS and Linux as a single portable binary.

## Features

- **Port scanning**: lists serial ports (USB, Bluetooth, PCI) with one click.
- **Full configuration**: baud rate, data bits (5–8), parity (none/even/odd), stop bits
  (1/2) and flow control (none, RTS/CTS, XON/XOFF), summarized as a `8N1 · no flow` chip.
- **Real-time console**: `RX`, `TX`, `INF`, `WRN` and `ERR` rows with millisecond
  timestamps, RX/TX/ERR counters, auto-scroll, and a 10,000-row cap.
- **Send**: type a line and send it with no ending, LF, CR or CRLF. Sent data shows up
  as `TX` rows.
- **Line filter**: skip the first N chars, keep N chars, remove characters. A live
  preview shows what the filter cuts.
- **Clear errors**: failures show in a banner with a hint for each OS (for example, the
  `dialout` group on Linux or a busy port on Windows) and a Retry button.

## Download

Every release has portable archives. Nothing to install:

| OS | File | Notes |
|---|---|---|
| Windows 7 SP1 – 11 (x86/x64) | `port-monitor-vX.Y.Z-windows-x86.zip` | Keep the three `.dll` files next to `port-monitor.exe`. Windows 7 needs them. |
| macOS (Apple Silicon + Intel) | `port-monitor-vX.Y.Z-macos-universal.zip` | Unsigned: allow it in System Settings → Privacy & Security → Open Anyway. |
| Linux x86_64 (glibc ≥ 2.35) | `port-monitor-vX.Y.Z-linux-x86_64.tar.gz` | Add your user to `dialout` to open ports without root. |

## Tech stack

| Part | Technology |
|---|---|
| UI | Slint 1.18 (FemtoVG on macOS/Linux, software renderer on Windows) |
| Serial | `serialport` crate, one I/O thread per connection |
| Toolchain | Pinned Rust nightly (needed for the tier-3 Windows 7 target) |

## Development

Requires [rustup](https://rustup.rs/). `rust-toolchain.toml` installs the right
nightly. On Fedora, also install `fontconfig-devel` and `libxkbcommon-devel`.

```bash
cargo run -p app                 # run the app
cargo test --workspace           # all tests
cargo clippy --workspace --all-targets -- -D warnings
cargo win7 -p app                # Windows 7+ exe (MSVC host)
cargo win7-gnu -p app            # same exe via mingw on Linux
win7-shims/build.sh              # shim DLLs for Windows 7 (mingw)
scripts/bundle-macos.sh          # universal .app zip (macOS host)
```

## Project layout

```
crates/domain   pure rules: config types, line buffer, filter, send encoding, errors
crates/data     serial port access (serialport + link thread)
crates/app      Slint UI, controller, and the event pump between them
patches/        patched i-slint-backend-winit so Slint loads on Windows 7
win7-shims/     tiny DLLs that satisfy Windows 8+ imports on Windows 7
packaging/      per-OS README files shipped inside the archives
```

Dependencies point one way: `app → data → domain`.

## Release

1. Bump `[workspace.package] version` in `Cargo.toml`.
2. Push an **annotated** tag `vX.Y.Z` (`git tag -a`). CI rejects lightweight tags and
   tags that differ from the Cargo version.
3. CI builds the three archives and publishes the GitHub release. The tag message
   becomes the release notes.

When Slint is upgraded, re-apply the patches in `patches/` (see `patches/README.md`).

## Author

kuhlen
