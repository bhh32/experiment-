use crate::config::{load_config, load_preferences, update_config, AppPreferences, APP_ID, CONFIG_VERS};
use crate::logic::{
    self, clear_status, copy_to_clipboard, enable_exit_node, exit_node_allow_lan_access,
    format_bytes, get_acct_list, get_avail_exit_nodes, get_current_acct, get_device_details,
    get_is_exit_node, get_lock_status, get_magic_dns_status, get_serve_status,
    get_tailscale_con_status, get_tailscale_devices, get_tailscale_ip, get_tailscale_ipv6,
    get_tailscale_routes_status, get_tailscale_ssh_status, login_new_account, ping_device,
    set_exit_node, set_magic_dns, set_routes, set_ssh, switch_accounts, tailscale_int_up,
    tailscale_receive, tailscale_send, check_tailscale_available, check_operator_set,
    get_advertised_routes, set_advertised_routes,
    add_serve, remove_serve, toggle_funnel,
    lock_sign_node, DeviceInfo, LockStatus, PingResult, ServeEntry, TailscaleError,
};
use crate::notifications;
use cosmic::app::Core;
use cosmic::cosmic_config::Config;
use cosmic::dialog::file_chooser::{self, FileFilter};
use cosmic::iced::{
    alignment::Horizontal,
    platform_specific::shell::commands::popup::{destroy_popup, get_popup},
    widget::{column, horizontal_space, row, scrollable, text_input},
    window::Id,
    Alignment, Length, Limits, Subscription,
};
use cosmic::iced_runtime::core::window;
use cosmic::iced_widget::Row;
use cosmic::widget::{
    button, dropdown, icon, list_column,
    settings::{self},
    text, toggler,
};
use cosmic::{Action, Element, Task};
use std::fmt::Debug;
use std::path::PathBuf;
use url::Url;

const DEFAULT_EXIT_NODE: &str = "Select Exit Node";
const POPUP_MAX_WIDTH: f32 = 800.0;
const POPUP_MIN_WIDTH: f32 = 640.0;
const POPUP_MAX_HEIGHT: f32 = 1080.0;
const POPUP_MIN_HEIGHT: f32 = 200.0;
const STATUS_CLEAR_TIME: u64 = 5;

// ─── View tabs for the popup ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Status,
    TailDrop,
    ExitNode,
    Devices,
    Serve,
    Subnets,
    Settings,
}

impl Tab {
    fn label(&self) -> &'static str {
        match self {
            Tab::Status => "Status",
            Tab::TailDrop => "Tail Drop",
            Tab::ExitNode => "Exit Node",
            Tab::Devices => "Devices",
            Tab::Serve => "Serve",
            Tab::Subnets => "Subnets",
            Tab::Settings => "Settings",
        }
    }

    fn all() -> &'static [Tab] {
        &[
            Tab::Status,
            Tab::TailDrop,
            Tab::ExitNode,
            Tab::Devices,
            Tab::Serve,
            Tab::Subnets,
            Tab::Settings,
        ]
    }
}

// ─── Application State ──────────────────────────────────────────────────────

/// The applet health state (from startup checks).
#[derive(Debug, Clone)]
pub enum AppHealth {
    /// Everything is good.
    Healthy,
    /// Tailscale is not installed.
    NotInstalled,
    /// Tailscale daemon is not running.
    DaemonDown,
    /// Operator permission is not set.
    NoOperator,
    /// Some other error.
    Error(String),
}

/// Holds the applet's state.
pub struct Window {
    core: Core,
    config: Config,
    popup: Option<Id>,
    health: AppHealth,

    // ── Connection state ──
    ssh: bool,
    routes: bool,
    connect: bool,
    magic_dns: bool,
    ip_v4: String,
    ip_v6: String,

    // ── Tail Drop ──
    device_options: Vec<String>,
    selected_device: String,
    selected_device_idx: Option<usize>,
    send_files: Vec<Option<String>>,
    send_file_status: String,
    files_sent: bool,
    receive_file_status: String,

    // ── Exit Node ──
    avail_exit_nodes: Vec<String>,
    sel_exit_node: String,
    sel_exit_node_idx: Option<usize>,
    allow_lan: bool,
    is_exit_node: bool,

    // ── Accounts ──
    acct_list: Vec<String>,
    cur_acct: String,

    // ── Devices (detailed) ──
    devices: Vec<DeviceInfo>,
    selected_device_detail_idx: Option<usize>,
    ping_result: Option<PingResult>,
    ping_in_progress: bool,

    // ── Serve & Funnel ──
    serve_entries: Vec<ServeEntry>,
    serve_port_input: String,
    serve_path_input: String,

    // ── Subnets ──
    advertised_routes: Vec<String>,
    subnet_input: String,

    // ── Tailnet Lock ──
    lock_status: Option<LockStatus>,

    // ── Preferences ──
    preferences: AppPreferences,

    // ── UI state ──
    active_tab: Tab,
    previous_connect_state: bool,
    previous_device_count: usize,

    // ── Notification tracking ──
    notifications_initialized: bool,
}

