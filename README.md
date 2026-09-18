# TaPeer CLI

[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-blue)](#)

A fast, ergonomic, and cross-platform command-line client written in Rust for [**TaPeer**](https://github.com/facundopidal/TaPeer), the peer-to-peer file and text snippet sharing application.

> [!TIP]
> This CLI connects to a running [TaPeer Server](https://github.com/facundopidal/TaPeer). If you haven't deployed the server yet, check the [TaPeer repository](https://github.com/facundopidal/TaPeer) for setup and hosting instructions.

---

## ✨ Features

- 📦 **File Sharing**: Upload any file from your disk directly to your local network or private VPN.
- 📝 **Text Sharing**: Quickly share notes, code snippets, links, or passwords across devices.
- 🎯 **Smart Item Selector**:
  - Download or view the latest shared item with zero arguments (`tapeer get`).
  - Access items by simple list index numbers (`tapeer get 1`, `tapeer get 2`).
  - Match items by filename or partial substring (`tapeer get photo.png`).
  - Match by full UUID or short prefix (`tapeer get c1f7`).
- 📋 **Clipboard Integration**: Seamlessly copy text snippets or downloaded file paths to your clipboard with `-c` / `--copy`.
- 📁 **Smart Folder Output**: Save files into folders directly (`-o downloads/`), with automatic directory creation if they don't exist yet.
- 🎨 **Rich Terminal Output**: Colored badges, human-readable file sizes (`KB`, `MB`), and real-time expiration countdowns.
- 🌐 **Flexible Networking**: Works out of the box with `localhost`, local Wi-Fi networks, and private mesh VPNs like **Tailscale**.
- 🐧 **Cross-Platform**: Tested on Windows and Linux (CachyOS / Arch Linux with Wayland and X11).

---

## 🚀 Installation

### Quick Install (Prebuilt Binaries)

**Linux & macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/facundopidal/tapeer-cli/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/facundopidal/tapeer-cli/main/install.ps1 | iex
```

---

### Build from Source (via Cargo)

If you have [Rust & Cargo](https://rustup.rs/) installed:

```bash
cargo install --git https://github.com/facundopidal/tapeer-cli.git
```

Or clone and compile locally:

```bash
git clone https://github.com/facundopidal/tapeer-cli.git
cd tapeer-cli
cargo install --path .
```

---

## 📖 Usage & Examples

### 1. List Shared Items
View all active items currently shared on TaPeer:

```bash
tapeer list
```

Example output:
```text
 TaPeer  (2 active items)

   #1 📦 [File] report.pdf  •  1.45 MB  •  expires in 23h 40m
      ↳ id: 550e8400-e29b-41d4-a716-446655440000

   #2 📝 [Text] snippet.txt  •  28 B  •  expires in 23h 55m
      ↳ "wifi-key: secret-2026"
```

---

### 2. Share Text Snippets
Share notes, links, or quick text messages:

```bash
tapeer text "Server restarted at 14:00"
```

---

### 3. Share Files
Upload any local file:

```bash
tapeer send ./photos/vacation.jpg
```

---

### 4. Download or View Items (`get`)
The `get` command provides flexible ways to access shared content:

```bash
# Get the most recent item (no arguments needed!)
tapeer get

# Download the first item in the list
tapeer get 1

# Copy snippet #2 directly to your clipboard
tapeer get 2 -c

# Download matching file and save to a specific directory
tapeer get vacation.jpg -o downloads/

# Save snippet to a custom file
tapeer get 2 -o ./notes.txt
```

---

## ⚙️ Configuration (Base URL)

By default, TaPeer CLI connects to `http://localhost:3000`.

You can configure your server URL in two ways:

### Option A: Environment Variable (Recommended)
Set `TAPEER_URL` so you never have to type the server address:

- **Windows (PowerShell):**
  ```powershell
  [System.Environment]::SetEnvironmentVariable("TAPEER_URL", "https://your-server-address/tapeer", "User")
  ```

- **Linux / macOS (`~/.bashrc` or `~/.zshrc`):**
  ```bash
  export TAPEER_URL="https://your-server-address/tapeer"
  ```

### Option B: Command-line Flag
Override the server on the fly:

```bash
tapeer --server http://192.168.1.50:3000 list
# or using short flag:
tapeer -s https://my-server.ts.net/tapeer get 1
```

---

## 🛠️ Architecture

Built using modern, idiomatic Rust:

```text
src/
├── main.rs         # Entry point, CLI command dispatcher & rich terminal formatting
├── client.rs       # Async HTTP client built with reqwest and multipart support
└── models.rs       # Serde data structures representing TaPeer REST API payloads
```

- **[clap](https://crates.io/crates/clap)** (v4): Robust command-line argument parsing with derive macros and environment variable support.
- **[tokio](https://crates.io/crates/tokio)**: Asynchronous runtime for high-performance I/O.
- **[reqwest](https://crates.io/crates/reqwest)**: HTTP client with streaming and multipart upload support.
- **[serde](https://crates.io/crates/serde)**: High-speed JSON serialization and deserialization.
- **[arboard](https://crates.io/crates/arboard)**: Fast, native, cross-platform clipboard access (Windows, macOS, Wayland, X11).
- **[colored](https://crates.io/crates/colored)**: Clean ANSI terminal styling.
- **[anyhow](https://crates.io/crates/anyhow)**: Ergonomic and descriptive error handling.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
