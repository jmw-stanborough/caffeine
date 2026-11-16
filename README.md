# Caffeine

Cross-platform network monitoring, endpoint management, and IT workspace desktop application built with Tauri v2, SvelteKit, TypeScript, and Rust.

## Core Modules

### Network Monitoring
- **Dashboard Overview**: Unified operational summary across monitored infrastructure and alerts.
- **Server Monitor**:
  - Continuous ICMP/TCP reachability probes with optional custom TCP probe ports.
  - Integrated remote SSH terminal powered by `portable-pty` and native RDP session launcher.
  - Status transition history logs and full-screen NOC display mode.
- **Printer Dashboard**:
  - SNMP v2c polling for supply/toner telemetry, page counters, and printer status alerts.
  - Tag-based filtering and an embedded native child webview for each printer's management console.

### Endpoint Management
- **Hardware & System Telemetry**: Live monitoring of CPU, memory, disk volumes, network interfaces, and OS information.
- **OS Updates**: Check for, download, and install operating system updates, with restart prompts when required.
- **Filesystem Explorer & Search**: Multi-volume file browser and search with saved filter presets and batch delete support.
- **Local Terminal**: Integrated local PTY terminal (`@xterm/xterm` + `portable-pty`).

### IT Workspace
- **Todo Manager**: Task tracking with due-date warning banners and snoozeable desktop notifications.
- **Markdown Notes**: Note editor with live preview and syntax highlighting.
- **Script Library**: Store, format, and execute local scripts via the integrated script runner and formatter.

---

## OS Prerequisites

- **Administrator / `sudo` Privileges**:
  - Required to install OS updates.
  - Required to save machine-wide Server Mode settings and store the shared server database in system storage:
    - **Windows**: `C:\ProgramData\caffeine` (or `%ProgramData%\caffeine`)
    - **macOS**: `/Library/Application Support/caffeine`
    - **Linux**: `/var/lib/caffeine`
- **Remote Access Clients**:
  - **SSH**: An OpenSSH client (`ssh`) available on `PATH` for remote server terminals.
  - **RDP**: Native RDP client support:
    - **Windows**: `mstsc`
    - **macOS**: `open rdp://` (Microsoft Remote Desktop / Windows App URL handler)
    - **Linux**: `remmina` or `xfreerdp` (`xfreerdp3`, `sdl-freerdp`, `xfreerdp`)
- **Network / Firewall Access**:
  - Outbound **UDP port 161** access from the polling host to managed printers for SNMP v2c telemetry.
  - Inbound **TCP port 6310** (or your configured Server Port) on the Caffeine Server host when serving remote clients.

---

## Client / Server Mode & HMAC Token Setup

Caffeine can run standalone (`127.0.0.1`) or in a distributed Client/Server topology where one host runs probes and polling while remote workstations connect to view or manage resources.

### 1. Enabling Server Mode (Host)
1. Open **Settings -> Network**.
2. Enable **Enable Caffeine Server Mode** (requires Administrator / `sudo` privileges to write machine-wide settings).
3. Configure the **Bind Address** (defaults to `0.0.0.0`) and **Server Port** (defaults to `6310`).
4. Under **Admin Access Tokens**, create one or more named tokens. Generated tokens use the `caff_sec_...` format. Copy the token when created, as masked tokens are shown afterward.
5. Save settings to start the embedded HTTP/WebSocket server.

### 2. Connecting Remote Clients
1. On a client workstation, open **Settings -> Network**.
2. Set **Target Host / IP** and **Target Port** (default `6310`) to match the Caffeine Server instance.
3. Choose an access level:
   - **Read-Only Mode**: Leave **Admin Access Token** blank. The client receives live status updates and snapshots without write permissions. Sensitive credentials (such as printer SNMP community strings and server agent keys) are redacted on public snapshot endpoints.
   - **Authenticated Admin Mode**: Paste a `caff_sec_...` admin token into **Admin Access Token**. Mutating API requests are signed using **HMAC-SHA256** (`X-Caffeine-Timestamp`, `X-Caffeine-Nonce`, and `X-Caffeine-Signature` headers) with a 60-second clock-drift window and nonce cache to prevent replay attacks; plaintext `Authorization` Bearer tokens are rejected.

---

## Development, Testing, Versioning & Release

### Prerequisites
- [Node.js](https://nodejs.org/) 22+ and [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/) (stable toolchain)
- Platform dependencies for Tauri v2 (on Ubuntu/Debian: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`)

### Common Commands
```bash
# Install frontend dependencies
pnpm install

# Run the desktop app in development mode
pnpm tauri dev

# Run frontend unit tests (Vitest)
pnpm test

# Run SvelteKit sync and TypeScript/Svelte type checking
pnpm check

# Run Rust backend unit and integration tests
cargo test --manifest-path src-tauri/Cargo.toml

# Build production desktop installers
pnpm tauri build
```

### Version Management & Releases
Version numbers are synchronized across `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and the sidebar UI badge:

```bash
# Increment patch version (X.Y.Z -> X.Y.Z+1)
pnpm version:patch

# Set an explicit semantic version across all manifests
pnpm version:set <X.Y.Z>

# Verify all manifests match, create tag v<X.Y.Z>, and push it to origin
pnpm release:tag
```

Pushing a `v*` tag triggers the GitHub Actions release workflow (`.github/workflows/release.yml`), which runs all checks/tests across Windows, Linux (`ubuntu-22.04`), and macOS and creates a draft GitHub Release with the built installers.
