use crate::config::{load_preferences, update_config, AppPreferences, APP_ID, CONFIG_VERS};
use crate::logic::{
    self, clear_status, copy_to_clipboard, default_download_dir, format_bytes,
    AccountInfo, DeviceInfo, PingResult, TailscaleState, WaitingFile,
};
use crate::notifications;
use crate::tailscale_api::{TailscaleClient, TailscaleError};
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
    Settings,
}

impl Tab {
    fn label(&self) -> &'static str {
        match self {
            Tab::Status => "Status",
            Tab::TailDrop => "Tail Drop",
            Tab::ExitNode => "Exit Node",
            Tab::Devices => "Devices",
            Tab::Settings => "Settings",
        }
    }

    fn all() -> &'static [Tab] {
        &[
            Tab::Status,
            Tab::TailDrop,
            Tab::ExitNode,
            Tab::Devices,
            Tab::Settings,
        ]
    }
}

// ─── Application State ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum AppHealth {
    Healthy,
    SocketNotFound,
    OperatorNotSet,
    Error(String),
}

pub struct Window {
    core: Core,
    config: Config,
    client: TailscaleClient,
    popup: Option<Id>,
    health: AppHealth,

    // ── Tailscale state (refreshed by polling) ──
    state: TailscaleState,

    // ── Tail Drop UI state ──
    selected_device_idx: Option<usize>,
    selected_device_name: String,
    send_files: Vec<String>,
    send_file_status: String,
    files_sent: bool,
    receive_file_status: String,

    // ── Exit Node UI state ──
    exit_node_names: Vec<String>,
    sel_exit_node_idx: Option<usize>,

    // ── Account dropdown ──
    acct_names: Vec<String>,

    // ── Devices tab ──
    selected_device_detail_idx: Option<usize>,
    ping_result: Option<PingResult>,
    ping_in_progress: bool,

    // ── Subnet input ──
    subnet_input: String,

    // ── Preferences ──
    preferences: AppPreferences,

    // ── UI state ──
    active_tab: Tab,
    previous_connect_state: bool,
    previous_device_count: usize,
    notifications_initialized: bool,
    initial_load_done: bool,
}