// ─── Messages ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum Message {
    // ── Popup management ──
    TogglePopup,
    PopupClosed(Id),

    // ── Tab navigation ──
    SwitchTab(Tab),

    // ── Periodic polling ──
    Tick,
    TickResult(TickData),

    // ── Connection controls ──
    EnableSSH(bool),
    AcceptRoutes(bool),
    ConnectDisconnect(bool),
    ToggleMagicDns(bool),

    // ── Account management ──
    SwitchAccount(usize),
    LoginNewAccount,

    // ── Tail Drop ──
    DeviceSelected(usize),
    ChooseFiles,
    FilesSelected(Vec<Url>),
    SendFiles,
    FilesSent(Option<String>),
    FileChoosingCancelled,
    ReceiveFiles,
    FilesReceived(String),
    ClearTailDropStatus,

    // ── Exit Node ──
    ExitNodeSelected(usize),
    AllowExitNodeLanAccess(bool),
    UpdateIsExitNode(bool),

    // ── Device details ──
    SelectDeviceDetail(usize),
    PingDevice(String),
    PingResult(Result<PingResult, String>),
    CopyToClipboard(String),

    // ── Serve & Funnel ──
    ServePortInput(String),
    ServePathInput(String),
    AddServe,
    RemoveServe(String),
    ToggleFunnel(u16, bool),
    RefreshServe,

    // ── Subnets ──
    SubnetInput(String),
    AddSubnet,
    RemoveSubnet(usize),

    // ── Settings ──
    SetAutoConnect(bool),
    SetNotificationsEnabled(bool),
    SetNotifyConnection(bool),
    SetNotifyFiles(bool),
    SetNotifyDevice(bool),
    SetPollInterval(String),
    SetIconStyle(bool),
    ChooseDownloadDir,
    DownloadDirSelected(Vec<Url>),
    DownloadDirCancelled,
}

/// Data returned by the periodic tick.
#[derive(Clone, Debug)]
pub struct TickData {
    pub connected: bool,
    pub ip_v4: String,
    pub ip_v6: String,
    pub ssh: bool,
    pub routes: bool,
    pub magic_dns: bool,
    pub devices: Vec<DeviceInfo>,
    pub device_names: Vec<String>,
    pub is_exit_node: bool,
}

// ─── Application Implementation ─────────────────────────────────────────────

