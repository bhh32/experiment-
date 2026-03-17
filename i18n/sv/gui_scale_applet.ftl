cosmic-applet-button = Cosmic knapp

# Fliketiketter
tab-status = Status
tab-taildrop = Tail Drop
tab-exit-node = Utgångsnod
tab-devices = Enheter
tab-serve = Serve
tab-subnets = Subnät
tab-settings = Inställningar

# Status-flik
status-account = Konto
status-new-login = Ny inloggning
status-ipv4 = IPv4-adress
status-ipv6 = IPv6-adress
status-connection = Status
status-connected = Ansluten
status-disconnected = Frånkopplad
status-enable-ssh = Aktivera SSH
status-accept-routes = Acceptera rutter
status-magic-dns = MagicDNS
status-connect-toggle = Ansluten
status-tailnet-lock = Tailnet Lock
status-lock-enabled-signed = Aktiverad (signerad)
status-lock-enabled-unsigned = Aktiverad (osignerad)
status-lock-disabled = Inaktiverad
status-pending-signatures = Väntande signaturer

# Tail Drop-flik
taildrop-target-device = Målenhet
taildrop-selected-files = Valda filer
taildrop-no-files = Inga filer valda
taildrop-select-files = Välj fil(er)
taildrop-send-files = Skicka fil(er)
taildrop-receive-files = Ta emot fil(er)
taildrop-transfer-status = Överföringsstatus
taildrop-download-directory = Nedladdningsmapp
taildrop-files-sent = Fil(er) skickades framgångsrikt!
taildrop-choose-device-first = Välj en enhet först och välj sedan dina filer.
taildrop-select-device-files = Välj en enhet och fil(er) först.
taildrop-send-tooltip = Skicka de valda filerna.
taildrop-receive-tooltip = Ta emot filer som väntar i din Tail Drop-inkorg.
taildrop-received = Tog emot fil(er) i { $path }

# Utgångsnod-flik
exit-node-selected = Vald nod
exit-node-enable-host = Aktivera värdutgångsnod
exit-node-disable-host = Inaktivera värdutgångsnod
exit-node-allow-lan = Tillåt LAN-åtkomst
exit-node-cant-select = Kan inte välja en utgångsnod medan värden är en utgångsnod!

# Enheter-flik
devices-title = Enheter på Tailnet
devices-select-prompt = Välj en enhet ovan för att se detaljer.
devices-not-found = Enheten hittades inte
devices-this-device = (denna enhet)
devices-name = Namn
devices-dns-name = DNS-namn
devices-ip-address = IP-adress
devices-os = Operativsystem
devices-online = Online
devices-yes = Ja
devices-no = Nej
devices-exit-node = Utgångsnod
devices-tags = Taggar
devices-tags-none = Inga
devices-relay = Relä
devices-traffic = Trafik
devices-last-seen = Senast sedd
devices-last-seen-now = Nu
devices-ping = Ping
devices-pinging = Pingar...
devices-copy-dns = Kopiera DNS-namn
devices-copy-ip = Kopiera IP
devices-copy-clipboard = Kopiera till urklipp

# Serve-flik
serve-title = Tailscale Serve
serve-no-entries = Inga aktiva serve-poster.
serve-add-title = Lägg till ny serve-post
serve-port-placeholder = Port (t.ex. 3000)
serve-path-placeholder = Sökväg (t.ex. /)
serve-add = Lägg till
serve-remove = Ta bort
serve-refresh = Uppdatera
serve-funnel = (Funnel)

# Subnät-flik
subnets-title = Subnätrutter
subnets-no-routes = Inga annonserade subnätrutter.
subnets-add-title = Lägg till subnätrutt
subnets-cidr-placeholder = CIDR (t.ex. 192.168.1.0/24)
subnets-add = Lägg till
subnets-remove = Ta bort

# Inställningar-flik
settings-auto-connect = Anslut automatiskt vid start
settings-dynamic-icon = Dynamisk panelikon
settings-download-dir = Nedladdningsmapp
settings-download-dir-default = ~/Downloads (standard)
settings-download-dir-change = Ändra
settings-poll-interval = Pollintervall (sekunder)
settings-notifications-title = Aviseringar
settings-notifications-enabled = Aktivera aviseringar
settings-notify-connection = Anslutningsändringar
settings-notify-files = Inkommande filer
settings-notify-devices = Nya enheter

# Hälsa / Felstatus
health-not-installed-title = Tailscale inte installerat
health-not-installed-body = CLI-verktyget tailscale hittades inte.
health-not-installed-hint = Installera Tailscale: https://tailscale.com/download/linux
health-daemon-down-title = Tailscale-demonen körs inte
health-daemon-down-body = Tjänsten tailscaled körs inte.
health-daemon-down-hint = Starta med: sudo systemctl start tailscaled
health-no-operator-title = Operatörsbehörighet krävs
health-no-operator-body = Tailscale-operatören är inte inställd för din användare.
health-no-operator-hint = Kör: sudo tailscale set --operator=$USER
health-error-title = Tailscale-fel
health-error-hint = Kontrollera din Tailscale-installation.

# Allmänt
select-default = Välj
none-default = Ingen
copy-tooltip = Kopiera

# Filväljare
file-chooser-title = Välj en fil eller filer...
dir-chooser-title = Välj nedladdningsmapp

# Ytterligare status
status-dns-suffix = DNS-suffix

# Ytterligare Tail Drop
taildrop-inbox = Inkorg
taildrop-no-waiting = Inga filer väntar.
taildrop-waiting-count = { $count } fil(er) väntar i inkorgen
taildrop-choose-device = Välj en enhet först.