// ─── Messages ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum Message {
    // Popup
    TogglePopup,
    PopupClosed(Id),

    // Tabs
    SwitchTab(Tab),

    // Polling
    Tick,
    StateLoaded(Result<TailscaleState, String>),

    // Connection
    EnableSSH(bool),
    AcceptRoutes(bool),
    ConnectDisconnect(bool),
    ToggleMagicDns(bool),

    // Accounts
    SwitchAccount(usize),
    LoginNewAccount,

    // Tail Drop
    DeviceSelected(usize),
    ChooseFiles,
    FilesSelected(Vec<Url>),
    SendFiles,
    FilesSent(Option<String>),
    FileChoosingCancelled,
    ReceiveFiles,
    FilesReceived(String),
    ClearTailDropStatus,

    // Exit Node
    ExitNodeSelected(usize),
    AllowExitNodeLanAccess(bool),
    UpdateIsExitNode(bool),

    // Device details
    SelectDeviceDetail(usize),
    PingDevice(String),
    PingCompleted(Result<PingResult, String>),
    CopyToClipboard(String),

    // Subnets
    SubnetInput(String),
    AddSubnet,
    RemoveSubnet(usize),

    // Settings
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

    ActionCompleted(Result<(), String>),
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
        let client = TailscaleClient::new();
        let preferences = load_preferences();

        let health = if client.is_available() {
            AppHealth::Healthy
        } else {
            AppHealth::SocketNotFound
        };

        let window = Window {
            core,
            config: Config::new(APP_ID, CONFIG_VERS).unwrap(),
            client: client.clone(),
            popup: None,
            health,

            state: TailscaleState::default(),

            selected_device_idx: Some(0),
            selected_device_name: "Select".to_string(),
            send_files: Vec::new(),
            send_file_status: String::new(),
            files_sent: false,
            receive_file_status: String::new(),

            exit_node_names: vec!["None".to_string()],
            sel_exit_node_idx: preferences.exit_node_idx,

            acct_names: Vec::new(),

            selected_device_detail_idx: None,
            ping_result: None,
            ping_in_progress: false,

            subnet_input: String::new(),

            preferences,
            active_tab: Tab::Status,
            previous_connect_state: false,
            previous_device_count: 0,
            notifications_initialized: false,
            initial_load_done: false,
        };

        // Kick off the initial async state load
        let init_client = client;
        let task = cosmic::task::future(async move {
            match logic::fetch_state(&init_client).await {
                Ok(state) => Message::StateLoaded(Ok(state)),
                Err(e) => Message::StateLoaded(Err(e.to_string())),
            }
        });

        (window, task)
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

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
            // ─── Popup ───────────────────────────────────────────────
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

            // ─── Tabs ────────────────────────────────────────────────
            Message::SwitchTab(tab) => {
                self.active_tab = tab;
            }

            // ─── Polling ─────────────────────────────────────────────
            Message::Tick => {
                let client = self.client.clone();
                return cosmic::task::future(async move {
                    match logic::fetch_state(&client).await {
                        Ok(state) => Message::StateLoaded(Ok(state)),
                        Err(e) => Message::StateLoaded(Err(e.to_string())),
                    }
                });
            }
            Message::StateLoaded(result) => {
                match result {
                    Ok(new_state) => {
                        // Notifications for state changes
                        if self.notifications_initialized
                            && self.preferences.notifications_enabled
                        {
                            if self.preferences.notify_on_connection_change
                                && new_state.connected != self.previous_connect_state
                            {
                                notifications::notify_connection_change(new_state.connected);
                            }

                            if self.preferences.notify_on_new_device
                                && new_state.devices.len() > self.previous_device_count
                            {
                                for dev in &new_state.devices {
                                    if !self.state.devices.iter().any(|d| d.id == dev.id) {
                                        notifications::notify_new_device(&dev.name);
                                    }
                                }
                            }

                            if self.preferences.notify_on_incoming_files
                                && !new_state.waiting_files.is_empty()
                            {
                                notifications::notify_incoming_files();
                            }
                        }

                        self.previous_connect_state = new_state.connected;
                        self.previous_device_count = new_state.devices.len();
                        self.notifications_initialized = true;

                        // Update derived UI state
                        self.acct_names = new_state
                            .accounts
                            .iter()
                            .map(|a| a.name.clone())
                            .collect();

                        // Exit node dropdown names
                        let mut en_names = vec!["None".to_string()];
                        for dev in &new_state.exit_node_options {
                            en_names.push(dev.name.clone());
                        }
                        self.exit_node_names = en_names;

                        // Auto-connect on first load if configured
                        if !self.initial_load_done
                            && self.preferences.auto_connect
                            && !new_state.connected
                        {
                            let client = self.client.clone();
                            self.initial_load_done = true;
                            self.state = new_state;
                            return cosmic::task::future(async move {
                                let _ = logic::set_connected(&client, true).await;
                                Message::ActionCompleted(Ok(()))
                            });
                        }

                        self.initial_load_done = true;
                        self.state = new_state;
                    }
                    Err(e) => {
                        if e.contains("not found") || e.contains("Socket") {
                            self.health = AppHealth::SocketNotFound;
                        } else if e.contains("operator") || e.contains("403") {
                            self.health = AppHealth::OperatorNotSet;
                        } else {
                            self.health = AppHealth::Error(e);
                        }
                    }
                }
            }

            // ─── Connection Controls ─────────────────────────────────
            Message::EnableSSH(enabled) => {
                let client = self.client.clone();
                return cosmic::task::future(async move {
                    let _ = logic::set_ssh(&client, enabled).await;
                    Message::ActionCompleted(Ok(()))
                });
            }
            Message::AcceptRoutes(accepted) => {
                let client = self.client.clone();
                return cosmic::task::future(async move {
                    let _ = logic::set_routes(&client, accepted).await;
                    Message::ActionCompleted(Ok(()))
                });
            }
            Message::ConnectDisconnect(connected) => {
                let client = self.client.clone();
                let notify = self.preferences.notifications_enabled
                    && self.preferences.notify_on_connection_change;
                return cosmic::task::future(async move {
                    let _ = logic::set_connected(&client, connected).await;
                    if notify {
                        notifications::notify_connection_change(connected);
                    }
                    Message::ActionCompleted(Ok(()))
                });
            }
            Message::ToggleMagicDns(enabled) => {
                let client = self.client.clone();
                return cosmic::task::future(async move {
                    let _ = logic::set_magic_dns(&client, enabled).await;
                    Message::ActionCompleted(Ok(()))
                });
            }

            // ─── Accounts ────────────────────────────────────────────
            Message::SwitchAccount(idx) => {
                if let Some(acct) = self.state.accounts.get(idx) {
                    let client = self.client.clone();
                    let profile_id = acct.id.clone();
                    let acct_name = acct.name.clone();
                    let notify = self.preferences.notifications_enabled;
                    return cosmic::task::future(async move {
                        let _ = logic::switch_account(&client, &profile_id).await;
                        if notify {
                            notifications::notify_account_switched(&acct_name);
                        }
                        Message::ActionCompleted(Ok(()))
                    });
                }
            }
            Message::LoginNewAccount => {
                let client = self.client.clone();
                return cosmic::task::future(async move {
                    let _ = logic::login_new_account(&client).await;
                    Message::ActionCompleted(Ok(()))
                });
            }

            // ─── Tail Drop ───────────────────────────────────────────
            Message::DeviceSelected(idx) => {
                self.selected_device_idx = Some(idx);
                self.selected_device_name = self
                    .state
                    .device_names
                    .get(idx)
                    .cloned()
                    .unwrap_or_else(|| "Select".to_string());
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
                        Ok(responses) => Message::FilesSelected(responses.urls().to_vec()),
                        Err(file_chooser::Error::Cancelled) => Message::FileChoosingCancelled,
                        Err(e) => {
                            eprintln!("File chooser error: {e}");
                            Message::FileChoosingCancelled
                        }
                    }
                });
            }
            Message::FilesSelected(urls) => {
                for url in &urls {
                    if let Ok(path) = url.to_file_path() {
                        if path.exists() {
                            if let Some(s) = path.to_str() {
                                self.send_files.push(s.to_string());
                            }
                        }
                    }
                }
                self.files_sent = false;
                return self.reopen_popup();
            }
            Message::SendFiles => {
                if self.selected_device_name != "Select" && !self.send_files.is_empty() {
                    self.files_sent = true;
                    let client = self.client.clone();
                    let files = self.send_files.clone();
                    let dev_name = self.selected_device_name.clone();
                    let notify = self.preferences.notifications_enabled;

                    // Find the peer ID for the selected device
                    let peer_id = self
                        .state
                        .devices
                        .iter()
                        .find(|d| d.name == dev_name)
                        .map(|d| d.id.clone())
                        .unwrap_or_default();

                    let file_count = files.len();

                    return cosmic::task::future(async move {
                        let result = logic::send_files(&client, &peer_id, &files).await;
                        if notify && result.is_none() {
                            notifications::notify_files_sent(&dev_name, file_count);
                        }
                        Message::FilesSent(result)
                    });
                }
            }
            Message::FilesSent(tx_status) => {
                self.send_file_status = match tx_status {
                    Some(err) => err,
                    None => "File(s) sent successfully!".to_string(),
                };
                if !self.send_file_status.is_empty() {
                    self.send_files.clear();
                    return cosmic::task::future(async move { Message::ClearTailDropStatus });
                }
            }
            Message::FileChoosingCancelled => {
                return self.reopen_popup();
            }
            Message::ReceiveFiles => {
                let client = self.client.clone();
                let download_dir = self
                    .preferences
                    .download_dir
                    .clone()
                    .unwrap_or_else(default_download_dir);
                let notify = self.preferences.notifications_enabled
                    && self.preferences.notify_on_incoming_files;

                return cosmic::task::future(async move {
                    match logic::receive_files(&client, &download_dir).await {
                        Ok(names) => {
                            if notify {
                                notifications::notify_files_received(&download_dir);
                            }
                            Message::FilesReceived(format!(
                                "Received {} file(s) in {}",
                                names.len(),
                                download_dir
                            ))
                        }
                        Err(e) => Message::FilesReceived(e),
                    }
                });
            }
            Message::FilesReceived(status) => {
                self.receive_file_status = status;
                if !self.receive_file_status.is_empty() {
                    return cosmic::task::future(async move { Message::ClearTailDropStatus });
                }
            }
            Message::ClearTailDropStatus => {
                if !self.receive_file_status.is_empty() {
                    return cosmic::task::future(async move {
                        clear_status(STATUS_CLEAR_TIME).await;
                        Message::FilesReceived(String::new())
                    });
                } else if !self.send_file_status.is_empty() || self.files_sent {
                    self.selected_device_idx = Some(0);
                    self.selected_device_name = "Select".to_string();
                    return cosmic::task::future(async move {
                        clear_status(STATUS_CLEAR_TIME).await;
                        Message::FilesSent(Some(String::new()))
                    });
                }
            }

            // ─── Exit Node ───────────────────────────────────────────
            Message::ExitNodeSelected(idx) => {
                if !self.state.is_exit_node {
                    self.sel_exit_node_idx = Some(idx);
                    let client = self.client.clone();

                    let node_ip = if idx == 0 {
                        String::new()
                    } else {
                        self.state
                            .exit_node_options
                            .get(idx - 1)
                            .and_then(|d| d.tailscale_ips.first())
                            .cloned()
                            .unwrap_or_default()
                    };

                    update_config(self.config.clone(), "exit-node", idx);

                    return cosmic::task::future(async move {
                        let _ = logic::set_exit_node(&client, &node_ip).await;
                        Message::ActionCompleted(Ok(()))
                    });
                }
            }
            Message::AllowExitNodeLanAccess(allow) => {
                if self.state.is_exit_node {
                    let client = self.client.clone();
                    update_config(self.config.clone(), "allow-lan", allow);
                    return cosmic::task::future(async move {
                        let _ = logic::set_exit_node_allow_lan(&client, allow).await;
                        Message::ActionCompleted(Ok(()))
                    });
                }
            }
            Message::UpdateIsExitNode(enable) => {
                if self.sel_exit_node_idx == Some(0) || self.sel_exit_node_idx.is_none() {
                    let client = self.client.clone();
                    return cosmic::task::future(async move {
                        let _ = logic::set_advertise_exit_node(&client, enable).await;
                        Message::ActionCompleted(Ok(()))
                    });
                }
            }

            // ─── Device Details ──────────────────────────────────────
            Message::SelectDeviceDetail(idx) => {
                self.selected_device_detail_idx = Some(idx);
                self.ping_result = None;
            }
            Message::PingDevice(ip) => {
                self.ping_in_progress = true;
                let client = self.client.clone();
                return cosmic::task::future(async move {
                    match logic::ping_device(&client, &ip).await {
                        Ok(pr) => Message::PingCompleted(Ok(pr)),
                        Err(e) => Message::PingCompleted(Err(e.to_string())),
                    }
                });
            }
            Message::PingCompleted(result) => {
                self.ping_in_progress = false;
                self.ping_result = result.ok();
            }
            Message::CopyToClipboard(value) => {
                let _ = copy_to_clipboard(&value);
            }

            // ─── Subnets ─────────────────────────────────────────────
            Message::SubnetInput(val) => {
                self.subnet_input = val;
            }
            Message::AddSubnet => {
                if !self.subnet_input.is_empty() {
                    let mut routes = self.state.advertised_routes.clone();
                    routes.push(self.subnet_input.clone());
                    self.subnet_input.clear();
                    let client = self.client.clone();
                    return cosmic::task::future(async move {
                        let _ = logic::set_advertised_routes(&client, routes).await;
                        Message::ActionCompleted(Ok(()))
                    });
                }
            }
            Message::RemoveSubnet(idx) => {
                if idx < self.state.advertised_routes.len() {
                    let mut routes = self.state.advertised_routes.clone();
                    routes.remove(idx);
                    let client = self.client.clone();
                    return cosmic::task::future(async move {
                        let _ = logic::set_advertised_routes(&client, routes).await;
                        Message::ActionCompleted(Ok(()))
                    });
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
                    update_config(
                        self.config.clone(),
                        "poll-interval",
                        self.preferences.poll_interval_secs,
                    );
                }
            }
            Message::SetIconStyle(dynamic) => {
                self.preferences.icon_style = if dynamic {
                    "dynamic".to_string()
                } else {
                    "static".to_string()
                };
                update_config(
                    self.config.clone(),
                    "icon-style",
                    self.preferences.icon_style.clone(),
                );
            }
            Message::ChooseDownloadDir => {
                return cosmic::task::future(async move {
                    let dialog = file_chooser::open::Dialog::new()
                        .title("Choose download directory");
                    match dialog.open_folders().await {
                        Ok(r) => Message::DownloadDirSelected(r.urls().to_vec()),
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

            // ─── Generic completion ──────────────────────────────────
            Message::ActionCompleted(_) => {
                // State will be refreshed on next tick
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button("tailscale-icon")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        if !matches!(self.health, AppHealth::Healthy) {
            return self.view_unhealthy();
        }

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

        let content = match self.active_tab {
            Tab::Status => self.view_status_tab(),
            Tab::TailDrop => self.view_taildrop_tab(),
            Tab::ExitNode => self.view_exit_node_tab(),
            Tab::Devices => self.view_devices_tab(),
            Tab::Settings => self.view_settings_tab(),
        };

        let full_content = column![tab_bar, content].spacing(8).padding(8);

        self.core
            .applet
            .popup_container(scrollable(full_content))
            .into()
    }
}

// ─── View Helpers ────────────────────────────────────────────────────────────

impl Window {
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

    fn view_unhealthy(&self) -> Element<'_, Message> {
        let (title, body, hint) = match &self.health {
            AppHealth::SocketNotFound => (
                "Tailscale Daemon Not Found",
                "Cannot find the tailscaled socket.",
                "Start it with: sudo systemctl start tailscaled",
            ),
            AppHealth::OperatorNotSet => (
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

    fn view_status_tab(&self) -> Element<'_, Message> {
        let st = &self.state;

        let mut sel_acct_idx = None;
        for (idx, name) in self.acct_names.iter().enumerate() {
            if *name == st.current_account {
                sel_acct_idx = Some(idx);
                break;
            }
        }

        let conn_label = if st.connected { "Connected" } else { "Disconnected" };

        let content = list_column()
            .padding(5)
            .spacing(0)
            .add(settings::item(
                "Account",
                row![
                    dropdown(&self.acct_names, sel_acct_idx, Message::SwitchAccount),
                    button::standard("New Login")
                        .on_press(Message::LoginNewAccount)
                        .width(Length::Shrink),
                ]
                .spacing(8),
            ))
            .add(settings::item(
                "IPv4 Address",
                row![
                    text(&st.ip_v4),
                    button::icon(icon::from_name("edit-copy-symbolic"))
                        .on_press(Message::CopyToClipboard(st.ip_v4.clone()))
                        .tooltip("Copy"),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            ))
            .add(settings::item(
                "IPv6 Address",
                row![
                    text(&st.ip_v6),
                    button::icon(icon::from_name("edit-copy-symbolic"))
                        .on_press(Message::CopyToClipboard(st.ip_v6.clone()))
                        .tooltip("Copy"),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            ))
            .add(settings::item(
                "DNS Suffix",
                row![
                    text(&st.dns_suffix),
                    button::icon(icon::from_name("edit-copy-symbolic"))
                        .on_press(Message::CopyToClipboard(st.dns_suffix.clone()))
                        .tooltip("Copy"),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            ))
            .add(settings::item("Status", text(conn_label)))
            .add(settings::item(
                "Enable SSH",
                toggler(st.ssh_enabled).on_toggle(Message::EnableSSH),
            ))
            .add(settings::item(
                "Accept Routes",
                toggler(st.accept_routes).on_toggle(Message::AcceptRoutes),
            ))
            .add(settings::item(
                "MagicDNS",
                toggler(st.magic_dns).on_toggle(Message::ToggleMagicDns),
            ))
            .add(settings::item(
                "Connected",
                toggler(st.connected).on_toggle(Message::ConnectDisconnect),
            ));

        // Subnet routes section
        let mut subnets_section = column![text("Subnet Routes").size(14)].spacing(4).padding(4);

        if st.advertised_routes.is_empty() {
            subnets_section = subnets_section.push(text("No advertised subnet routes.").size(12));
        } else {
            for (idx, route) in st.advertised_routes.iter().enumerate() {
                subnets_section = subnets_section.push(
                    row![
                        text(route).width(Length::Fill),
                        button::destructive("Remove")
                            .on_press(Message::RemoveSubnet(idx))
                            .width(Length::Shrink),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                );
            }
        }

        subnets_section = subnets_section.push(
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
        );

        Element::from(
            column![Element::from(content), Element::from(subnets_section)]
                .spacing(8),
        )
    }

    fn view_taildrop_tab(&self) -> Element<'_, Message> {
        let file_list_text = if self.send_files.is_empty() {
            "No files selected".to_string()
        } else {
            self.send_files
                .iter()
                .map(|p| p.rsplit('/').next().unwrap_or(p).to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };

        let send_btn: Element<'_, Message> = if !self.send_files.is_empty()
            && self.selected_device_name != "Select"
        {
            Element::from(
                button::suggested("Send File(s)")
                    .on_press(Message::SendFiles)
                    .width(150),
            )
        } else {
            Element::from(button::standard("Send File(s)").width(150))
        };

        let status_text = if !self.send_file_status.is_empty() {
            self.send_file_status.clone()
        } else if self.files_sent {
            "File(s) were sent successfully!".to_string()
        } else if self.selected_device_name == "Select" && !self.send_files.is_empty() {
            "Choose a device first.".to_string()
        } else {
            String::new()
        };

        // Waiting files indicator
        let waiting_text = if self.state.waiting_files.is_empty() {
            "No files waiting.".to_string()
        } else {
            format!(
                "{} file(s) waiting in inbox",
                self.state.waiting_files.len()
            )
        };

        let content = list_column()
            .padding(5)
            .spacing(0)
            .add(settings::item(
                "Target Device",
                dropdown(
                    &self.state.device_names,
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
                        .width(150),
                    horizontal_space().width(Length::Fill),
                    send_btn,
                ]
                .spacing(8)
                .padding(8),
            ))
            .add(settings::item("Inbox", text(waiting_text).size(12)))
            .add(Element::from(
                row![
                    button::standard("Receive File(s)")
                        .on_press(Message::ReceiveFiles)
                        .width(150),
                ]
                .padding(8),
            ))
            .add(settings::item(
                "Transfer Status",
                column![text(status_text), text(self.receive_file_status.clone())].spacing(2),
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

    fn view_exit_node_tab(&self) -> Element<'_, Message> {
        let can_toggle_host = self.sel_exit_node_idx == Some(0)
            || self.sel_exit_node_idx.is_none();

        let host_exit_toggler: Element<'_, Message> = if can_toggle_host {
            Element::from(
                toggler(self.state.is_exit_node)
                    .label(if self.state.is_exit_node {
                        "Disable Host Exit Node"
                    } else {
                        "Enable Host Exit Node"
                    })
                    .on_toggle(Message::UpdateIsExitNode),
            )
        } else {
            Element::from(toggler(self.state.is_exit_node).label("Enable Host Exit Node"))
        };

        let lan_toggler: Element<'_, Message> = if self.state.is_exit_node {
            Element::from(
                toggler(self.state.exit_node_allow_lan)
                    .label("Allow LAN Access")
                    .on_toggle(Message::AllowExitNodeLanAccess),
            )
        } else {
            Element::from(toggler(self.state.exit_node_allow_lan).label("Allow LAN Access"))
        };

        let content = list_column()
            .padding(5)
            .spacing(0)
            .add(settings::item(
                "Selected Node",
                dropdown(
                    &self.exit_node_names,
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

    fn view_devices_tab(&self) -> Element<'_, Message> {
        let mut device_list = list_column().padding(5).spacing(0);

        for (idx, dev) in self.state.devices.iter().enumerate() {
            let dot = if dev.online { "●" } else { "○" };
            let self_label = if dev.is_self { " (this device)" } else { "" };
            let ip = dev.tailscale_ips.first().map(|s| s.as_str()).unwrap_or("");

            let label = format!("{dot} {}{self_label} — {} — {ip}", dev.name, dev.os);

            device_list = device_list.add(Element::from(
                button::text(label)
                    .on_press(Message::SelectDeviceDetail(idx))
                    .width(Length::Fill),
            ));
        }

        let detail: Element<'_, Message> =
            if let Some(idx) = self.selected_device_detail_idx {
                if let Some(dev) = self.state.devices.get(idx) {
                    let tags_str = if dev.tags.is_empty() {
                        "None".to_string()
                    } else {
                        dev.tags.join(", ")
                    };

                    let ip = dev
                        .tailscale_ips
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "N/A".to_string());

                    let ping_section: Element<'_, Message> = if self.ping_in_progress {
                        Element::from(text("Pinging..."))
                    } else if let Some(ref pr) = self.ping_result {
                        let via = if pr.is_direct { "direct" } else { "relay" };
                        Element::from(text(format!(
                            "{:.1}ms ({})",
                            pr.latency_seconds * 1000.0,
                            via
                        )))
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
                                        .on_press(Message::CopyToClipboard(dev.dns_name.clone()))
                                        .tooltip("Copy"),
                                ]
                                .spacing(8)
                                .align_y(Alignment::Center),
                            ))
                            .add(settings::item(
                                "IP Address",
                                row![
                                    text(&ip),
                                    button::icon(icon::from_name("edit-copy-symbolic"))
                                        .on_press(Message::CopyToClipboard(ip.clone()))
                                        .tooltip("Copy"),
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
                                        .on_press(Message::PingDevice(ip.clone()))
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
                toggler(self.preferences.auto_connect).on_toggle(Message::SetAutoConnect),
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
