cosmic-applet-button = Botón Cosmic

# Pestañas
tab-status = Estado
tab-taildrop = Tail Drop
tab-exit-node = Nodo de salida
tab-devices = Dispositivos
tab-serve = Serve
tab-subnets = Subredes
tab-settings = Configuración

# Pestaña de estado
status-account = Cuenta
status-new-login = Nuevo inicio de sesión
status-ipv4 = Dirección IPv4
status-ipv6 = Dirección IPv6
status-connection = Estado
status-connected = Conectado
status-disconnected = Desconectado
status-enable-ssh = Habilitar SSH
status-accept-routes = Aceptar rutas
status-magic-dns = MagicDNS
status-connect-toggle = Conectado
status-tailnet-lock = Tailnet Lock
status-lock-enabled-signed = Habilitado (firmado)
status-lock-enabled-unsigned = Habilitado (sin firmar)
status-lock-disabled = Deshabilitado
status-pending-signatures = Firmas pendientes

# Pestaña Tail Drop
taildrop-target-device = Dispositivo destino
taildrop-selected-files = Archivos seleccionados
taildrop-no-files = No hay archivos seleccionados
taildrop-select-files = Seleccionar archivo(s)
taildrop-send-files = Enviar archivo(s)
taildrop-receive-files = Recibir archivo(s)
taildrop-transfer-status = Estado de transferencia
taildrop-download-directory = Directorio de descarga
taildrop-files-sent = ¡Archivo(s) enviado(s) exitosamente!
taildrop-choose-device-first = Elija un dispositivo primero, luego seleccione sus archivos.
taildrop-select-device-files = Seleccione un dispositivo y archivo(s) primero.
taildrop-send-tooltip = Enviar los archivos seleccionados.
taildrop-receive-tooltip = Recibir archivos esperando en su bandeja de Tail Drop.
taildrop-received = Archivo(s) recibido(s) en { $path }

# Pestaña nodo de salida
exit-node-selected = Nodo seleccionado
exit-node-enable-host = Habilitar nodo de salida del host
exit-node-disable-host = Deshabilitar nodo de salida del host
exit-node-allow-lan = Permitir acceso LAN
exit-node-cant-select = ¡No se puede seleccionar un nodo de salida mientras el host es un nodo de salida!

# Pestaña dispositivos
devices-title = Dispositivos en Tailnet
devices-select-prompt = Seleccione un dispositivo arriba para ver detalles.
devices-not-found = Dispositivo no encontrado
devices-this-device = (este dispositivo)
devices-name = Nombre
devices-dns-name = Nombre DNS
devices-ip-address = Dirección IP
devices-os = Sistema operativo
devices-online = En línea
devices-yes = Sí
devices-no = No
devices-exit-node = Nodo de salida
devices-tags = Etiquetas
devices-tags-none = Ninguna
devices-relay = Relé
devices-traffic = Tráfico
devices-last-seen = Última conexión
devices-last-seen-now = Ahora
devices-ping = Ping
devices-pinging = Haciendo ping...
devices-copy-dns = Copiar nombre DNS
devices-copy-ip = Copiar IP
devices-copy-clipboard = Copiar al portapapeles

# Pestaña Serve
serve-title = Tailscale Serve
serve-no-entries = No hay entradas de serve activas.
serve-add-title = Agregar nueva entrada de serve
serve-port-placeholder = Puerto (ej. 3000)
serve-path-placeholder = Ruta (ej. /)
serve-add = Agregar
serve-remove = Eliminar
serve-refresh = Actualizar
serve-funnel = (Funnel)

# Pestaña subredes
subnets-title = Rutas de subred
subnets-no-routes = No hay rutas de subred anunciadas.
subnets-add-title = Agregar ruta de subred
subnets-cidr-placeholder = CIDR (ej. 192.168.1.0/24)
subnets-add = Agregar
subnets-remove = Eliminar

# Pestaña configuración
settings-auto-connect = Conectar automáticamente al iniciar
settings-dynamic-icon = Ícono de panel dinámico
settings-download-dir = Directorio de descarga
settings-download-dir-default = ~/Downloads (predeterminado)
settings-download-dir-change = Cambiar
settings-poll-interval = Intervalo de sondeo (segundos)
settings-notifications-title = Notificaciones
settings-notifications-enabled = Habilitar notificaciones
settings-notify-connection = Cambios de conexión
settings-notify-files = Archivos entrantes
settings-notify-devices = Nuevos dispositivos

# Estados de salud / Error
health-not-installed-title = Tailscale no instalado
health-not-installed-body = No se encontró la herramienta CLI de tailscale.
health-not-installed-hint = Instalar Tailscale: https://tailscale.com/download/linux
health-daemon-down-title = Demonio de Tailscale no está ejecutándose
health-daemon-down-body = El servicio tailscaled no está ejecutándose.
health-daemon-down-hint = Inicie con: sudo systemctl start tailscaled
health-no-operator-title = Permiso de operador requerido
health-no-operator-body = El operador de tailscale no está configurado para su usuario.
health-no-operator-hint = Ejecute: sudo tailscale set --operator=$USER
health-error-title = Error de Tailscale
health-error-hint = Verifique su instalación de Tailscale.

# General
select-default = Seleccionar
none-default = Ninguno
copy-tooltip = Copiar

# Selector de archivos
file-chooser-title = Elija un archivo o archivos...
dir-chooser-title = Elija directorio de descarga

# Estado adicional
status-dns-suffix = Sufijo DNS

# Tail Drop adicional
taildrop-inbox = Bandeja de entrada
taildrop-no-waiting = No hay archivos en espera.
taildrop-waiting-count = { $count } archivo(s) en la bandeja de entrada
taildrop-choose-device = Elija un dispositivo primero.
