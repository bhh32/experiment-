cosmic-applet-button = Cosmic Button

# Tab labels
tab-status = Status
tab-taildrop = Tail Drop
tab-exit-node = Exit Node
tab-devices = Devices
tab-serve = Serve
tab-subnets = Subnets
tab-settings = Settings

# Status tab
status-account = Account
status-new-login = New Login
status-ipv4 = IPv4 Address
status-ipv6 = IPv6 Address
status-connection = Status
status-connected = Connected
status-disconnected = Disconnected
status-enable-ssh = Enable SSH
status-accept-routes = Accept Routes
status-magic-dns = MagicDNS
status-connect-toggle = Connected
status-tailnet-lock = Tailnet Lock
status-lock-enabled-signed = Enabled (signed)
status-lock-enabled-unsigned = Enabled (unsigned)
status-lock-disabled = Disabled
status-pending-signatures = Pending Signatures

# Tail Drop tab
taildrop-target-device = Target Device
taildrop-selected-files = Selected Files
taildrop-no-files = No files selected
taildrop-select-files = Select File(s)
taildrop-send-files = Send File(s)
taildrop-receive-files = Receive File(s)
taildrop-transfer-status = Transfer Status
taildrop-download-directory = Download Directory
taildrop-files-sent = File(s) sent successfully!
taildrop-choose-device-first = Choose a device first, then select your file(s).
taildrop-select-device-files = Select a device and file(s) first.
taildrop-send-tooltip = Send the selected file(s).
taildrop-receive-tooltip = Receive files waiting in your Tail Drop inbox.
taildrop-received = Received file(s) in { $path }

# Exit Node tab
exit-node-selected = Selected Node
exit-node-enable-host = Enable Host Exit Node
exit-node-disable-host = Disable Host Exit Node
exit-node-allow-lan = Allow LAN Access
exit-node-cant-select = Can't select an exit node while host is an exit node!

# Devices tab
devices-title = Devices on Tailnet
devices-select-prompt = Select a device above to see details.
devices-not-found = Device not found
devices-this-device = (this device)
devices-name = Name
devices-dns-name = DNS Name
devices-ip-address = IP Address
devices-os = OS
devices-online = Online
devices-yes = Yes
devices-no = No
devices-exit-node = Exit Node
devices-tags = Tags
devices-tags-none = None
devices-relay = Relay
devices-traffic = Traffic
devices-last-seen = Last Seen
devices-last-seen-now = Now
devices-ping = Ping
devices-pinging = Pinging...
devices-copy-dns = Copy DNS name
devices-copy-ip = Copy IP
devices-copy-clipboard = Copy to clipboard

# Serve tab
serve-title = Tailscale Serve
serve-no-entries = No active serve entries.
serve-add-title = Add New Serve Entry
serve-port-placeholder = Port (e.g. 3000)
serve-path-placeholder = Path (e.g. /)
serve-add = Add
serve-remove = Remove
serve-refresh = Refresh
serve-funnel = (Funnel)

# Subnets tab
subnets-title = Subnet Routes
subnets-no-routes = No advertised subnet routes.
subnets-add-title = Add Subnet Route
subnets-cidr-placeholder = CIDR (e.g. 192.168.1.0/24)
subnets-add = Add
subnets-remove = Remove

# Settings tab
settings-auto-connect = Auto-connect on startup
settings-dynamic-icon = Dynamic panel icon
settings-download-dir = Download Directory
settings-download-dir-default = ~/Downloads (default)
settings-download-dir-change = Change
settings-poll-interval = Poll Interval (seconds)
settings-notifications-title = Notifications
settings-notifications-enabled = Enable Notifications
settings-notify-connection = Connection Changes
settings-notify-files = Incoming Files
settings-notify-devices = New Devices

# Health / Error states
health-not-installed-title = Tailscale Not Installed
health-not-installed-body = The tailscale CLI tool was not found.
health-not-installed-hint = Install Tailscale: https://tailscale.com/download/linux
health-daemon-down-title = Tailscale Daemon Not Running
health-daemon-down-body = The tailscaled service is not running.
health-daemon-down-hint = Start it with: sudo systemctl start tailscaled
health-no-operator-title = Operator Permission Required
health-no-operator-body = The tailscale operator is not set for your user.
health-no-operator-hint = Run: sudo tailscale set --operator=$USER
health-error-title = Tailscale Error
health-error-hint = Check your Tailscale installation.

# File chooser
file-chooser-title = Choose a file or files...
dir-chooser-title = Choose download directory
