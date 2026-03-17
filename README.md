# GUI Scale COSMIC Desktop Applet

## About

A secure, memory-safe implementation of a Tailscale GUI management applet for the System76 COSMIC desktop environment. This project demonstrates secure systems programming practices while providing comprehensive functionality for Tailscale network (tailnet) management.

### Project Overview

The GUI Scale applet provides a user-friendly interface for managing Tailscale configurations within the COSMIC desktop environment. It showcases secure systems programming practices while maintaining high usability standards.

### Key Features

- **Flatpak-Safe Architecture** — Communicates directly with the `tailscaled` daemon via its LocalAPI Unix socket (`/var/run/tailscale/tailscaled.sock`) using HTTP-over-UDS. No dependency on the `tailscale` CLI binary — works inside Flatpak sandboxes
- **Tabbed Interface** — Organized UI with tabs for Status, Tail Drop, Exit Node, Devices, and Settings
- **Real-time Status Monitoring** — Periodic background polling keeps connection state, device list, and IP addresses up to date without blocking the UI
- **Connection Management** — Connect/disconnect, enable SSH, accept routes, toggle MagicDNS
- **Multi-Account Support** — Switch between Tailscale accounts, log in to new accounts
- **Tail Drop File Transfer** — Send and receive files between tailnet devices with configurable download directory
- **Exit Node Management** — Select exit nodes, advertise host as exit node, toggle LAN access
- **Device Browser** — View all tailnet devices with detailed info (IP, DNS name, OS, online status, tags, traffic stats, relay, last seen) and one-click ping with latency display
- **MagicDNS Integration** — View and copy MagicDNS hostnames, toggle MagicDNS on/off
- **Tailscale Serve & Funnel** — Add/remove serve entries, toggle Funnel for public exposure
- **Subnet Router Management** — Advertise and manage subnet routes
- **Tailnet Lock Visibility** — View lock status, signed/unsigned state, pending signatures
- **Desktop Notifications** — Alerts for connection changes, new devices, incoming files, account switches
- **Copy to Clipboard** — One-click copy of IP addresses and DNS names (Wayland via wl-copy)
- **Persistent Preferences** — All settings saved across sessions (auto-connect, download dir, poll interval, notification prefs, icon style)
- **Graceful Error Handling** — Friendly messages when Tailscale is not installed, daemon is down, or operator permission is missing
- Memory-safe implementation in Rust
    - Utilizes Rust's ownership system for memory-safety operations
    - No unsafe code blocks used
    - All external data properly validated
    - Buffer overflow protection through Rust's bounds checking

### Security Architecture

The applet implements a layered security approach:

1. Privilege Management
    - Minimal required permissions
    - Proper capability isolation
    - Startup check for operator permission
2. Error Handling
    - All LocalAPI interactions wrapped in Result types — no panics
    - Graceful degradation when daemon is unavailable
    - No information leakage

## Dependencies

You must have `tailscaled` running and set the operator permission:

```bash
sudo systemctl start tailscaled
sudo tailscale set --operator=$USER
```

This makes it where the applet doesn't need sudo (root) to do its job.

**Note:** The applet does NOT require the `tailscale` CLI binary. It communicates directly with the `tailscaled` daemon via its Unix socket LocalAPI.

### System Dependencies

- `tailscaled` (the Tailscale daemon — must be running)
- `wl-copy` (for clipboard support on Wayland, optional)

### Flatpak

The applet is Flatpak-compatible. It needs access to the tailscaled socket:

```bash
# Grant socket access to the Flatpak app
flatpak override --user com.github.bhh32.GUIScaleApplet --filesystem=/var/run/tailscale
```

Or in your Flatpak manifest's `finish-args`:

```json
"--filesystem=/var/run/tailscale"
```

## Screenshots

![gui-scale-applet-panel](/screenshots/gui-scale-panel.png)
![gui-scale-applet-open](/screenshots/gui-scale-applet-open.png)

## Installation

### Fedora/Fedora based distros

Add the Copr repo:

```bash
sudo dnf copr enable bhh32/gui-scale-applet
sudo dnf update --refresh
sudo dnf install -y gui-scale-applet
```

### Debian/Ubuntu (including Pop!OS) based Distros

Unfortunately, I don't know anything like Copr for these distros, so you can download the deb package from the releases section of this repo.

### Other

For any other distros (except atomic/immutable distros) you can run:

```bash
git clone https://github.com/cosmic-utils/gui-scale-applet.git
cd gui-scale-applet
sudo just install
```

## Architecture

```
src/
├── main.rs           # Entry point, launches COSMIC applet
├── tailscale_api.rs  # LocalAPI client (HTTP-over-Unix-socket to tailscaled)
├── logic.rs          # High-level Tailscale operations built on the API client
├── window.rs         # UI state, message handling, tabbed views, subscriptions
├── config.rs         # Persistent preferences via COSMIC config API
└── notifications.rs  # Desktop notifications via notify-rust
```

### Communication Architecture

```
┌──────────────────┐    HTTP/1.1 over     ┌──────────────┐
│  GUI Scale       │ ──────────────────── │  tailscaled   │
│  Applet          │    Unix socket       │  daemon       │
│  (this app)      │                      │               │
│                  │  /localapi/v0/...    │  manages the  │
│  tailscale_api.rs│  (JSON req/resp)     │  WireGuard    │
│  → hyper client  │                      │  tunnel       │
└──────────────────┘                      └──────────────┘
        │
        └── /var/run/tailscale/tailscaled.sock
```

No `tailscale` CLI binary is spawned. All operations use structured JSON over the LocalAPI.

### Tabs

| Tab | Description |
|-----|-------------|
| **Status** | Account, IPs, connection toggles, SSH, routes, MagicDNS, subnet routes |
| **Tail Drop** | File send/receive with device selection, inbox, and status |
| **Exit Node** | Exit node selection, host exit node toggle, LAN access |
| **Devices** | Full device browser with details panel and ping |
| **Settings** | Auto-connect, notifications, download dir, poll interval, icon style |
