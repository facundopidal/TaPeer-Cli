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
- 📋 **Clipboard Integration**: Copy text snippets or downloaded file paths with `-c` / `--copy`, or explicitly share plain-text clipboard contents with `text --clipboard`.
- 📁 **Smart Folder Output**: Save files into folders directly (`-o downloads/`), with automatic directory creation if they don't exist yet.
- 🎨 **Rich Terminal Output**: Colored badges, human-readable file sizes (`KB`, `MB`), and real-time expiration countdowns.
- 🌐 **Flexible Networking**: Works out of the box with `localhost`, local Wi-Fi networks, and private mesh VPNs like **Tailscale**.
- 🐧 **Cross-Platform**: Windows, macOS, and Linux clipboard adapters. Linux desktop runtime verification is described below.

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

# Share the clipboard as plain text without placing its contents in shell arguments
tapeer text --clipboard
```

Choose exactly one source: positional text **or** `--clipboard`. Clipboard input is never
implicit, previewed, executed, cleared, or replaced with another source on failure.
Empty, non-text, unavailable, or invalid UTF-8 clipboard data fails before uploading.
Unicode, quotes, whitespace, and newlines are preserved; positional `""` remains accepted.
`--stdin` is not supported. `send` still uploads files only.

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

For files, `--copy` copies the downloaded **path**, not the file contents. A failed
path copy returns an error but leaves the successfully downloaded file intact.

### Linux clipboard lifetime

No `wl-copy`, `wl-paste`, or `xclip` executable is required. When Wayland environment
variables are present, TaPeer uses native Wayland data-control (ext or wlr); it does
not silently fall back to XWayland if the compositor or protocol is unavailable.
Otherwise it uses X11, requiring the XFixes extension.

Copying starts a detached helper instance of the **same binary**. Text passes through
private pipes, never helper arguments, environment variables, logs, or temporary files.
Success requires a native served readback: a unique private MIME probe on Wayland
(normal plain-text MIME types remain offered), or verified ownership and text on X11.
Startup and parent acceptance are bounded; failure terminates and reaps the helper.
After acceptance there is no lifetime timer: the helper serves until replacement,
session disconnection, or a backend error. Clipboard managers may separately retain history.

#### Verify on the actual desktop

Run these checks locally, **not over SSH**, using the newly built binary and a test server.
Compilation and unit tests on Windows do not prove Hyprland or X11 clipboard behavior.

1. **Hyprland:** keep your normal `WAYLAND_DISPLAY` / `XDG_RUNTIME_DIR` and run
   `tapeer get <text-item> --copy`. Wait for the shell prompt, then paste twice into
   a native Wayland editor. Confirm exact multiline Unicode text after the CLI exits.
2. **X11:** in a real X11 session without Wayland variables, repeat the same check.
   Confirm paste after exit and replace the clipboard in the editor; the old
   `tapeer --clipboard-helper` process should exit. Repeat replacement on Hyprland.
3. Copy a multiline snippet with quotes and trailing whitespace in the editor, run
   `tapeer text --clipboard`, and compare the server's stored text byte-for-byte.
   No clipboard content should appear in CLI output. Try an empty clipboard and
   an image-only clipboard: both must fail without a new server item.
4. Run `tapeer get <file-item> --copy -o downloads/`: confirm the file exists and
   pasting yields its path. With invalid display settings, copying must fail rather
   than print copy success; a file already downloaded must remain on disk.
5. Inspect process arguments and the current directory during copying: no clipboard
   payload or temporary payload file should appear. After clipboard replacement or
   session shutdown, confirm no obsolete helper remains.

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
├── clipboard.rs    # Shared clipboard adapter; Linux helper/protocol live in clipboard/
├── client.rs       # Async HTTP client built with reqwest and multipart support
└── models.rs       # Serde data structures representing TaPeer REST API payloads
```

- **[clap](https://crates.io/crates/clap)** (v4): Robust command-line argument parsing with derive macros and environment variable support.
- **[tokio](https://crates.io/crates/tokio)**: Asynchronous runtime for high-performance I/O.
- **[reqwest](https://crates.io/crates/reqwest)**: HTTP client with streaming and multipart upload support.
- **[serde](https://crates.io/crates/serde)**: High-speed JSON serialization and deserialization.
- **[arboard](https://crates.io/crates/arboard)**: Windows/macOS clipboard access and X11 text reading.
- **[wl-clipboard-rs](https://crates.io/crates/wl-clipboard-rs)** and **[x11-clipboard](https://crates.io/crates/x11-clipboard)**: Native Linux clipboard serving, packaged in the same executable.
- **[colored](https://crates.io/crates/colored)**: Clean ANSI terminal styling.
- **[anyhow](https://crates.io/crates/anyhow)**: Ergonomic and descriptive error handling.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
