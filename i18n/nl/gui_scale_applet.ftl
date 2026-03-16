cosmic-applet-button = COSMIC-knop

# Tablabels
tab-status = Status
tab-taildrop = Tail Drop
tab-exit-node = Uitgangsnode
tab-devices = Apparaten
tab-serve = Serve
tab-subnets = Subnetten
tab-settings = Instellingen

# Status tab
status-account = Account
status-new-login = Nieuw inloggen
status-ipv4 = IPv4-adres
status-ipv6 = IPv6-adres
status-connection = Status
status-connected = Verbonden
status-disconnected = Niet verbonden
status-enable-ssh = SSH inschakelen
status-accept-routes = Routes accepteren
status-magic-dns = MagicDNS
status-connect-toggle = Verbonden
status-tailnet-lock = Tailnet Lock
status-lock-enabled-signed = Ingeschakeld (ondertekend)
status-lock-enabled-unsigned = Ingeschakeld (niet ondertekend)
status-lock-disabled = Uitgeschakeld
status-pending-signatures = Wachtende ondertekeningen

# Tail Drop tab
taildrop-target-device = Doelapparaat
taildrop-selected-files = Geselecteerde bestanden
taildrop-no-files = Geen bestanden geselecteerd
taildrop-select-files = Bestand(en) selecteren
taildrop-send-files = Bestand(en) verzenden
taildrop-receive-files = Bestand(en) ontvangen
taildrop-transfer-status = Overdrachtsstatus
taildrop-download-directory = Downloadmap
taildrop-files-sent = Bestand(en) succesvol verzonden!
taildrop-choose-device-first = Kies eerst een apparaat en selecteer dan uw bestand(en).
taildrop-select-device-files = Selecteer eerst een apparaat en bestand(en).
taildrop-send-tooltip = Verzend de geselecteerde bestanden.
taildrop-receive-tooltip = Ontvang bestanden die wachten in uw Tail Drop inbox.
taildrop-received = Bestand(en) ontvangen in { $path }

# Exit Node tab
exit-node-selected = Geselecteerde node
exit-node-enable-host = Hostuitgangsnode inschakelen
exit-node-disable-host = Hostuitgangsnode uitschakelen
exit-node-allow-lan = LAN-toegang toestaan
exit-node-cant-select = Kan geen uitgangsnode selecteren terwijl host een uitgangsnode is!

# Devices tab
devices-title = Apparaten op Tailnet
devices-select-prompt = Selecteer een apparaat hierboven voor details.
devices-not-found = Apparaat niet gevonden
devices-this-device = (dit apparaat)
devices-name = Naam
devices-dns-name = DNS-naam
devices-ip-address = IP-adres
devices-os = Besturingssysteem
devices-online = Online
devices-yes = Ja
devices-no = Nee
devices-exit-node = Uitgangsnode
devices-tags = Tags
devices-tags-none = Geen
devices-relay = Relay
devices-traffic = Verkeer
devices-last-seen = Laatst gezien
devices-last-seen-now = Nu
devices-ping = Ping
devices-pinging = Pingen...
devices-copy-dns = DNS-naam kopiëren
devices-copy-ip = IP kopiëren
devices-copy-clipboard = Kopiëren naar klembord

# Serve tab
serve-title = Tailscale Serve
serve-no-entries = Geen actieve serve-items.
serve-add-title = Nieuw serve-item toevoegen
serve-port-placeholder = Poort (bijv. 3000)
serve-path-placeholder = Pad (bijv. /)
serve-add = Toevoegen
serve-remove = Verwijderen
serve-refresh = Vernieuwen
serve-funnel = (Funnel)

# Subnets tab
subnets-title = Subnetroutes
subnets-no-routes = Geen geadverteerde subnetroutes.
subnets-add-title = Subnetroute toevoegen
subnets-cidr-placeholder = CIDR (bijv. 192.168.1.0/24)
subnets-add = Toevoegen
subnets-remove = Verwijderen

# Settings tab
settings-auto-connect = Automatisch verbinden bij opstarten
settings-dynamic-icon = Dynamisch paneelpictogram
settings-download-dir = Downloadmap
settings-download-dir-default = ~/Downloads (standaard)
settings-download-dir-change = Wijzigen
settings-poll-interval = Poll-interval (seconden)
settings-notifications-title = Meldingen
settings-notifications-enabled = Meldingen inschakelen
settings-notify-connection = Verbindingswijzigingen
settings-notify-files = Inkomende bestanden
settings-notify-devices = Nieuwe apparaten

# Health / Error states
health-not-installed-title = Tailscale niet geïnstalleerd
health-not-installed-body = Het tailscale CLI-hulpmiddel werd niet gevonden.
health-not-installed-hint = Installeer Tailscale: https://tailscale.com/download/linux
health-daemon-down-title = Tailscale-daemon draait niet
health-daemon-down-body = De tailscaled-service draait niet.
health-daemon-down-hint = Start met: sudo systemctl start tailscaled
health-no-operator-title = Operator-machtiging vereist
health-no-operator-body = De tailscale-operator is niet ingesteld voor uw gebruiker.
health-no-operator-hint = Voer uit: sudo tailscale set --operator=$USER
health-error-title = Tailscale-fout
health-error-hint = Controleer uw Tailscale-installatie.

# File chooser
file-chooser-title = Kies een bestand of bestanden...
dir-chooser-title = Kies downloadmap
