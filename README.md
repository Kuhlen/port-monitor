# Port Monitor

A desktop serial port monitoring application built with **Tauri 2** and **Leptos**. Connect to serial devices, monitor incoming data in real-time, and apply filters to focus on what matters.

> **Note:** This is a learning project built while exploring Tauri 2 and Leptos as a full-Rust stack for native desktop applications. Feedback and suggestions are welcome!

## Features

- **Port Scanning** — Auto-detect available serial ports (USB, Bluetooth, PCI) with one click
- **Full Configuration** — Baud rate (50–921600), data bits, stop bits, parity, and flow control
- **Real-time Console** — Color-coded log entries (info, data, warning, error) with millisecond timestamps
- **Data Filtering** — Offset, length, and character exclusion filters, toggleable without disconnecting
- **Auto-scroll** — Console follows incoming data automatically

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
- [Trunk](https://trunkrs.dev/) — `cargo install trunk`
- [Bun](https://bun.sh/) (or npm/pnpm for Tailwind CSS)
- [Tauri CLI](https://tauri.app/) — `cargo install tauri-cli`

### Development

```bash
# Install frontend dependencies
bun install

# Run in development mode
cargo tauri dev
```

### Build

```bash
cargo tauri build
```

## Project Structure

```
src/                    # Frontend (Leptos / WASM)
├── pages/              #   Page components
└── components/         #   UI components (connection, console, filter)

src-tauri/              # Backend (Rust / Tauri)
├── src/commands/       #   IPC commands (list_ports, connect, disconnect)
├── src/state.rs        #   Serial connection state management
└── src/types.rs        #   Shared data structures
```

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

MIT
