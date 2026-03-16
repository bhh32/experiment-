cosmic-applet-button = Botão Cosmic

# Abas
tab-status = Estado
tab-taildrop = Tail Drop
tab-exit-node = Nó de saída
tab-devices = Dispositivos
tab-serve = Serve
tab-subnets = Sub-redes
tab-settings = Configurações

# Aba de estado
status-account = Conta
status-new-login = Novo login
status-ipv4 = Endereço IPv4
status-ipv6 = Endereço IPv6
status-connection = Estado
status-connected = Conectado
status-disconnected = Desconectado
status-enable-ssh = Habilitar SSH
status-accept-routes = Aceitar rotas
status-magic-dns = MagicDNS
status-connect-toggle = Conectado
status-tailnet-lock = Tailnet Lock
status-lock-enabled-signed = Habilitado (assinado)
status-lock-enabled-unsigned = Habilitado (não assinado)
status-lock-disabled = Desabilitado
status-pending-signatures = Assinaturas pendentes

# Aba Tail Drop
taildrop-target-device = Dispositivo destino
taildrop-selected-files = Arquivos selecionados
taildrop-no-files = Nenhum arquivo selecionado
taildrop-select-files = Selecionar arquivo(s)
taildrop-send-files = Enviar arquivo(s)
taildrop-receive-files = Receber arquivo(s)
taildrop-transfer-status = Estado da transferência
taildrop-download-directory = Diretório de download
taildrop-files-sent = Arquivo(s) enviado(s) com sucesso!
taildrop-choose-device-first = Escolha um dispositivo primeiro, depois selecione seus arquivos.
taildrop-select-device-files = Selecione um dispositivo e arquivo(s) primeiro.
taildrop-send-tooltip = Enviar os arquivos selecionados.
taildrop-receive-tooltip = Receber arquivos esperando na sua caixa de Tail Drop.
taildrop-received = Arquivo(s) recebido(s) em { $path }

# Aba nó de saída
exit-node-selected = Nó selecionado
exit-node-enable-host = Habilitar nó de saída do host
exit-node-disable-host = Desabilitar nó de saída do host
exit-node-allow-lan = Permitir acesso LAN
exit-node-cant-select = Não é possível selecionar um nó de saída enquanto o host é um nó de saída!

# Aba dispositivos
devices-title = Dispositivos no Tailnet
devices-select-prompt = Selecione um dispositivo acima para ver detalhes.
devices-not-found = Dispositivo não encontrado
devices-this-device = (este dispositivo)
devices-name = Nome
devices-dns-name = Nome DNS
devices-ip-address = Endereço IP
devices-os = Sistema operacional
devices-online = Online
devices-yes = Sim
devices-no = Não
devices-exit-node = Nó de saída
devices-tags = Tags
devices-tags-none = Nenhuma
devices-relay = Relay
devices-traffic = Tráfego
devices-last-seen = Última conexão
devices-last-seen-now = Agora
devices-ping = Ping
devices-pinging = Pingando...
devices-copy-dns = Copiar nome DNS
devices-copy-ip = Copiar IP
devices-copy-clipboard = Copiar para área de transferência

# Aba Serve
serve-title = Tailscale Serve
serve-no-entries = Nenhuma entrada serve ativa.
serve-add-title = Adicionar nova entrada serve
serve-port-placeholder = Porta (ex. 3000)
serve-path-placeholder = Caminho (ex. /)
serve-add = Adicionar
serve-remove = Remover
serve-refresh = Atualizar
serve-funnel = (Funnel)

# Aba sub-redes
subnets-title = Rotas de sub-rede
subnets-no-routes = Nenhuma rota de sub-rede anunciada.
subnets-add-title = Adicionar rota de sub-rede
subnets-cidr-placeholder = CIDR (ex. 192.168.1.0/24)
subnets-add = Adicionar
subnets-remove = Remover

# Aba configurações
settings-auto-connect = Conectar automaticamente ao iniciar
settings-dynamic-icon = Ícone de painel dinâmico
settings-download-dir = Diretório de download
settings-download-dir-default = ~/Downloads (padrão)
settings-download-dir-change = Alterar
settings-poll-interval = Intervalo de consulta (segundos)
settings-notifications-title = Notificações
settings-notifications-enabled = Habilitar notificações
settings-notify-connection = Mudanças de conexão
settings-notify-files = Arquivos recebidos
settings-notify-devices = Novos dispositivos

# Estados de saúde / Erro
health-not-installed-title = Tailscale não instalado
health-not-installed-body = A ferramenta CLI tailscale não foi encontrada.
health-not-installed-hint = Instalar Tailscale: https://tailscale.com/download/linux
health-daemon-down-title = Daemon do Tailscale não está rodando
health-daemon-down-body = O serviço tailscaled não está rodando.
health-daemon-down-hint = Inicie com: sudo systemctl start tailscaled
health-no-operator-title = Permissão de operador necessária
health-no-operator-body = O operador do tailscale não está configurado para seu usuário.
health-no-operator-hint = Execute: sudo tailscale set --operator=$USER
health-error-title = Erro do Tailscale
health-error-hint = Verifique sua instalação do Tailscale.

# Seletor de arquivos
file-chooser-title = Escolher arquivo ou arquivos...
dir-chooser-title = Escolher diretório de download
