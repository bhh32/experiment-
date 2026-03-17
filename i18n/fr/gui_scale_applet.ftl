cosmic-applet-button = Bouton Cosmic

# Onglets
tab-status = État
tab-taildrop = Tail Drop
tab-exit-node = Nœud de sortie
tab-devices = Appareils
tab-serve = Serve
tab-subnets = Sous-réseaux
tab-settings = Paramètres

# Onglet état
status-account = Compte
status-new-login = Nouvelle connexion
status-ipv4 = Adresse IPv4
status-ipv6 = Adresse IPv6
status-connection = État
status-connected = Connecté
status-disconnected = Déconnecté
status-enable-ssh = Activer SSH
status-accept-routes = Accepter les routes
status-magic-dns = MagicDNS
status-connect-toggle = Connecté
status-tailnet-lock = Tailnet Lock
status-lock-enabled-signed = Activé (signé)
status-lock-enabled-unsigned = Activé (non signé)
status-lock-disabled = Désactivé
status-pending-signatures = Signatures en attente

# Onglet Tail Drop
taildrop-target-device = Appareil cible
taildrop-selected-files = Fichiers sélectionnés
taildrop-no-files = Aucun fichier sélectionné
taildrop-select-files = Sélectionner fichier(s)
taildrop-send-files = Envoyer fichier(s)
taildrop-receive-files = Recevoir fichier(s)
taildrop-transfer-status = État du transfert
taildrop-download-directory = Répertoire de téléchargement
taildrop-files-sent = Fichier(s) envoyé(s) avec succès !
taildrop-choose-device-first = Choisissez d'abord un appareil, puis sélectionnez vos fichiers.
taildrop-select-device-files = Sélectionnez d'abord un appareil et des fichiers.
taildrop-send-tooltip = Envoyer les fichiers sélectionnés.
taildrop-receive-tooltip = Recevoir les fichiers en attente dans votre boîte Tail Drop.
taildrop-received = Fichier(s) reçu(s) dans { $path }

# Onglet nœud de sortie
exit-node-selected = Nœud sélectionné
exit-node-enable-host = Activer le nœud de sortie hôte
exit-node-disable-host = Désactiver le nœud de sortie hôte
exit-node-allow-lan = Autoriser l'accès LAN
exit-node-cant-select = Impossible de sélectionner un nœud de sortie lorsque l'hôte est un nœud de sortie !

# Onglet appareils
devices-title = Appareils sur le Tailnet
devices-select-prompt = Sélectionnez un appareil ci-dessus pour voir les détails.
devices-not-found = Appareil non trouvé
devices-this-device = (cet appareil)
devices-name = Nom
devices-dns-name = Nom DNS
devices-ip-address = Adresse IP
devices-os = Système d'exploitation
devices-online = En ligne
devices-yes = Oui
devices-no = Non
devices-exit-node = Nœud de sortie
devices-tags = Étiquettes
devices-tags-none = Aucune
devices-relay = Relais
devices-traffic = Trafic
devices-last-seen = Dernière connexion
devices-last-seen-now = Maintenant
devices-ping = Ping
devices-pinging = Ping en cours...
devices-copy-dns = Copier le nom DNS
devices-copy-ip = Copier l'IP
devices-copy-clipboard = Copier dans le presse-papiers

# Onglet Serve
serve-title = Tailscale Serve
serve-no-entries = Aucune entrée serve active.
serve-add-title = Ajouter une nouvelle entrée serve
serve-port-placeholder = Port (ex. 3000)
serve-path-placeholder = Chemin (ex. /)
serve-add = Ajouter
serve-remove = Supprimer
serve-refresh = Actualiser
serve-funnel = (Funnel)

# Onglet sous-réseaux
subnets-title = Routes de sous-réseau
subnets-no-routes = Aucune route de sous-réseau annoncée.
subnets-add-title = Ajouter une route de sous-réseau
subnets-cidr-placeholder = CIDR (ex. 192.168.1.0/24)
subnets-add = Ajouter
subnets-remove = Supprimer

# Onglet paramètres
settings-auto-connect = Connexion automatique au démarrage
settings-dynamic-icon = Icône de panneau dynamique
settings-download-dir = Répertoire de téléchargement
settings-download-dir-default = ~/Downloads (par défaut)
settings-download-dir-change = Modifier
settings-poll-interval = Intervalle de sondage (secondes)
settings-notifications-title = Notifications
settings-notifications-enabled = Activer les notifications
settings-notify-connection = Changements de connexion
settings-notify-files = Fichiers entrants
settings-notify-devices = Nouveaux appareils

# États de santé / Erreur
health-not-installed-title = Tailscale non installé
health-not-installed-body = L'outil CLI tailscale n'a pas été trouvé.
health-not-installed-hint = Installer Tailscale : https://tailscale.com/download/linux
health-daemon-down-title = Démon Tailscale non actif
health-daemon-down-body = Le service tailscaled n'est pas actif.
health-daemon-down-hint = Démarrez avec : sudo systemctl start tailscaled
health-no-operator-title = Permission opérateur requise
health-no-operator-body = L'opérateur tailscale n'est pas défini pour votre utilisateur.
health-no-operator-hint = Exécutez : sudo tailscale set --operator=$USER
health-error-title = Erreur Tailscale
health-error-hint = Vérifiez votre installation Tailscale.

# Général
select-default = Sélectionner
none-default = Aucun
copy-tooltip = Copier

# Sélecteur de fichiers
file-chooser-title = Choisir un fichier ou des fichiers...
dir-chooser-title = Choisir le répertoire de téléchargement

# Statut supplémentaire
status-dns-suffix = Suffixe DNS

# Tail Drop supplémentaire
taildrop-inbox = Boîte de réception
taildrop-no-waiting = Aucun fichier en attente.
taildrop-waiting-count = { $count } fichier(s) en attente
taildrop-choose-device = Choisissez d'abord un appareil.