impl cosmic::Application for Window {
    type Executor = cosmic::executor::multi::Executor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Window, Task<Action<Self::Message>>) {
        // Perform startup health checks
        let health = match check_tailscale_available() {
            Ok(()) => match check_operator_set() {
                Ok(true) => AppHealth::Healthy,
                Ok(false) => AppHealth::NoOperator,
                Err(e) => AppHealth::Error(e.to_string()),
            },
            Err(TailscaleError::NotInstalled) => AppHealth::NotInstalled,
            Err(TailscaleError::DaemonNotRunning) => AppHealth::DaemonDown,
            Err(e) => AppHealth::Error(e.to_string()),
        };

        // Load persistent preferences
        let preferences = load_preferences();

        // Only query Tailscale state if healthy
        let (ssh, routes, connect, magic_dns, ip_v4, ip_v6, device_options, devices,
             allow_lan, is_exit_node, avail_exit_nodes, acct_list, cur_acct,
             serve_entries, advertised_routes, lock_status) =
            if matches!(health, AppHealth::Healthy) {
                let ssh = get_tailscale_ssh_status().unwrap_or(false);
                let routes = get_tailscale_routes_status().unwrap_or(false);
                let connect = get_tailscale_con_status().unwrap_or(false);
                let magic_dns = get_magic_dns_status().unwrap_or(true);
                let ip_v4 = get_tailscale_ip().unwrap_or_else(|_| "N/A".to_string());
                let ip_v6 = get_tailscale_ipv6().unwrap_or_else(|_| "N/A".to_string());
                let device_options = get_tailscale_devices().unwrap_or_else(|_| vec!["Select".to_string()]);
                let devices = get_device_details().unwrap_or_default();
                let is_exit_node = get_is_exit_node().unwrap_or(false);

                let allow_lan = preferences.allow_lan;

                let avail_exit_nodes = if !is_exit_node {
                    get_avail_exit_nodes().unwrap_or_else(|_| vec!["None".to_string()])
                } else {
                    vec!["Can't select an exit node\nwhile host is an exit node!".to_string()]
                };

                let acct_list = get_acct_list().unwrap_or_default();
                let cur_acct = get_current_acct().unwrap_or_else(|_| "Unknown".to_string());
                let serve_entries = get_serve_status().unwrap_or_default();
                let advertised_routes = get_advertised_routes().unwrap_or_default();
                let lock_status = get_lock_status().ok();

                // Auto-connect if configured
                if preferences.auto_connect && !connect {
                    let _ = tailscale_int_up(true);
                }

                (ssh, routes, connect, magic_dns, ip_v4, ip_v6, device_options, devices,
                 allow_lan, is_exit_node, avail_exit_nodes, acct_list, cur_acct,
                 serve_entries, advertised_routes, lock_status)
            } else {
                (false, false, false, true,
                 "N/A".to_string(), "N/A".to_string(),
                 vec!["Select".to_string()], Vec::new(),
                 false, false, vec!["None".to_string()],
                 Vec::new(), "Unknown".to_string(),
                 Vec::new(), Vec::new(), None)
            };

        let device_count = devices.len();

        let mut window = Window {
            core,
            config: Config::new(APP_ID, CONFIG_VERS).unwrap(),
            popup: None,
            health,

            ssh,
            routes,
            connect,
            magic_dns,
            ip_v4,
            ip_v6,

            device_options,
            selected_device: DEFAULT_EXIT_NODE.to_string(),
            selected_device_idx: Some(0),
            send_files: Vec::new(),
            send_file_status: String::new(),
            files_sent: false,
            receive_file_status: String::new(),

            avail_exit_nodes,
            sel_exit_node: DEFAULT_EXIT_NODE.to_string(),
            sel_exit_node_idx: preferences.exit_node_idx,
            allow_lan,
            is_exit_node,

            acct_list,
            cur_acct,

            devices,
            selected_device_detail_idx: None,
            ping_result: None,
            ping_in_progress: false,

            serve_entries,
            serve_port_input: String::new(),
            serve_path_input: String::new(),

            advertised_routes,
            subnet_input: String::new(),

            lock_status,

            preferences,
            active_tab: Tab::Status,
            previous_connect_state: connect,
            previous_device_count: device_count,
            notifications_initialized: false,
        };

        (window, Task::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    // ── Subscription for periodic polling ──
    fn subscription(&self) -> Subscription<Self::Message> {
        if !matches!(self.health, AppHealth::Healthy) {
            return Subscription::none();
        }

        let interval_secs = self.preferences.poll_interval_secs.max(5);
        cosmic::iced::time::every(std::time::Duration::from_secs(interval_secs))
            .map(|_| Message::Tick)
    }

    fn update(&mut self, message: Self::Message) -> Task<Action<Self::Message>> {
        match message {
            // ─── Popup Management ────────────────────────────────────
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    self.receive_file_status = String::new();
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id, None, None, None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(POPUP_MAX_WIDTH)
                        .min_width(POPUP_MIN_WIDTH)
                        .min_height(POPUP_MIN_HEIGHT)
                        .max_height(POPUP_MAX_HEIGHT);
                    get_popup(popup_settings)
                };
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }

            // ─── Tab Navigation ──────────────────────────────────────
            Message::SwitchTab(tab) => {
                self.active_tab = tab;
                // Refresh data when switching to certain tabs
                match tab {
                    Tab::Serve => {
                        self.serve_entries = get_serve_status().unwrap_or_default();
                    }
                    Tab::Subnets => {
                        self.advertised_routes = get_advertised_routes().unwrap_or_default();
                    }
                    Tab::Devices => {
                        self.devices = get_device_details().unwrap_or_default();
                    }
                    _ => {}
                }
            }

            // ─── Periodic Polling ────────────────────────────────────
            Message::Tick => {
                return cosmic::task::future(async {
                    let connected = get_tailscale_con_status().unwrap_or(false);
                    let ip_v4 = get_tailscale_ip().unwrap_or_else(|_| "N/A".into());
                    let ip_v6 = get_tailscale_ipv6().unwrap_or_else(|_| "N/A".into());
                    let ssh = get_tailscale_ssh_status().unwrap_or(false);
                    let routes = get_tailscale_routes_status().unwrap_or(false);
                    let magic_dns = get_magic_dns_status().unwrap_or(true);
                    let devices = get_device_details().unwrap_or_default();
                    let device_names = get_tailscale_devices()
                        .unwrap_or_else(|_| vec!["Select".to_string()]);
                    let is_exit_node = get_is_exit_node().unwrap_or(false);

                    Message::TickResult(TickData {
                        connected,
                        ip_v4,
                        ip_v6,
                        ssh,
                        routes,
                        magic_dns,
                        devices,
                        device_names,
                        is_exit_node,
                    })
                });
            }
            Message::TickResult(data) => {
                // Check for connection change notifications
                if self.notifications_initialized
                    && self.preferences.notifications_enabled
                    && self.preferences.notify_on_connection_change
                    && data.connected != self.previous_connect_state
                {
                    notifications::notify_connection_change(data.connected);
                }

                // Check for new device notifications
                if self.notifications_initialized
                    && self.preferences.notifications_enabled
                    && self.preferences.notify_on_new_device
                    && data.devices.len() > self.previous_device_count
                {
                    let new_devices: Vec<&DeviceInfo> = data
                        .devices
                        .iter()
                        .filter(|d| !self.devices.iter().any(|existing| existing.name == d.name))
                        .collect();
                    for dev in new_devices {
                        notifications::notify_new_device(&dev.name);
                    }
                }

                self.previous_connect_state = data.connected;
                self.previous_device_count = data.devices.len();
                self.notifications_initialized = true;

                self.connect = data.connected;
                self.ip_v4 = data.ip_v4;
                self.ip_v6 = data.ip_v6;
                self.ssh = data.ssh;
                self.routes = data.routes;
                self.magic_dns = data.magic_dns;
                self.devices = data.devices;
                self.device_options = data.device_names;
                self.is_exit_node = data.is_exit_node;
            }

            // ─── Connection Controls ─────────────────────────────────
            Message::EnableSSH(enabled) => {
                self.ssh = enabled;
                let _ = set_ssh(self.ssh);
                update_config(self.config.clone(), "ssh-enabled", self.ssh);
            }
            Message::AcceptRoutes(accepted) => {
                self.routes = accepted;
                let _ = set_routes(self.routes);
                update_config(self.config.clone(), "routes-accepted", self.routes);
            }
            Message::ConnectDisconnect(connection) => {
                self.connect = connection;
                let _ = tailscale_int_up(self.connect);

                if self.preferences.notifications_enabled
                    && self.preferences.notify_on_connection_change
                {
                    notifications::notify_connection_change(self.connect);
                }
            }
            Message::ToggleMagicDns(enabled) => {
                self.magic_dns = enabled;
                let _ = set_magic_dns(self.magic_dns);
            }

            // ─── Account Management ─────────────────────────────────
            Message::SwitchAccount(new_acct) => {
                self.cur_acct = self.acct_list[new_acct].clone();
                let _ = switch_accounts(self.cur_acct.clone());

                // Refresh state for new account
                self.ssh = get_tailscale_ssh_status().unwrap_or(false);
                self.routes = get_tailscale_routes_status().unwrap_or(false);
                self.device_options = get_tailscale_devices()
                    .unwrap_or_else(|_| vec!["Select".to_string()]);
                self.avail_exit_nodes = get_avail_exit_nodes()
                    .unwrap_or_else(|_| vec!["None".to_string()]);
                self.devices = get_device_details().unwrap_or_default();

                if self.preferences.notifications_enabled {
                    notifications::notify_account_switched(&self.cur_acct);
                }
            }
            Message::LoginNewAccount => {
                let _ = login_new_account();
            }

            // ─── Tail Drop ───────────────────────────────────────────
            Message::DeviceSelected(device) => {
                self.selected_device = self.device_options[device].clone();
                self.selected_device_idx = Some(device);
                if self.files_sent {
                    self.files_sent = false;
                }
            }
            Message::ChooseFiles => {
                return cosmic::task::future(async move {
                    let file_filter = FileFilter::new("Any").glob("*.*");
                    let dialog = file_chooser::open::Dialog::new()
                        .title("Choose a file or files...")
                        .filter(file_filter);

                    match dialog.open_files().await {
                        Ok(file_responses) => {
                            Message::FilesSelected(file_responses.urls().to_vec())
                        }
                        Err(file_chooser::Error::Cancelled) => Message::FileChoosingCancelled,
                        Err(e) => {
                            eprintln!("Choosing a file or files went wrong: {e}");
                            Message::FileChoosingCancelled
                        }
                    }
                });
            }
            Message::FilesSelected(urls) => {
                for url in urls.iter() {
                    let path = match url.to_file_path() {
                        Ok(good_path) => good_path,
                        Err(_) => PathBuf::new(),
                    };

                    if path.exists() {
                        if let Some(f_path) = path.as_path().to_str() {
                            self.send_files.push(Some(String::from(f_path)));
                        }
                    }
                }
                self.files_sent = false;
                return self.reopen_popup();
            }
            Message::SendFiles => {
                let files = self.send_files.clone();
                let dev = self.selected_device.clone();

                if dev != "Select" {
                    self.files_sent = true;
                    let file_count = files.len();
                    let notify = self.preferences.notifications_enabled;
                    let dev_clone = dev.clone();
                    return cosmic::task::future(async move {
                        let tx_status = tailscale_send(files, &dev).await;
                        if notify && tx_status.is_none() {
                            notifications::notify_files_sent(&dev_clone, file_count);
                        }
                        Message::FilesSent(tx_status)
                    });
                }
            }
            Message::FilesSent(tx_status) => {
                self.send_file_status = match tx_status {
                    Some(err_val) => err_val,
                    None => String::from("File(s) sent successfully!"),
                };

                if !self.send_file_status.is_empty() {
                    if !self.send_files.is_empty() {
                        self.send_files.clear();
                    }
                    return cosmic::task::future(async move { Message::ClearTailDropStatus });
                }
            }
            Message::FileChoosingCancelled => {
                return self.reopen_popup();
            }
            Message::ReceiveFiles => {
                let download_dir = self.preferences.download_dir.clone();
                let notify = self.preferences.notifications_enabled
                    && self.preferences.notify_on_incoming_files;

                return cosmic::task::future(async move {
                    let rx_status = tailscale_receive(download_dir.clone()).await;
                    if notify && !rx_status.contains("error") && !rx_status.contains("Failed") {
                        notifications::notify_files_received(
                            &download_dir.unwrap_or_else(|| "~/Downloads".to_string()),
                        );
                    }
                    Message::FilesReceived(rx_status)
                });
            }
            Message::FilesReceived(rx_status) => {
                self.receive_file_status = rx_status;
                if !self.receive_file_status.is_empty() {
                    return cosmic::task::future(async move { Message::ClearTailDropStatus });
                }
            }
            Message::ClearTailDropStatus => {
                if !self.receive_file_status.is_empty() {
                    return cosmic::task::future(async move {
                        Message::FilesReceived(
                            match clear_status(STATUS_CLEAR_TIME).await {
                                Some(bad) => format!("Status clear error: {bad}"),
                                None => String::new(),
                            }
                        )
                    });
                } else if !self.send_file_status.is_empty() || self.files_sent {
                    self.selected_device_idx = Some(0);
                    self.selected_device = self.device_options[0].clone();
                    return cosmic::task::future(async move {
                        Message::FilesSent(
                            match clear_status(STATUS_CLEAR_TIME).await {
                                Some(bad) => Some(format!("Status clear error: {bad}")),
                                None => Some(String::new()),
                            }
                        )
                    });
                }
            }

            // ─── Exit Node ───────────────────────────────────────────
            Message::ExitNodeSelected(exit_node) => {
                if !self.is_exit_node {
                    self.sel_exit_node = self.avail_exit_nodes[exit_node].clone();
                    self.sel_exit_node_idx = Some(exit_node);

                    if exit_node == 0 {
                        let _ = set_exit_node(String::new());
                    } else {
                        let _ = set_exit_node(self.sel_exit_node.clone());
                    }

                    update_config(
                        self.config.clone(),
                        "exit-node",
                        self.sel_exit_node_idx.unwrap_or(0),
                    );
                }
            }
            Message::AllowExitNodeLanAccess(allow) => {
                self.allow_lan = allow;
                if self.is_exit_node {
                    let _ = exit_node_allow_lan_access(self.allow_lan);
                    update_config(self.config.clone(), "allow-lan", self.allow_lan);
                }
            }
            Message::UpdateIsExitNode(is_exit) => {
                if self.sel_exit_node_idx == Some(0) || self.sel_exit_node_idx.is_none() {
                    self.is_exit_node = is_exit;
                    let _ = enable_exit_node(self.is_exit_node);
                    self.avail_exit_nodes = get_avail_exit_nodes()
                        .unwrap_or_else(|_| vec!["None".to_string()]);
                }
            }

            // ─── Device Details ──────────────────────────────────────
            Message::SelectDeviceDetail(idx) => {
                self.selected_device_detail_idx = Some(idx);
                self.ping_result = None;
            }
            Message::PingDevice(target) => {
                self.ping_in_progress = true;
                return cosmic::task::future(async move {
                    match ping_device(target).await {
                        Ok(result) => Message::PingResult(Ok(result)),
                        Err(e) => Message::PingResult(Err(e.to_string())),
                    }
                });
            }
            Message::PingResult(result) => {
                self.ping_in_progress = false;
                match result {
                    Ok(pr) => self.ping_result = Some(pr),
                    Err(e) => {
                        eprintln!("Ping failed: {e}");
                        self.ping_result = None;
                    }
                }
            }
            Message::CopyToClipboard(value) => {
                let _ = copy_to_clipboard(&value);
            }

            // ─── Serve & Funnel ──────────────────────────────────────
            Message::ServePortInput(val) => {
                self.serve_port_input = val;
            }
            Message::ServePathInput(val) => {
                self.serve_path_input = val;
            }
            Message::AddServe => {
                if let Ok(port) = self.serve_port_input.parse::<u16>() {
                    let _ = add_serve(port, &self.serve_path_input);
                    self.serve_entries = get_serve_status().unwrap_or_default();
                    self.serve_port_input.clear();
                    self.serve_path_input.clear();
                }
            }
            Message::RemoveServe(path) => {
                let _ = remove_serve(&path);
                self.serve_entries = get_serve_status().unwrap_or_default();
            }
            Message::ToggleFunnel(port, enable) => {
                let _ = toggle_funnel(port, enable);
                self.serve_entries = get_serve_status().unwrap_or_default();
            }
            Message::RefreshServe => {
                self.serve_entries = get_serve_status().unwrap_or_default();
            }

            // ─── Subnets ─────────────────────────────────────────────
            Message::SubnetInput(val) => {
                self.subnet_input = val;
            }
            Message::AddSubnet => {
                if !self.subnet_input.is_empty() {
                    let mut routes = self.advertised_routes.clone();
                    routes.push(self.subnet_input.clone());
                    let _ = set_advertised_routes(&routes);
                    self.advertised_routes = get_advertised_routes().unwrap_or_default();
                    self.subnet_input.clear();
                }
            }
            Message::RemoveSubnet(idx) => {
                if idx < self.advertised_routes.len() {
                    let mut routes = self.advertised_routes.clone();
                    routes.remove(idx);
                    let _ = set_advertised_routes(&routes);
                    self.advertised_routes = get_advertised_routes().unwrap_or_default();
                }
            }

            // ─── Settings ────────────────────────────────────────────
            Message::SetAutoConnect(val) => {
                self.preferences.auto_connect = val;
                update_config(self.config.clone(), "auto-connect", val);
            }
            Message::SetNotificationsEnabled(val) => {
                self.preferences.notifications_enabled = val;
                update_config(self.config.clone(), "notifications-enabled", val);
            }
            Message::SetNotifyConnection(val) => {
                self.preferences.notify_on_connection_change = val;
                update_config(self.config.clone(), "notify-connection", val);
            }
            Message::SetNotifyFiles(val) => {
                self.preferences.notify_on_incoming_files = val;
                update_config(self.config.clone(), "notify-files", val);
            }
            Message::SetNotifyDevice(val) => {
                self.preferences.notify_on_new_device = val;
                update_config(self.config.clone(), "notify-device", val);
            }
            Message::SetPollInterval(val) => {
                if let Ok(secs) = val.parse::<u64>() {
                    self.preferences.poll_interval_secs = secs.max(5);
                    update_config(self.config.clone(), "poll-interval", self.preferences.poll_interval_secs);
                }
            }
            Message::SetIconStyle(dynamic) => {
                self.preferences.icon_style = if dynamic {
                    "dynamic".to_string()
                } else {
                    "static".to_string()
                };
                update_config(self.config.clone(), "icon-style", self.preferences.icon_style.clone());
            }
            Message::ChooseDownloadDir => {
                return cosmic::task::future(async move {
                    let dialog = file_chooser::open::Dialog::new()
                        .title("Choose download directory");

                    match dialog.open_folders().await {
                        Ok(responses) => {
                            Message::DownloadDirSelected(responses.urls().to_vec())
                        }
                        Err(_) => Message::DownloadDirCancelled,
                    }
                });
            }
            Message::DownloadDirSelected(urls) => {
                if let Some(url) = urls.first() {
                    if let Ok(path) = url.to_file_path() {
                        let dir = path.to_string_lossy().to_string();
                        self.preferences.download_dir = Some(dir.clone());
                        update_config(self.config.clone(), "download-dir", dir);
                    }
                }
                return self.reopen_popup();
            }
            Message::DownloadDirCancelled => {
                return self.reopen_popup();
            }
        }
        Task::none()
    }

    // ── Panel icon (dynamic based on connection status) ──
    fn view(&self) -> Element<'_, Self::Message> {
        let icon_name = if self.preferences.icon_style == "dynamic" {
            if self.connect {
                "tailscale-icon" // connected
            } else {
                "tailscale-icon" // TODO: add disconnected icon variant
            }
        } else {
            "tailscale-icon"
        };

        self.core
            .applet
            .icon_button(icon_name)
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        // If the applet is not healthy, show an error message
        if !matches!(self.health, AppHealth::Healthy) {
            return self.view_unhealthy();
        }

        // ── Tab bar ──
        let tab_bar = row(
            Tab::all()
                .iter()
                .map(|tab| {
                    let is_active = *tab == self.active_tab;
                    let btn = if is_active {
                        button::suggested(tab.label())
                    } else {
                        button::standard(tab.label())
                    };
                    let tab_val = *tab;
                    Element::from(
                        btn.on_press(Message::SwitchTab(tab_val))
                            .width(Length::Shrink),
                    )
                })
                .collect::<Vec<_>>(),
        )
        .spacing(4)
        .align_y(Alignment::Center);

        // ── Tab content ──
        let content = match self.active_tab {
            Tab::Status => self.view_status_tab(),
            Tab::TailDrop => self.view_taildrop_tab(),
            Tab::ExitNode => self.view_exit_node_tab(),
            Tab::Devices => self.view_devices_tab(),
            Tab::Serve => self.view_serve_tab(),
            Tab::Subnets => self.view_subnets_tab(),
            Tab::Settings => self.view_settings_tab(),
        };

        let full_content = column![
            tab_bar,
            content,
        ]
        .spacing(8)
        .padding(8);

        self.core
            .applet
            .popup_container(scrollable(full_content))
            .into()
    }
}

