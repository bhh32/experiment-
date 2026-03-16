# Advanced Feature Recommendations for GUI Scale Applet

After a thorough analysis of the codebase (`src/main.rs`, `src/window.rs`, `src/logic.rs`, `src/config.rs`), the build system, and the Tailscale CLI capabilities, the following advanced features are recommended. They are grouped by category and ordered by impact.

---

## 1. Real-Time Status Monitoring & Notifications

**Problem:** The applet only queries Tailscale state at startup (`init()`) and on certain user actions. Worse, `get_tailscale_ip()` and `get_tailscale_con_status()` are called synchronously inside `view_window()` on every render, which blocks the UI thread.

**Recommendations:**

- **Background Polling via Subscription:** Implement `cosmic::Application::subscription()` to poll Tailscale status on a timed interval (e.g., every 5-10 seconds). This replaces the synchronous calls in `view_window()` and keeps the UI responsive while reflecting real-time state changes (peer connects/disconnects, IP changes, connection drops).

- **Desktop Notifications:** Use `cosmic-notifications` or `notify-rust` to alert the user when:
  - Connection drops or reconnects
  - A new device joins the tailnet
  - Incoming Tail Drop file transfer is waiting
  - Exit node becomes unreachable
  - Account authentication expires

- **Dynamic Panel Icon:** Change the panel icon based on connection state (e.g., green/connected, red/disconnected, yellow/connecting). This gives at-a-glance status without opening the popup.

---

## 2. Tailscale Serve & Funnel Support

**Problem:** Tailscale Serve (expose local services to your tailnet) and Funnel (expose to the public internet) are powerful features with no GUI in this applet.

**Recommendations:**

- **Serve Management Panel:** Add a UI section to:
  - List active `tailscale serve` entries (parsed from `tailscale serve status --json`)
  - Add new serve rules: local port -> tailnet HTTPS path
  - Remove/toggle individual serve entries
  - Show the tailnet URL for each served service

- **Funnel Toggle:** Add a toggler to enable/disable Funnel for each served port, with a clear warning that this exposes the service to the public internet.

- **Serve Status Display:** Show a list of currently served ports with their mapped URLs directly in the popup, similar to how exit nodes are displayed.

---

## 3. Device Details & Network Topology View

**Problem:** The device list (`get_tailscale_devices()`) only shows device names in a dropdown for Tail Drop. There is no way to inspect device details.

**Recommendations:**

- **Device Info Panel:** When a device is selected, show:
  - Tailscale IP address (v4 and v6)
  - OS type and hostname
  - Online/offline status (with last-seen timestamp)
  - Whether it offers exit node capability
  - Tags applied to the device
  - All data available from `tailscale status --json`

- **Network Topology View:** A scrollable list showing all tailnet devices with their status indicators (online/offline dots), organized by user or tag. This replaces the bare dropdown with a richer device browser.

- **Ping/Latency Check:** Add a "Ping" button per device that runs `tailscale ping <device>` and displays the latency and connection path (direct vs. DERP relay).

---

## 4. MagicDNS Integration

**Problem:** MagicDNS is a core Tailscale feature but the applet has no visibility into it.

**Recommendations:**

- **DNS Name Display:** Show the MagicDNS name (e.g., `device.tailnet-name.ts.net`) for each device alongside its IP.

- **Copy-to-Clipboard:** One-click copy of any device's MagicDNS hostname or IP address. Useful for SSH commands, browser URLs, etc.

- **DNS Status:** Show whether MagicDNS is enabled/disabled, and provide a toggle to control it via `tailscale set --accept-dns`.

---

## 5. Subnet Router Management

**Problem:** The applet handles exit nodes but ignores subnet routers, which are a major Tailscale enterprise feature.

**Recommendations:**

- **Advertise Subnets:** Allow the user to add/remove subnet routes that this machine advertises via `tailscale set --advertise-routes=<CIDR>`.

- **View Accepted Subnets:** Display which subnet routes are being accepted from other devices on the tailnet.

- **Subnet Status:** Show which subnet routers are online and which subnets they expose.

---

## 6. File Transfer Improvements

**Problem:** The current Tail Drop implementation has several UX gaps:
- No progress indication during transfers
- Hardcoded download path (`/home/{user}/Downloads/`)
- No drag-and-drop support
- `clear_status()` uses `thread::sleep()` inside an async fn, which blocks the tokio runtime thread

**Recommendations:**

- **Transfer Progress Bar:** Parse `tailscale file cp` output for progress information and display a progress bar or percentage in the UI.

- **Configurable Download Directory:** Let users choose their preferred download directory through a settings dialog, persisted via the existing config system.

- **Drag-and-Drop File Sending:** Support dragging files onto the applet popup to initiate a Tail Drop send.

- **Transfer History:** Maintain a brief log of recent transfers (file name, device, timestamp, success/failure) viewable from the popup.

- **Fix `clear_status()`:** Replace `thread::sleep()` with `tokio::time::sleep()` in `logic.rs:211` to avoid blocking the async runtime:
  ```rust
  pub async fn clear_status(wait_time: u64) -> Option<String> {
      tokio::time::sleep(Duration::from_secs(wait_time)).await;
      None
  }
  ```

- **Incoming File Queue:** Show pending incoming files (from `tailscale file get --wait`) with accept/reject options instead of blindly downloading everything.

