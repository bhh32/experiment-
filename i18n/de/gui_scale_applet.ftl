cosmic-applet-button = Cosmic-Taste

# Registerkarten
tab-status = Status
tab-taildrop = Tail Drop
tab-exit-node = Ausgangsknoten
tab-devices = Geräte
tab-serve = Serve
tab-subnets = Subnetze
tab-settings = Einstellungen

# Status-Registerkarte
status-account = Konto
status-new-login = Neue Anmeldung
status-ipv4 = IPv4-Adresse
status-ipv6 = IPv6-Adresse
status-connection = Status
status-connected = Verbunden
status-disconnected = Getrennt
status-enable-ssh = SSH aktivieren
status-accept-routes = Routen akzeptieren
status-magic-dns = MagicDNS
status-connect-toggle = Verbunden
status-tailnet-lock = Tailnet Lock
status-lock-enabled-signed = Aktiviert (signiert)
status-lock-enabled-unsigned = Aktiviert (unsigniert)
status-lock-disabled = Deaktiviert
status-pending-signatures = Ausstehende Signaturen

# Tail Drop-Registerkarte
taildrop-target-device = Zielgerät
taildrop-selected-files = Ausgewählte Dateien
taildrop-no-files = Keine Dateien ausgewählt
taildrop-select-files = Datei(en) auswählen
taildrop-send-files = Datei(en) senden
taildrop-receive-files = Datei(en) empfangen
taildrop-transfer-status = Übertragungsstatus
taildrop-download-directory = Download-Verzeichnis
taildrop-files-sent = Datei(en) erfolgreich gesendet!
taildrop-choose-device-first = Wählen Sie zuerst ein Gerät und dann Ihre Dateien.
taildrop-select-device-files = Wählen Sie zuerst ein Gerät und Datei(en).
taildrop-send-tooltip = Die ausgewählten Dateien senden.
taildrop-receive-tooltip = Dateien empfangen, die in Ihrem Tail Drop-Posteingang warten.
taildrop-received = Datei(en) empfangen in { $path }

# Ausgangsknoten-Registerkarte
exit-node-selected = Ausgewählter Knoten
exit-node-enable-host = Host-Ausgangsknoten aktivieren
exit-node-disable-host = Host-Ausgangsknoten deaktivieren
exit-node-allow-lan = LAN-Zugriff erlauben
exit-node-cant-select = Kann keinen Ausgangsknoten auswählen, während der Host ein Ausgangsknoten ist!

# Geräte-Registerkarte
devices-title = Geräte im Tailnet
devices-select-prompt = Wählen Sie ein Gerät oben für Details.
devices-not-found = Gerät nicht gefunden
devices-this-device = (dieses Gerät)
devices-name = Name
devices-dns-name = DNS-Name
devices-ip-address = IP-Adresse
devices-os = Betriebssystem
devices-online = Online
devices-yes = Ja
devices-no = Nein
devices-exit-node = Ausgangsknoten
devices-tags = Tags
devices-tags-none = Keine
devices-relay = Relay
devices-traffic = Datenverkehr
devices-last-seen = Zuletzt gesehen
devices-last-seen-now = Jetzt
devices-ping = Ping
devices-pinging = Ping läuft...
devices-copy-dns = DNS-Name kopieren
devices-copy-ip = IP kopieren
devices-copy-clipboard = In Zwischenablage kopieren

# Serve-Registerkarte
serve-title = Tailscale Serve
serve-no-entries = Keine aktiven Serve-Einträge.
serve-add-title = Neuen Serve-Eintrag hinzufügen
serve-port-placeholder = Port (z.B. 3000)
serve-path-placeholder = Pfad (z.B. /)
serve-add = Hinzufügen
serve-remove = Entfernen
serve-refresh = Aktualisieren
serve-funnel = (Funnel)

# Subnetze-Registerkarte
subnets-title = Subnetzrouten
subnets-no-routes = Keine beworbenen Subnetzrouten.
subnets-add-title = Subnetzroute hinzufügen
subnets-cidr-placeholder = CIDR (z.B. 192.168.1.0/24)
subnets-add = Hinzufügen
subnets-remove = Entfernen

# Einstellungen-Registerkarte
settings-auto-connect = Automatisch verbinden beim Start
settings-dynamic-icon = Dynamisches Panel-Symbol
settings-download-dir = Download-Verzeichnis
settings-download-dir-default = ~/Downloads (Standard)
settings-download-dir-change = Ändern
settings-poll-interval = Abfrageintervall (Sekunden)
settings-notifications-title = Benachrichtigungen
settings-notifications-enabled = Benachrichtigungen aktivieren
settings-notify-connection = Verbindungsänderungen
settings-notify-files = Eingehende Dateien
settings-notify-devices = Neue Geräte

# Gesundheits- / Fehlerzustände
health-not-installed-title = Tailscale nicht installiert
health-not-installed-body = Das CLI-Tool tailscale wurde nicht gefunden.
health-not-installed-hint = Tailscale installieren: https://tailscale.com/download/linux
health-daemon-down-title = Tailscale-Daemon läuft nicht
health-daemon-down-body = Der tailscaled-Dienst läuft nicht.
health-daemon-down-hint = Starten mit: sudo systemctl start tailscaled
health-no-operator-title = Operator-Berechtigung erforderlich
health-no-operator-body = Der Tailscale-Operator ist nicht für Ihren Benutzer eingestellt.
health-no-operator-hint = Ausführen: sudo tailscale set --operator=$USER
health-error-title = Tailscale-Fehler
health-error-hint = Überprüfen Sie Ihre Tailscale-Installation.

# Allgemein
select-default = Auswählen
none-default = Keine
copy-tooltip = Kopieren

# Dateiauswahl
file-chooser-title = Datei oder Dateien auswählen...
dir-chooser-title = Download-Verzeichnis auswählen

# Zusätzlicher Status
status-dns-suffix = DNS-Suffix

# Zusätzliches Tail Drop
taildrop-inbox = Posteingang
taildrop-no-waiting = Keine Dateien warten.
taildrop-waiting-count = { $count } Datei(en) im Posteingang
taildrop-choose-device = Wählen Sie zuerst ein Gerät.