// ─── View Helpers ────────────────────────────────────────────────────────────

impl Window {
    /// Reopen the popup after a dialog closes.
    fn reopen_popup(&mut self) -> Task<Action<Message>> {
        let new_id = Id::unique();
        self.popup.replace(new_id);
        let mut popup_settings = self.core.applet.get_popup_settings(
            self.core.main_window_id().unwrap(),
            new_id, None, None, None,
        );
        popup_settings.positioner.size_limits = Limits::NONE
            .max_width(POPUP_MAX_WIDTH)
            .min_width(POPUP_MIN_WIDTH)
            .min_height(POPUP_MIN_HEIGHT)
            .max_height(POPUP_MAX_HEIGHT);
        get_popup(popup_settings)
    }

    /// View shown when Tailscale is not available.
    fn view_unhealthy(&self) -> Element<'_, Message> {
        let (title, body, hint) = match &self.health {
            AppHealth::NotInstalled => (
                "Tailscale Not Installed",
                "The tailscale CLI tool was not found.",
                "Install Tailscale: https://tailscale.com/download/linux",
            ),
            AppHealth::DaemonDown => (
                "Tailscale Daemon Not Running",
                "The tailscaled service is not running.",
                "Start it with: sudo systemctl start tailscaled",
            ),
            AppHealth::NoOperator => (
                "Operator Permission Required",
                "The tailscale operator is not set for your user.",
                "Run: sudo tailscale set --operator=$USER",
            ),
            AppHealth::Error(msg) => (
                "Tailscale Error",
                msg.as_str(),
                "Check your Tailscale installation.",
            ),
            AppHealth::Healthy => unreachable!(),
        };