---

## 7. ACL & Network Lock (Tailnet Lock) Visibility

**Problem:** No visibility into access control or network lock status.

**Recommendations:**

- **Tailnet Lock Status:** Display whether Tailnet Lock is enabled and whether this node is signed, using `tailscale lock status`.

- **Lock Signing Requests:** If Tailnet Lock is enabled, show pending signing requests and allow the user to sign them from the applet.

- **ACL Tag Display:** Show which ACL tags are applied to this device, helping users understand their access permissions.

---

## 8. Multi-Account UX Improvements

**Problem:** Account switching works but refreshes state synchronously, blocking the UI. The account list shows raw tailnet names with no visual distinction.

**Recommendations:**

- **Async Account Switching:** Move `switch_accounts()` and subsequent state refresh calls in `Message::SwitchAccount` to async tasks to prevent UI freezing.

- **Account Status Indicators:** Show which accounts are logged in vs. expired, using data from `tailscale switch --list`.

- **Quick Login:** Add a "Log in to new account" button that opens `tailscale login` in the browser.

- **Account Badges:** Show the current account prominently at the top of the popup with the tailnet name and user identity.

---

## 9. Keyboard Shortcuts & Accessibility

**Problem:** The applet is mouse-only with no keyboard shortcuts or accessibility features.

**Recommendations:**

- **Global Hotkey:** Register a system-wide keyboard shortcut to toggle the popup (e.g., `Super+T`).

- **Keyboard Navigation:** Ensure all togglers, dropdowns, and buttons are keyboard-focusable and operable with Enter/Space.

- **Screen Reader Labels:** Add accessible labels to all UI elements for assistive technology compatibility, using COSMIC's AccessKit integration.

---

## 10. Internationalization Expansion

**Problem:** Only 3 languages (English, Dutch, Swedish) with a single translated string (`cosmic-applet-button`). None of the UI text in `window.rs` is localized.

**Recommendations:**

- **Localize All UI Strings:** Extract all hardcoded strings from `window.rs` into Fluent `.ftl` files:
  - "Account", "Tailscale Address", "Connection Status"
  - "Enable SSH", "Accept Routes", "Connected"
  - "Tail Drop", "Select File(s)", "Send File(s)", "Receive File(s)"
  - "Exit Node", "Selected Node", "Enable Host Exit Node", "Allow LAN Access"
  - All status messages and error text

- **Add More Languages:** Contribute translations for common languages (Spanish, French, German, Portuguese, Japanese, Chinese, etc.).

- **Fix Typos in i18n:** The `i18n.toml` has `fallback_langage` (should be `fallback_language` — though this may be the expected key name for the i18n-embed crate).

---

## 11. Settings Persistence & Preferences Panel

**Problem:** Only `exit-node` and `allow-lan` are persisted. Many user preferences are lost on restart.

**Recommendations:**

- **Persist All Toggle States:** Save SSH, routes, and connection preferences to the config file so the applet can restore them after a restart/crash.

- **Preferences Dialog:** Add a settings gear icon that opens a configuration panel for:
  - Default download directory for Tail Drop
  - Polling interval for status updates
  - Notification preferences (which events trigger notifications)
  - Panel icon style preference
  - Auto-connect on startup option

---

## 12. Error Handling & Resilience

**Problem:** Extensive use of `.unwrap()` throughout `logic.rs` means the applet will panic if Tailscale is not installed, not running, or returns unexpected output. The README claims "comprehensive error handling" but the implementation doesn't match.

**Recommendations:**

- **Graceful Degradation:** Replace all `.unwrap()` calls with proper error handling. If Tailscale is not installed or the daemon is not running, show a friendly message in the popup instead of crashing.

- **Tailscale Installation Check:** On startup, verify that the `tailscale` binary exists and is accessible. If not, display installation instructions in the popup.

- **Operator Permission Check:** Verify that `--operator=$USER` has been set. If not, show a one-time setup prompt with the command to run.

- **Retry Logic:** For transient CLI failures, implement retry with backoff rather than crashing.

---

## Summary Priority Matrix

| Priority | Feature | Impact | Effort |
|----------|---------|--------|--------|
| **P0** | Fix `clear_status()` blocking async runtime | Bug fix | Low |
| **P0** | Remove synchronous CLI calls from `view_window()` | Performance | Medium |
| **P0** | Replace `.unwrap()` with proper error handling | Stability | Medium |
| **P1** | Real-time status monitoring via subscriptions | UX | Medium |
| **P1** | Dynamic panel icon (connected/disconnected) | UX | Low |
| **P1** | Desktop notifications | UX | Medium |
| **P1** | Localize all UI strings | i18n | Medium |
| **P2** | Device details panel with ping | Feature | Medium |
| **P2** | MagicDNS integration & copy-to-clipboard | Feature | Low |
| **P2** | File transfer progress & configurable download dir | UX | Medium |
| **P2** | Settings persistence & preferences panel | UX | Medium |
| **P3** | Tailscale Serve & Funnel support | Feature | High |
| **P3** | Subnet router management | Feature | High |
| **P3** | Tailnet Lock visibility | Feature | Medium |
| **P3** | Keyboard shortcuts & accessibility | a11y | Medium |
| **P3** | Multi-account UX improvements | UX | Low |
