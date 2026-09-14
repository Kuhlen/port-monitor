# Port Monitor

A desktop serial port monitoring application built with **Tauri 2** and **Leptos**. Connect to serial devices, monitor incoming data in real-time, and apply filters to focus on what matters.

> **Note:** This is a learning project built while exploring Tauri 2 and Leptos as a full-Rust stack for native desktop applications. Feedback and suggestions are welcome!

## Features

- **Port Scanning** - Auto-detect available serial ports (USB, Bluetooth, PCI) with one click
- **Full Configuration** - Baud rate (50-921600), data bits, stop bits, parity, and flow control
- **Real-time Console** - Color-coded log entries (info, data, warning, error) with millisecond timestamps
- **Data Filtering** - Offset, length, and character exclusion filters, toggleable without disconnecting
- **Auto-scroll** - Console follows incoming data automatically

## Tech Stack

| Layer    | Technology                        |
| -------- | --------------------------------- |
| Frontend | Leptos 0.8 (Rust → WASM)         |
| Backend  | Tauri 2 + `serialport` crate     |
| Styling  | Tailwind CSS 4                    |
| Bundler  | Trunk                            |

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Trunk](https://trunkrs.dev/) - `cargo install trunk`
- [Tauri CLI](https://tauri.app/) - `cargo install tauri-cli`

No Node toolchain needed: Trunk downloads the Tailwind CLI itself
(pinned in `Trunk.toml`).

### Development

```bash
cargo tauri dev
```

### Build

```bash
cargo tauri build
```

## Project Structure

Single Cargo workspace. Dependencies point one way: both sides depend on
`crates/core`, and `crates/core` depends on nothing of its own.

```
crates/core/            # Shared domain - must compile to wasm32
├── src/error.rs        #   AppError, crosses IPC typed
└── src/features/       #   serial, filter, update: DTOs + trait contracts

src/                    # Frontend (Leptos / WASM)
├── bridge.rs           #   the ONLY place invoke() is called
├── pages/              #   Page components
└── components/         #   UI components (connection, console, filter)

src-tauri/              # Backend (Rust / Tauri)
├── src/commands.rs     #   pure delegation, no logic
├── src/state.rs        #   AppState, implements the core traits
├── src/infra.rs        #   serialport, reader thread, updater
└── tests/commands.rs   #   smoke test: every command is registered
```

IPC types are defined once, in `crates/core`. Changing a trait signature
breaks both sides at compile time.

### Tests

```bash
cargo test -p port-monitor-core   # domain logic
cargo test -p port-monitor        # command registration + capabilities
```

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

MIT