        let content = column![
            text(title).size(18),
            text(body).size(14),
            text(hint).size(12),
        ]
        .spacing(8)
        .padding(16)
        .align_x(Alignment::Center);

        self.core.applet.popup_container(content).into()
    }

    // ── Status Tab ──
    fn view_status_tab(&self) -> Element<'_, Message> {
        let acct_list = &self.acct_list;
        let mut sel_acct_idx = None;
        for (idx, acct) in acct_list.iter().enumerate() {
            if acct == &self.cur_acct {
                sel_acct_idx = Some(idx);
                break;
            }
        }

        let conn_label = if self.connect {
            "Connected"
        } else {
            "Disconnected"
        };

        // Lock status display
        let lock_info: Element<'_, Message> = if let Some(ref lock) = self.lock_status {
            if lock.enabled {
                Element::from(column![
                    row![settings::item(
                        "Tailnet Lock",
                        text(if lock.node_signed { "Enabled (signed)" } else { "Enabled (unsigned)" })
                    )],
                    row![settings::item(
                        "Pending Signatures",
                        text(format!("{}", lock.pending_signatures))
                    )],
                ].spacing(2))
            } else {
                Element::from(
                    row![settings::item("Tailnet Lock", text("Disabled"))]
                )
            }
        } else {
            Element::from(text(""))
        };

        let content = list_column()
            .padding(5)
            .spacing(0)
            .add(settings::item(
                "Account",
                row![
                    dropdown(acct_list, sel_acct_idx, Message::SwitchAccount),
                    button::standard("New Login")
                        .on_press(Message::LoginNewAccount)
                        .width(Length::Shrink),
                ]
                .spacing(8),
            ))
            .add(settings::item(
                "IPv4 Address",
                row![
                    text(self.ip_v4.clone()),
                    button::icon(icon::from_name("edit-copy-symbolic"))
                        .on_press(Message::CopyToClipboard(self.ip_v4.clone()))
                        .tooltip("Copy to clipboard"),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            ))
            .add(settings::item(
                "IPv6 Address",
                row![
                    text(self.ip_v6.clone()),
                    button::icon(icon::from_name("edit-copy-symbolic"))
                        .on_press(Message::CopyToClipboard(self.ip_v6.clone()))
                        .tooltip("Copy to clipboard"),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            ))
            .add(settings::item(
                "Status",
                text(conn_label),
            ))
            .add(settings::item(
                "Enable SSH",
                toggler(self.ssh).on_toggle(Message::EnableSSH),
            ))
            .add(settings::item(
                "Accept Routes",
                toggler(self.routes).on_toggle(Message::AcceptRoutes),
            ))
            .add(settings::item(
                "MagicDNS",
                toggler(self.magic_dns).on_toggle(Message::ToggleMagicDns),
            ))
            .add(settings::item(
                "Connected",
                toggler(self.connect).on_toggle(Message::ConnectDisconnect),
            ))
            .add(lock_info);

        Element::from(content)
    }

    // ── Tail Drop Tab ──
    fn view_taildrop_tab(&self) -> Element<'_, Message> {
        let file_list_text = if self.send_files.is_empty() {
            "No files selected".to_string()
        } else {
            self.send_files
                .iter()
                .filter_map(|f| f.as_ref())
                .map(|p| {
                    p.rsplit('/')
                        .next()
                        .unwrap_or(p)
                        .to_string()
                })
                .collect::<Vec<_>>()
                .join(", ")
        };

        let send_btn: Element<'_, Message> = if !self.send_files.is_empty()
            && self.selected_device != *"Select"
        {
            Element::from(
                button::suggested("Send File(s)")
                    .on_press(Message::SendFiles)
                    .width(150)
                    .tooltip("Send the selected file(s)."),
            )
        } else {
            Element::from(
                button::standard("Send File(s)")
                    .width(150)
                    .tooltip("Select a device and file(s) first."),
            )
        };

        let status_text = if !self.send_file_status.is_empty() {
            self.send_file_status.clone()
        } else if self.files_sent && self.selected_device != *"Select" {
            "File(s) were sent successfully!".to_string()
        } else if self.selected_device == *"Select" && !self.files_sent {
            "Choose a device first, then select your file(s).".to_string()
        } else {
            String::new()
        };

        let content = list_column()
            .padding(5)
            .spacing(0)
            .add(settings::item(
                "Target Device",
                dropdown(
                    &self.device_options,
                    self.selected_device_idx,
                    Message::DeviceSelected,
                )
                .width(200),
            ))
            .add(settings::item(
                "Selected Files",
                text(file_list_text).size(12),
            ))
            .add(Element::from(
                row![
                    button::standard("Select File(s)")
                        .on_press(Message::ChooseFiles)
                        .width(150)
                        .tooltip("Select file(s) to send."),
                    horizontal_space().width(Length::Fill),
                    send_btn,
                ]
                .spacing(8)
                .padding(8),
            ))
            .add(Element::from(
                row![
                    button::standard("Receive File(s)")
                        .on_press(Message::ReceiveFiles)
                        .width(150)
                        .tooltip("Receive files waiting in your Tail Drop inbox."),
                ]
                .padding(8),
            ))
            .add(settings::item(
                "Transfer Status",
                column![
                    text(status_text),
                    text(self.receive_file_status.clone()),
                ]
                .spacing(2),
            ))
            .add(settings::item(
                "Download Directory",
                text(
                    self.preferences
                        .download_dir
                        .as_deref()
                        .unwrap_or("~/Downloads"),
                ),
            ));

        Element::from(content)
    }

    // ── Exit Node Tab ──
    fn view_exit_node_tab(&self) -> Element<'_, Message> {
        let config_exit_node = self.sel_exit_node_idx;

        let host_exit_toggler: Element<'_, Message> = if config_exit_node == Some(0)
            || config_exit_node.is_none()
        {
            Element::from(
                toggler(self.is_exit_node)
                    .label(if self.is_exit_node {
                        "Disable Host Exit Node"
                    } else {
                        "Enable Host Exit Node"
                    })
                    .on_toggle(Message::UpdateIsExitNode),
            )
        } else {
            Element::from(
                toggler(self.is_exit_node)
                    .label("Enable Host Exit Node"),
            )
        };

        let lan_toggler: Element<'_, Message> = if self.is_exit_node {
            Element::from(
                toggler(self.allow_lan)
                    .label("Allow LAN Access")
                    .on_toggle(Message::AllowExitNodeLanAccess),
            )
        } else {
            Element::from(
                toggler(self.allow_lan).label("Allow LAN Access"),
            )
        };

        let content = list_column()
            .padding(5)
            .spacing(0)
            .add(settings::item(
                "Selected Node",
                dropdown(
                    &self.avail_exit_nodes,
                    self.sel_exit_node_idx,
                    Message::ExitNodeSelected,
                )
                .width(200),
            ))
            .add(Element::from(
                column![host_exit_toggler, lan_toggler]
                    .spacing(8)
                    .padding(8),
            ));

        Element::from(content)
    }

    // ── Devices Tab ──
    fn view_devices_tab(&self) -> Element<'_, Message> {
        let mut device_list = list_column().padding(5).spacing(0);

        for (idx, dev) in self.devices.iter().enumerate() {
            let status_dot = if dev.online { "●" } else { "○" };
            let self_label = if dev.is_self { " (this device)" } else { "" };

            let label = format!(
                "{status_dot} {}{self_label} — {} — {}",
                dev.name, dev.os, dev.tailscale_ip
            );

            device_list = device_list.add(
                Element::from(
                    button::text(label)
                        .on_press(Message::SelectDeviceDetail(idx))
                        .width(Length::Fill),
                ),
            );
        }

        // Device detail panel
        let detail: Element<'_, Message> =
            if let Some(idx) = self.selected_device_detail_idx {
                if let Some(dev) = self.devices.get(idx) {
                    let tags_str = if dev.tags.is_empty() {
                        "None".to_string()
                    } else {
                        dev.tags.join(", ")
                    };

                    let ping_section: Element<'_, Message> = if self.ping_in_progress {
                        Element::from(text("Pinging..."))
                    } else if let Some(ref pr) = self.ping_result {
                        Element::from(
                            text(format!("{:.1}ms via {}", pr.latency_ms, pr.via)),
                        )
                    } else {
                        Element::from(text(""))
                    };

                    Element::from(
                        list_column()
                            .padding(5)
                            .spacing(0)
                            .add(settings::item("Name", text(&dev.name)))
                            .add(settings::item(
                                "DNS Name",
                                row![
                                    text(&dev.dns_name),
                                    button::icon(icon::from_name("edit-copy-symbolic"))
                                        .on_press(Message::CopyToClipboard(
                                            dev.dns_name.clone(),
                                        ))
                                        .tooltip("Copy DNS name"),
                                ]
                                .spacing(8)
                                .align_y(Alignment::Center),
                            ))
                            .add(settings::item(
                                "IP Address",
                                row![
                                    text(&dev.tailscale_ip),
                                    button::icon(icon::from_name("edit-copy-symbolic"))
                                        .on_press(Message::CopyToClipboard(
                                            dev.tailscale_ip.clone(),
                                        ))
                                        .tooltip("Copy IP"),
                                ]
                                .spacing(8)
                                .align_y(Alignment::Center),
                            ))
                            .add(settings::item("OS", text(&dev.os)))
                            .add(settings::item(
                                "Online",
                                text(if dev.online { "Yes" } else { "No" }),
                            ))
                            .add(settings::item(
                                "Exit Node",
                                text(if dev.is_exit_node { "Yes" } else { "No" }),
                            ))
                            .add(settings::item("Tags", text(tags_str)))
                            .add(settings::item("Relay", text(&dev.relay)))
                            .add(settings::item(
                                "Traffic",
                                text(format!(
                                    "↓ {} / ↑ {}",
                                    format_bytes(dev.rx_bytes),
                                    format_bytes(dev.tx_bytes)
                                )),
                            ))
                            .add(settings::item(
                                "Last Seen",
                                text(if dev.last_seen.is_empty() {
                                    "Now"
                                } else {
                                    &dev.last_seen
                                }),
                            ))
                            .add(Element::from(
                                row![
                                    button::standard("Ping")
                                        .on_press(Message::PingDevice(
                                            dev.name.clone(),
                                        ))
                                        .tooltip("Ping this device"),
                                    ping_section,
                                ]
                                .spacing(8)
                                .padding(8)
                                .align_y(Alignment::Center),
                            )),
                    )
                } else {
                    Element::from(text("Device not found"))
                }
            } else {
                Element::from(text("Select a device above to see details."))
            };

        Element::from(
            column![
                text("Devices on Tailnet").size(16),
                Element::from(device_list),
                detail,
            ]
            .spacing(8),
        )
    }

    // ── Serve Tab ──
    fn view_serve_tab(&self) -> Element<'_, Message> {
        let mut entries_list = list_column().padding(5).spacing(0);

        if self.serve_entries.is_empty() {
            entries_list = entries_list.add(
                Element::from(text("No active serve entries.").size(14)),
            );
        } else {
            for entry in &self.serve_entries {
                let funnel_label = if entry.funnel { " (Funnel)" } else { "" };
                let label = format!(
                    "{} → {}{funnel_label}",
                    entry.path, entry.local_addr
                );
                entries_list = entries_list.add(
                    Element::from(
                        row![
                            text(label).width(Length::Fill),
                            button::destructive("Remove")
                                .on_press(Message::RemoveServe(entry.path.clone()))
                                .width(Length::Shrink),
                        ]
                        .spacing(8)
                        .padding(4)
                        .align_y(Alignment::Center),
                    ),
                );
            }
        }

        let content = column![
            text("Tailscale Serve").size(16),
            Element::from(entries_list),
            text("Add New Serve Entry").size(14),
            row![
                text_input("Port (e.g. 3000)", &self.serve_port_input)
                    .on_input(Message::ServePortInput)
                    .width(120),
                text_input("Path (e.g. /)", &self.serve_path_input)
                    .on_input(Message::ServePathInput)
                    .width(120),
                button::suggested("Add")
                    .on_press(Message::AddServe)
                    .width(Length::Shrink),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                button::standard("Refresh")
                    .on_press(Message::RefreshServe)
                    .width(Length::Shrink),
            ]
            .padding(4),
        ]
        .spacing(8)
        .padding(4);

        Element::from(content)
    }

    // ── Subnets Tab ──
    fn view_subnets_tab(&self) -> Element<'_, Message> {
        let mut routes_list = list_column().padding(5).spacing(0);

        if self.advertised_routes.is_empty() {
            routes_list = routes_list.add(
                Element::from(text("No advertised subnet routes.").size(14)),
            );
        } else {
            for (idx, route) in self.advertised_routes.iter().enumerate() {
                routes_list = routes_list.add(
                    Element::from(
                        row![
                            text(route).width(Length::Fill),
                            button::destructive("Remove")
                                .on_press(Message::RemoveSubnet(idx))
                                .width(Length::Shrink),
                        ]
                        .spacing(8)
                        .padding(4)
                        .align_y(Alignment::Center),
                    ),
                );
            }
        }

        let content = column![
            text("Subnet Routes").size(16),
            Element::from(routes_list),
            text("Add Subnet Route").size(14),
            row![
                text_input("CIDR (e.g. 192.168.1.0/24)", &self.subnet_input)
                    .on_input(Message::SubnetInput)
                    .width(250),
                button::suggested("Add")
                    .on_press(Message::AddSubnet)
                    .width(Length::Shrink),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(8)
        .padding(4);

        Element::from(content)
    }

    // ── Settings Tab ──
    fn view_settings_tab(&self) -> Element<'_, Message> {
        let download_dir_display = self
            .preferences
            .download_dir
            .as_deref()
            .unwrap_or("~/Downloads (default)");

        let content = list_column()
            .padding(5)
            .spacing(0)
            .add(settings::item(
                "Auto-connect on startup",
                toggler(self.preferences.auto_connect)
                    .on_toggle(Message::SetAutoConnect),
            ))
            .add(settings::item(
                "Dynamic panel icon",
                toggler(self.preferences.icon_style == "dynamic")
                    .on_toggle(Message::SetIconStyle),
            ))
            .add(settings::item(
                "Download Directory",
                row![
                    text(download_dir_display),
                    button::standard("Change")
                        .on_press(Message::ChooseDownloadDir)
                        .width(Length::Shrink),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            ))
            .add(settings::item(
                "Poll Interval (seconds)",
                text_input("10", &self.preferences.poll_interval_secs.to_string())
                    .on_input(Message::SetPollInterval)
                    .width(80),
            ))
            .add(Element::from(
                column![text("Notifications").size(14)].padding(8),
            ))
            .add(settings::item(
                "Enable Notifications",
                toggler(self.preferences.notifications_enabled)
                    .on_toggle(Message::SetNotificationsEnabled),
            ))
            .add(settings::item(
                "Connection Changes",
                toggler(self.preferences.notify_on_connection_change)
                    .on_toggle(Message::SetNotifyConnection),
            ))
            .add(settings::item(
                "Incoming Files",
                toggler(self.preferences.notify_on_incoming_files)
                    .on_toggle(Message::SetNotifyFiles),
            ))
            .add(settings::item(
                "New Devices",
                toggler(self.preferences.notify_on_new_device)
                    .on_toggle(Message::SetNotifyDevice),
            ));

        Element::from(content)
    }
}
