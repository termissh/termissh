use iced::keyboard::{key::Named, Key, Modifiers};
use iced::widget::{button, column, container, rich_text, row, scrollable, text, text_input, Column};
use iced::{event, keyboard, Alignment, Element, Font, Length, Subscription, Task};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::process::{Child, ChildStdin};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::net::TcpStream;
use std::time::Duration;
use sysinfo::{Disks, System};
use vt100::Parser;

use crate::api;
use crate::config::{self, AppConfig, AppTheme, Host, Language, LayoutPreset};
use crate::ftp;
use crate::i18n::Texts;
use rfd;
use crate::terminal::bridge;
use crate::theme;
use crate::ui::{dialogs, ftp_panel, sidebar, status_bar, tab_bar, toolbar};

const TERMINAL_ROWS: u16 = 40;
const TERMINAL_COLS: u16 = 132;

fn normalize_api_url(input: &str) -> String {
    input.trim().trim_end_matches('/').to_string()
}

// --- Security audit types ---

#[derive(Debug, Clone, PartialEq)]
pub enum SecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl SecuritySeverity {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Critical => "CRITICAL",
            Self::High => "HIGH",
            Self::Medium => "MEDIUM",
            Self::Low => "LOW",
            Self::Info => "INFO",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Critical => "🔴",
            Self::High => "🟠",
            Self::Medium => "🟡",
            Self::Low => "🔵",
            Self::Info => "⚪",
        }
    }

    pub fn sort_key(&self) -> u8 {
        match self {
            Self::Critical => 0,
            Self::High => 1,
            Self::Medium => 2,
            Self::Low => 3,
            Self::Info => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub severity: SecuritySeverity,
    pub category: String,
    pub message: String,
}

pub fn run_security_audit(config: &AppConfig, api_url: &str) -> Vec<SecurityFinding> {
    let mut findings: Vec<SecurityFinding> = Vec::new();

    const COMMON_PASSWORDS: &[&str] = &[
        "password", "123456", "admin", "root", "qwerty",
        "letmein", "welcome", "monkey", "abc123", "1234",
        "pass", "test", "guest", "login", "master",
    ];

    for host in &config.hosts {
        // Root login
        if host.username == "root" {
            findings.push(SecurityFinding {
                severity: SecuritySeverity::High,
                category: "Authentication".into(),
                message: format!(
                    "[{}] Root login detected — use a non-root user with sudo instead",
                    host.alias
                ),
            });
        }

        // Password stored in config
        if let Some(ref pwd) = host.password {
            findings.push(SecurityFinding {
                severity: SecuritySeverity::Medium,
                category: "Credentials".into(),
                message: format!(
                    "[{}] Password saved in config — consider SSH key auth instead",
                    host.alias
                ),
            });

            // Short password
            if pwd.len() < 8 {
                findings.push(SecurityFinding {
                    severity: SecuritySeverity::Critical,
                    category: "Weak Password".into(),
                    message: format!(
                        "[{}] Password is too short ({} chars) — use at least 12 chars",
                        host.alias,
                        pwd.len()
                    ),
                });
            }

            // Common / trivial password
            let pwd_lower = pwd.to_lowercase();
            if COMMON_PASSWORDS.iter().any(|&c| pwd_lower == c) {
                findings.push(SecurityFinding {
                    severity: SecuritySeverity::Critical,
                    category: "Weak Password".into(),
                    message: format!(
                        "[{}] Trivial password detected — change it immediately!",
                        host.alias
                    ),
                });
            }
        }

        // Default SSH port (info — not bad, but worth noting)
        if host.port != 22 {
            findings.push(SecurityFinding {
                severity: SecuritySeverity::Info,
                category: "Port".into(),
                message: format!(
                    "[{}] Non-standard SSH port {} — obscures but does not replace security",
                    host.alias, host.port
                ),
            });
        }
    }

    // HTTP API endpoint
    if api_url.starts_with("http://") && !api_url.is_empty() {
        findings.push(SecurityFinding {
            severity: SecuritySeverity::High,
            category: "API Security".into(),
            message: "API URL uses plain HTTP — switch to HTTPS to protect your API key".into(),
        });
    }

    // API key format
    if let Some(ref key) = config.api_key {
        if !key.starts_with("termi_") || key.len() < 20 {
            findings.push(SecurityFinding {
                severity: SecuritySeverity::Medium,
                category: "API Key".into(),
                message: "API key format looks unusual — expected format: termi_<uuid>".into(),
            });
        }
    }

    // Custom commands with potentially dangerous scripts
    for cmd in &config.custom_commands {
        if cmd.script.contains("rm -rf") || cmd.script.contains("mkfs") || cmd.script.contains(":(){:|:&}") {
            findings.push(SecurityFinding {
                severity: SecuritySeverity::High,
                category: "Custom Command".into(),
                message: format!(
                    "[{}] Custom command '{}' contains potentially destructive operations",
                    cmd.trigger, cmd.description
                ),
            });
        }
        if cmd.script.contains("sudo") {
            findings.push(SecurityFinding {
                severity: SecuritySeverity::Low,
                category: "Custom Command".into(),
                message: format!(
                    "[{}] Custom command '{}' uses sudo — ensure you trust this script",
                    cmd.trigger, cmd.description
                ),
            });
        }
    }

    // Config encryption confirmation (always show as positive)
    findings.push(SecurityFinding {
        severity: SecuritySeverity::Info,
        category: "Storage".into(),
        message: "Config is AES-256-GCM encrypted on disk — credentials are protected at rest".into(),
    });

    if findings.iter().filter(|f| f.severity != SecuritySeverity::Info).count() == 0 {
        findings.push(SecurityFinding {
            severity: SecuritySeverity::Info,
            category: "Overall".into(),
            message: "No critical security issues found — good job!".into(),
        });
    }

    findings.sort_by_key(|f| f.severity.sort_key());
    findings
}

// --- Data structures ---

#[derive(Debug)]
pub struct TerminalTab {
    pub id: u64,
    pub host: Host,
    pub label: String,
    pub connected: bool,
    pub ssh_process: Option<SshProcessInfo>,
    pub relay_error: Option<String>,
    pub output: String,
    pub structure: Vec<String>,
    pub ftp: FtpState,
    // Terminal UX
    pub font_size: f32,
    pub search_active: bool,
    pub search_query: String,
    pub quick_cmds_visible: bool,
    // Input tracking & suggestions
    pub input_buffer: String,
    pub command_history: Vec<String>,
    pub suggestion_index: Option<usize>,
    // System management panel
    pub sys_open: bool,
    pub sys_state: crate::syspanel::SysState,
}

#[derive(Debug, Clone)]
pub struct SshProcessInfo {
    pub relay_path: String,
}

struct TerminalRuntime {
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    rx: mpsc::Receiver<Vec<u8>>,
    parser: Parser,
}

#[derive(Debug, Clone, Default)]
pub struct LocalSystemInfo {
    pub cpu_usage: f32,
    pub cpu_count: usize,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub memory_usage: f32,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
    pub disk_usage_percent: f32,
    pub os_name: String,
    pub hostname: String,
    pub uptime_secs: u64,
}

// --- FTP state ---

#[derive(Debug, Clone, PartialEq)]
pub enum FtpStatus {
    Idle,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum FtpLayout {
    #[default]
    Bottom,
    Right,
}

#[derive(Debug, Clone)]
pub struct FtpState {
    pub visible: bool,
    pub connected_host: Option<Host>,
    pub current_path: String,
    pub entries: Vec<ftp::FtpEntry>,
    pub loading: bool,
    pub status: FtpStatus,
    pub notification: Option<(String, bool)>, // (msg, is_error)
    /// (path, click_time) — çift tık tespiti için
    pub last_click: Option<(String, std::time::Instant)>,
    pub layout: FtpLayout,
    pub search_query: String,
    pub search_results: Option<Vec<ftp::FtpEntry>>,
    pub searching: bool,
}

impl Default for FtpState {
    fn default() -> Self {
        Self {
            visible: false,
            connected_host: None,
            current_path: "/".to_string(),
            entries: Vec::new(),
            loading: false,
            status: FtpStatus::Idle,
            notification: None,
            last_click: None,
            layout: FtpLayout::Bottom,
            search_query: String::new(),
            search_results: None,
            searching: false,
        }
    }
}

// --- Messages ---

#[derive(Debug, Clone)]
pub enum Message {
    // Connection
    SelectHost(usize),
    ConnectToHost(usize),
    CloseTab(usize),
    SwitchTab(usize),

    // CRUD dialogs
    OpenNewDialog,
    OpenEditDialog(usize),
    OpenDeleteConfirm(usize),
    ConfirmDelete(usize),
    CloseDialog,
    SaveDialog,
    DialogFieldChanged(String, String),

    // Settings
    OpenSettings,
    SaveSettings,

    // Search
    SearchInput(String),

    // Ping
    PingAll,
    PingResult(usize, Option<u128>),

    // API sync
    SyncFromApi,
    SyncComplete(Result<Vec<Host>, String>),

    // System info
    SystemInfoTick,

    // Theme / Language
    ToggleTheme,
    ToggleLanguage,
    SettingsThemeChanged(AppTheme),
    SettingsLanguageChanged(Language),

    // FTP / structure
    RefreshStructure,

    // SFTP browser
    FtpToggle,
    FtpToggleLayout,
    FtpNavigate(String),
    FtpRefresh,
    FtpListResult(Result<Vec<ftp::FtpEntry>, String>),
    FtpEntryClick(String),
    FtpDownloadFile(String),
    FtpDownloadResult(Result<String, String>),
    FtpPickUploadFile,
    FtpUploadChosen(Option<std::path::PathBuf>),
    FtpUploadResult(Result<(), String>),
    FtpSearchQueryChanged(String),
    FtpSearchSubmit,
    FtpSearchResult(Result<Vec<ftp::FtpEntry>, String>),
    FtpClearSearch,

    // Embedded terminal bridge
    TerminalKeyPressed(Key, Modifiers),
    TerminalSendBytes(Vec<u8>),
    TerminalClear,
    TerminalSendCtrlC,
    TerminalPoll,
    TerminalFontSizeInc,
    TerminalFontSizeDec,
    TerminalFontSizeReset,
    TerminalSearchToggle,
    TerminalSearchChanged(String),
    TerminalSearchClose,
    TerminalQuickCmdsToggle,
    TerminalQuickCmd(String),

    // Layout preset
    SettingsLayoutChanged(LayoutPreset),

    // Settings — terminal appearance
    SettingsFontSizeChanged(f32),
    SettingsShowBordersChanged(bool),
    SettingsSuggestionsChanged(bool),

    // Command suggestions
    TerminalSuggestionAccept(String),
    TerminalSuggestionMove(i32),
    TerminalCopyOutput,

    // Scroll mode (keyboard navigation through terminal output)
    TerminalScrollModeToggle,
    TerminalScrollBy(f32), // delta: negative = up, positive = down

    // Security audit
    OpenSecurityAudit,

    // Custom commands (aliases)
    OpenCustomCommands,
    AddCustomCommand,
    DeleteCustomCommand(usize),
    SaveCustomCommands,

    // Reserved for future richer terminal integration
    TerminalEvent(u64, String),

    // ── System management panel ──────────────────────────────────────────────
    SysPanelOpen(u64),
    SysPanelClose(u64),
    SysPanelTabSwitch(u64, String),
    SysPanelInput(u64, String, String),
    SysPanelFetch(u64, String),
    SysPanelAction(u64, String),
    SysPanelFetched(u64, String, String),
}

// --- Main App ---

pub struct App {
    pub config: AppConfig,
    pub api_url: String,

    // UI state
    pub selected_host: Option<usize>,
    pub search_query: String,

    // Terminal tabs
    pub terminal_tabs: Vec<TerminalTab>,
    pub active_tab: Option<usize>,
    tab_counter: u64,
    terminal_runtime: HashMap<u64, TerminalRuntime>,
    terminal_scroll_id: scrollable::Id,

    // Scroll mode (keyboard navigation through terminal output)
    pub scroll_mode: bool,
    pub scroll_position: f32, // 0.0 = top, 1.0 = bottom

    // Dialogs
    pub dialog: Option<dialogs::DialogState>,

    // System monitoring
    pub system_info: LocalSystemInfo,
    sys: System,
    disks: Disks,

    // Ping
    pub ping_results: HashMap<usize, Option<u128>>,

    // Theme
    pub theme: AppTheme,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        dotenv::dotenv().ok();
        let config = config::load_config();
        let theme = config.theme;
        let api_url = config
            .api_url
            .clone()
            .map(|u| normalize_api_url(&u))
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| {
                normalize_api_url(
                    &std::env::var("API_URL")
                        .unwrap_or_else(|_| "https://termissh.org".to_string()),
                )
            });

        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();

        let system_info = collect_system_info(&sys, &disks);

        (
            Self {
                config,
                api_url,
                selected_host: None,
                search_query: String::new(),
                terminal_tabs: Vec::new(),
                active_tab: None,
                tab_counter: 0,
                terminal_runtime: HashMap::new(),
                terminal_scroll_id: scrollable::Id::new("terminal-output"),
                scroll_mode: false,
                scroll_position: 1.0,
                dialog: None,
                system_info,
                sys,
                disks,
                ping_results: HashMap::new(),
                theme,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "Termissh".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectHost(idx) => {
                self.selected_host = Some(idx);
            }
            Message::ConnectToHost(idx) => {
                if idx < self.config.hosts.len() {
                    let host = self.config.hosts[idx].clone();
                    self.selected_host = Some(idx);

                    // Resolve relay launcher path (single-binary internal relay mode)
                    match bridge::find_relay_binary() {
                        Ok(relay_path) => {
                            self.tab_counter += 1;
                            let tab_id = self.tab_counter;

                            let tab = match bridge::spawn_relay_child(&relay_path, &host) {
                                Ok(mut child) => {
                                    let stdin = child.stdin.take();
                                    let stdout = child.stdout.take();
                                    let stderr = child.stderr.take();

                                    match (stdin, stdout, stderr) {
                                        (Some(stdin), Some(stdout), Some(stderr)) => {
                                            let (tx, rx) = mpsc::channel::<Vec<u8>>();
                                            spawn_reader_thread(stdout, tx.clone());
                                            spawn_reader_thread(stderr, tx);

                                            self.terminal_runtime.insert(
                                                tab_id,
                                                TerminalRuntime {
                                                    child,
                                                    stdin: Arc::new(Mutex::new(stdin)),
                                                    rx,
                                                    parser: Parser::new(
                                                        TERMINAL_ROWS,
                                                        TERMINAL_COLS,
                                                        10_000,
                                                    ),
                                                },
                                            );

                                            TerminalTab {
                                                id: tab_id,
                                                label: host.alias.clone(),
                                                host: host.clone(),
                                                connected: true,
                                                ssh_process: Some(SshProcessInfo {
                                                    relay_path: relay_path.clone(),
                                                }),
                                                relay_error: None,
                                                output: format!(
                                                    "Connected to {}@{}:{}\n",
                                                    host.username, host.hostname, host.port
                                                ),
                                                structure: fetch_remote_structure(&host),
                                                ftp: FtpState::default(),
                                                font_size: 13.0,
                                                search_active: false,
                                                search_query: String::new(),
                                                quick_cmds_visible: false,
                                                input_buffer: String::new(),
                                                command_history: Vec::new(),
                                                suggestion_index: None,
                                                sys_open: false,
                                                sys_state: crate::syspanel::SysState::new(),
                                            }
                                        }
                                        _ => TerminalTab {
                                            id: tab_id,
                                            label: host.alias.clone(),
                                            host: host.clone(),
                                            connected: false,
                                            ssh_process: Some(SshProcessInfo {
                                                relay_path: relay_path.clone(),
                                            }),
                                            relay_error: Some(
                                                "Relay started but stdio pipes are unavailable."
                                                    .to_string(),
                                            ),
                                            output: String::new(),
                                            structure: Vec::new(),
                                            ftp: FtpState::default(),
                                            font_size: 13.0,
                                            search_active: false,
                                            search_query: String::new(),
                                            quick_cmds_visible: false,
                                            input_buffer: String::new(),
                                            command_history: Vec::new(),
                                            suggestion_index: None,
                                            sys_open: false,
                                            sys_state: crate::syspanel::SysState::new(),
                                        },
                                    }
                                }
                                Err(err) => TerminalTab {
                                    id: tab_id,
                                    label: host.alias.clone(),
                                    host: host.clone(),
                                    connected: false,
                                    ssh_process: Some(SshProcessInfo {
                                        relay_path: relay_path.clone(),
                                    }),
                                    relay_error: Some(err.to_string()),
                                    output: String::new(),
                                    structure: Vec::new(),
                                    ftp: FtpState::default(),
                                    font_size: 13.0,
                                    search_active: false,
                                    search_query: String::new(),
                                    quick_cmds_visible: false,
                                    input_buffer: String::new(),
                                    command_history: Vec::new(),
                                    suggestion_index: None,
                                    sys_open: false,
                                    sys_state: crate::syspanel::SysState::new(),
                                },
                            };

                            self.terminal_tabs.push(tab);
                            self.active_tab = Some(self.terminal_tabs.len() - 1);
                        }
                        Err(err) => {
                            // Relay not found - show connection info instead
                            self.tab_counter += 1;
                            let tab = TerminalTab {
                                id: self.tab_counter,
                                label: host.alias.clone(),
                                host: host.clone(),
                                connected: false,
                                ssh_process: None,
                                relay_error: Some(err.to_string()),
                                output: String::new(),
                                structure: Vec::new(),
                                ftp: FtpState::default(),
                                font_size: 13.0,
                                search_active: false,
                                search_query: String::new(),
                                quick_cmds_visible: false,
                                input_buffer: String::new(),
                                command_history: Vec::new(),
                                suggestion_index: None,
                                sys_open: false,
                                sys_state: crate::syspanel::SysState::new(),
                            };
                            self.terminal_tabs.push(tab);
                            self.active_tab = Some(self.terminal_tabs.len() - 1);
                        }
                    }
                }
            }
            Message::CloseTab(idx) => {
                if idx < self.terminal_tabs.len() {
                    let tab_id = self.terminal_tabs[idx].id;
                    if let Some(mut runtime) = self.terminal_runtime.remove(&tab_id) {
                        let _ = runtime.child.kill();
                        let _ = runtime.child.wait();
                    }
                    self.terminal_tabs.remove(idx);
                    if self.terminal_tabs.is_empty() {
                        self.active_tab = None;
                    } else if let Some(active) = self.active_tab {
                        if active >= self.terminal_tabs.len() {
                            self.active_tab = Some(self.terminal_tabs.len() - 1);
                        } else if active > idx {
                            self.active_tab = Some(active - 1);
                        }
                    }
                }
            }
            Message::SwitchTab(idx) => {
                if idx < self.terminal_tabs.len() {
                    self.active_tab = Some(idx);
                }
            }
            Message::OpenNewDialog => {
                self.dialog = Some(dialogs::DialogState::NewConnection(
                    dialogs::ConnectionForm::default(),
                ));
            }
            Message::OpenEditDialog(idx) => {
                if idx < self.config.hosts.len() {
                    let host = &self.config.hosts[idx];
                    self.dialog = Some(dialogs::DialogState::EditConnection(
                        idx,
                        dialogs::ConnectionForm {
                            alias: host.alias.clone(),
                            hostname: host.hostname.clone(),
                            port: host.port.to_string(),
                            username: host.username.clone(),
                            password: host.password.clone().unwrap_or_default(),
                        },
                    ));
                }
            }
            Message::OpenDeleteConfirm(idx) => {
                self.dialog = Some(dialogs::DialogState::ConfirmDelete(idx));
            }
            Message::ConfirmDelete(idx) => {
                if idx < self.config.hosts.len() {
                    let host = &self.config.hosts[idx];
                    if let (Some(key), Some(id)) = (&self.config.api_key, &host.id) {
                        let _ = api::delete_on_api(&self.api_url, key, id);
                    }
                    self.config.hosts.remove(idx);
                    let _ = config::save_config(&self.config);
                    if self.selected_host == Some(idx) {
                        self.selected_host = None;
                    }
                }
                self.dialog = None;
            }
            Message::CloseDialog => {
                self.dialog = None;
            }
            Message::SaveDialog => {
                if let Some(ref dialog_state) = self.dialog.clone() {
                    match dialog_state {
                        dialogs::DialogState::NewConnection(form) => {
                            let port = form.port.parse::<u16>().unwrap_or(22);
                            let password = if form.password.is_empty() {
                                None
                            } else {
                                Some(form.password.clone())
                            };
                            let mut new_host = Host {
                                id: None,
                                alias: form.alias.clone(),
                                hostname: form.hostname.clone(),
                                port,
                                username: form.username.clone(),
                                password,
                            };
                            if let Some(key) = &self.config.api_key {
                                if let Ok(id) = api::create_on_api(&self.api_url, key, &new_host) {
                                    new_host.id = Some(id);
                                }
                            }
                            self.config.hosts.push(new_host);
                            let _ = config::save_config(&self.config);
                        }
                        dialogs::DialogState::EditConnection(idx, form) => {
                            let idx = *idx;
                            if idx < self.config.hosts.len() {
                                let port = form.port.parse::<u16>().unwrap_or(22);
                                let password = if form.password.is_empty() {
                                    None
                                } else {
                                    Some(form.password.clone())
                                };
                                let updated = Host {
                                    id: self.config.hosts[idx].id.clone(),
                                    alias: form.alias.clone(),
                                    hostname: form.hostname.clone(),
                                    port,
                                    username: form.username.clone(),
                                    password,
                                };
                                if let Some(key) = &self.config.api_key {
                                    let _ = api::update_on_api(&self.api_url, key, &updated);
                                }
                                self.config.hosts[idx] = updated;
                                let _ = config::save_config(&self.config);
                            }
                        }
                        _ => {}
                    }
                }
                self.dialog = None;
            }
            Message::DialogFieldChanged(field, value) => {
                if let Some(ref mut state) = self.dialog {
                    match state {
                        dialogs::DialogState::NewConnection(ref mut form)
                        | dialogs::DialogState::EditConnection(_, ref mut form) => match field.as_str()
                        {
                            "alias" => form.alias = value,
                            "hostname" => form.hostname = value,
                            "port" => form.port = value,
                            "username" => form.username = value,
                            "password" => form.password = value,
                            _ => {}
                        },
                        dialogs::DialogState::Settings(ref mut form) => match field.as_str() {
                            "api_key" => form.api_key = value,
                            "api_url" => form.api_url = value,
                            _ => {}
                        },
                        dialogs::DialogState::CustomCommands(ref mut form) => match field.as_str() {
                            "trigger" => form.new_trigger = value,
                            "script" => form.new_script = value,
                            "description" => form.new_description = value,
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            Message::OpenSettings => {
                self.dialog = Some(dialogs::DialogState::Settings(dialogs::SettingsForm {
                    api_key: self.config.api_key.clone().unwrap_or_default(),
                    api_url: self.api_url.clone(),
                    theme: self.theme,
                    language: self.config.language,
                    layout: self.config.layout,
                    terminal_font_size: self.config.terminal_font_size,
                    show_borders: self.config.show_borders,
                    suggestions_enabled: self.config.suggestions_enabled,
                }));
            }
            Message::SaveSettings => {
                if let Some(dialogs::DialogState::Settings(ref form)) = self.dialog {
                    let previous_api_key = self.config.api_key.clone();
                    let previous_api_url = self.api_url.clone();
                    let trimmed_api_key = form.api_key.trim().to_string();
                    self.config.api_key = if trimmed_api_key.is_empty() {
                        None
                    } else {
                        Some(trimmed_api_key)
                    };
                    let next_api_url = normalize_api_url(&form.api_url);
                    if !next_api_url.is_empty() {
                        self.api_url = next_api_url;
                    }
                    self.config.api_url = Some(self.api_url.clone());

                    let api_target_changed =
                        previous_api_key != self.config.api_key || previous_api_url != self.api_url;
                    if api_target_changed {
                        // Do not keep stale remote entries when endpoint or key changes.
                        self.config.hosts.retain(|h| h.id.is_none());
                    }

                    self.theme = form.theme;
                    self.config.theme = form.theme;
                    self.config.language = form.language;
                    self.config.layout = form.layout;
                    self.config.terminal_font_size = form.terminal_font_size;
                    self.config.show_borders = form.show_borders;
                    self.config.suggestions_enabled = form.suggestions_enabled;
                    let _ = config::save_config(&self.config);

                    // Sync from API if key is set
                    if let Some(ref key) = self.config.api_key {
                        if let Ok(hosts) = api::fetch_from_api(&self.api_url, key) {
                            self.config.hosts = hosts;
                            let _ = config::save_config(&self.config);
                        }
                    }
                }
                self.dialog = None;
            }
            Message::SettingsThemeChanged(t) => {
                if let Some(dialogs::DialogState::Settings(ref mut form)) = self.dialog {
                    form.theme = t;
                }
            }
            Message::SettingsLanguageChanged(language) => {
                if let Some(dialogs::DialogState::Settings(ref mut form)) = self.dialog {
                    form.language = language;
                }
            }
            Message::SearchInput(query) => {
                self.search_query = query;
            }
            Message::PingAll => {
                // TCP ping each host (blocking for now, TODO: async)
                for (idx, host) in self.config.hosts.iter().enumerate() {
                    let addr = format!("{}:{}", host.hostname, host.port);
                    let start = std::time::Instant::now();
                    let result =
                        std::net::TcpStream::connect_timeout(
                            &addr.parse().unwrap_or_else(|_| {
                                std::net::SocketAddr::from(([0, 0, 0, 0], 0))
                            }),
                            Duration::from_secs(3),
                        );
                    match result {
                        Ok(_) => {
                            self.ping_results
                                .insert(idx, Some(start.elapsed().as_millis()));
                        }
                        Err(_) => {
                            self.ping_results.insert(idx, None);
                        }
                    }
                }
            }
            Message::PingResult(idx, ms) => {
                self.ping_results.insert(idx, ms);
            }
            Message::SyncFromApi => {
                if let Some(ref key) = self.config.api_key {
                    if let Ok(hosts) = api::fetch_from_api(&self.api_url, key) {
                        self.config.hosts = hosts;
                        let _ = config::save_config(&self.config);
                    }
                }
            }
            Message::SyncComplete(result) => {
                if let Ok(hosts) = result {
                    self.config.hosts = hosts;
                    let _ = config::save_config(&self.config);
                }
            }
            Message::SystemInfoTick => {
                self.sys.refresh_all();
                self.disks = Disks::new_with_refreshed_list();
                self.system_info = collect_system_info(&self.sys, &self.disks);
            }
            Message::ToggleTheme => {
                let all = AppTheme::all();
                let cur = all.iter().position(|&t| t == self.theme).unwrap_or(0);
                self.theme = all[(cur + 1) % all.len()];
                self.config.theme = self.theme;
                let _ = config::save_config(&self.config);
            }
            Message::ToggleLanguage => {
                self.config.language = match self.config.language {
                    Language::Turkish => Language::English,
                    Language::English => Language::Turkish,
                };
                let _ = config::save_config(&self.config);
            }
            Message::RefreshStructure => {
                if let Some(active) = self.active_tab {
                    if let Some(tab) = self.terminal_tabs.get_mut(active) {
                        tab.structure = fetch_remote_structure(&tab.host);
                    }
                }
            }

            // ── SFTP browser ──────────────────────────────────────────
            Message::FtpToggleLayout => {
                let Some(active) = self.active_tab else { return Task::none(); };
                self.terminal_tabs[active].ftp.layout = match self.terminal_tabs[active].ftp.layout {
                    FtpLayout::Bottom => FtpLayout::Right,
                    FtpLayout::Right => FtpLayout::Bottom,
                };
            }
            Message::FtpSearchQueryChanged(q) => {
                let Some(active) = self.active_tab else { return Task::none(); };
                self.terminal_tabs[active].ftp.search_query = q;
            }
            Message::FtpSearchSubmit => {
                let Some(active) = self.active_tab else { return Task::none(); };
                let query = self.terminal_tabs[active].ftp.search_query.clone();
                if query.trim().is_empty() {
                    return Task::none();
                }
                if let Some(host) = self.terminal_tabs[active].ftp.connected_host.clone() {
                    let start_path = self.terminal_tabs[active].ftp.current_path.clone();
                    self.terminal_tabs[active].ftp.searching = true;
                    self.terminal_tabs[active].ftp.search_results = None;
                    self.terminal_tabs[active].ftp.notification = None;
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                ftp::search_files(&host, &start_path, &query)
                            })
                            .await
                            .unwrap_or_else(|e| Err(e.to_string()))
                        },
                        Message::FtpSearchResult,
                    );
                }
            }
            Message::FtpSearchResult(result) => {
                let Some(active) = self.active_tab else { return Task::none(); };
                self.terminal_tabs[active].ftp.searching = false;
                match result {
                    Ok(entries) => {
                        self.terminal_tabs[active].ftp.search_results = Some(entries);
                    }
                    Err(e) => {
                        self.terminal_tabs[active].ftp.notification =
                            Some((format!("Search failed: {}", e), true));
                    }
                }
            }
            Message::FtpClearSearch => {
                let Some(active) = self.active_tab else { return Task::none(); };
                self.terminal_tabs[active].ftp.search_results = None;
                self.terminal_tabs[active].ftp.search_query = String::new();
                self.terminal_tabs[active].ftp.searching = false;
            }
            Message::FtpToggle => {
                let Some(active) = self.active_tab else { return Task::none(); };
                self.terminal_tabs[active].ftp.visible = !self.terminal_tabs[active].ftp.visible;
                if self.terminal_tabs[active].ftp.visible {
                    let host = self.terminal_tabs[active].host.clone();
                    self.terminal_tabs[active].ftp.connected_host = Some(host.clone());
                    self.terminal_tabs[active].ftp.current_path = "/".to_string();
                    self.terminal_tabs[active].ftp.loading = true;
                    self.terminal_tabs[active].ftp.status = FtpStatus::Idle;
                    let path = "/".to_string();
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || ftp::list_directory(&host, &path))
                                .await
                                .unwrap_or_else(|e| Err(e.to_string()))
                        },
                        Message::FtpListResult,
                    );
                } else {
                    self.terminal_tabs[active].ftp = FtpState::default();
                }
            }
            Message::FtpNavigate(path) => {
                let Some(active) = self.active_tab else { return Task::none(); };
                if let Some(host) = self.terminal_tabs[active].ftp.connected_host.clone() {
                    self.terminal_tabs[active].ftp.loading = true;
                    self.terminal_tabs[active].ftp.current_path = path.clone();
                    self.terminal_tabs[active].ftp.notification = None;
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || ftp::list_directory(&host, &path))
                                .await
                                .unwrap_or_else(|e| Err(e.to_string()))
                        },
                        Message::FtpListResult,
                    );
                }
            }
            Message::FtpRefresh => {
                let Some(active) = self.active_tab else { return Task::none(); };
                let path = self.terminal_tabs[active].ftp.current_path.clone();
                return self.update(Message::FtpNavigate(path));
            }
            Message::FtpListResult(result) => {
                let Some(active) = self.active_tab else { return Task::none(); };
                self.terminal_tabs[active].ftp.loading = false;
                match result {
                    Ok(entries) => {
                        self.terminal_tabs[active].ftp.entries = entries;
                        self.terminal_tabs[active].ftp.status = FtpStatus::Idle;
                    }
                    Err(e) => {
                        self.terminal_tabs[active].ftp.status = FtpStatus::Error(e);
                    }
                }
            }
            Message::FtpEntryClick(path) => {
                let Some(active) = self.active_tab else { return Task::none(); };
                let now = std::time::Instant::now();
                let is_double = self.terminal_tabs[active]
                    .ftp
                    .last_click
                    .as_ref()
                    .map(|(p, t)| p == &path && now.duration_since(*t).as_millis() < 400)
                    .unwrap_or(false);
                if is_double {
                    self.terminal_tabs[active].ftp.last_click = None;
                    return self.update(Message::FtpDownloadFile(path));
                } else {
                    self.terminal_tabs[active].ftp.last_click = Some((path.clone(), now));
                    let cmd = format!("nano \"{}\"\r", path);
                    return self.update(Message::TerminalSendBytes(cmd.into_bytes()));
                }
            }
            Message::FtpDownloadFile(remote_path) => {
                let Some(active) = self.active_tab else { return Task::none(); };
                if let Some(host) = self.terminal_tabs[active].ftp.connected_host.clone() {
                    let file_name = std::path::Path::new(&remote_path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "file".to_string());
                    let dl_dir = directories::UserDirs::new()
                        .and_then(|u| u.download_dir().map(|p| p.to_path_buf()))
                        .unwrap_or_else(|| std::path::PathBuf::from("."));
                    let local_path = dl_dir.join(&file_name).to_string_lossy().to_string();
                    self.terminal_tabs[active].ftp.notification = Some(("Downloading...".to_string(), false));
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                ftp::download_file(&host, &remote_path, &local_path)
                                    .map(|_| local_path)
                            })
                            .await
                            .unwrap_or_else(|e| Err(e.to_string()))
                        },
                        Message::FtpDownloadResult,
                    );
                }
            }
            Message::FtpDownloadResult(result) => {
                let Some(active) = self.active_tab else { return Task::none(); };
                match result {
                    Ok(path) => {
                        self.terminal_tabs[active].ftp.notification =
                            Some((format!("Downloaded → {}", path), false));
                    }
                    Err(e) => {
                        self.terminal_tabs[active].ftp.notification =
                            Some((format!("Download failed: {}", e), true));
                    }
                }
            }
            Message::FtpPickUploadFile => {
                return Task::perform(
                    async {
                        tokio::task::spawn_blocking(|| {
                            rfd::FileDialog::new()
                                .set_title("Select File to Upload")
                                .pick_file()
                        })
                        .await
                        .ok()
                        .flatten()
                    },
                    Message::FtpUploadChosen,
                );
            }
            Message::FtpUploadChosen(maybe_path) => {
                let Some(active) = self.active_tab else { return Task::none(); };
                if let Some(local) = maybe_path {
                    if let Some(host) = self.terminal_tabs[active].ftp.connected_host.clone() {
                        let local_str = local.to_string_lossy().to_string();
                        let file_name = local
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "upload".to_string());
                        let remote_path = format!(
                            "{}/{}",
                            self.terminal_tabs[active].ftp.current_path.trim_end_matches('/'),
                            file_name
                        );
                        self.terminal_tabs[active].ftp.notification =
                            Some(("Uploading...".to_string(), false));
                        return Task::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    ftp::upload_file(&host, &local_str, &remote_path)
                                })
                                .await
                                .unwrap_or_else(|e| Err(e.to_string()))
                            },
                            Message::FtpUploadResult,
                        );
                    }
                }
            }
            Message::FtpUploadResult(result) => {
                let Some(active) = self.active_tab else { return Task::none(); };
                match result {
                    Ok(_) => {
                        self.terminal_tabs[active].ftp.notification =
                            Some(("Upload complete".to_string(), false));
                        let path = self.terminal_tabs[active].ftp.current_path.clone();
                        return self.update(Message::FtpNavigate(path));
                    }
                    Err(e) => {
                        self.terminal_tabs[active].ftp.notification =
                            Some((format!("Upload failed: {}", e), true));
                    }
                }
            }
            // ── Terminal UX features ──────────────────────────────────────
            Message::TerminalFontSizeInc => {
                let Some(i) = self.active_tab else { return Task::none(); };
                self.terminal_tabs[i].font_size = (self.terminal_tabs[i].font_size + 1.0).min(28.0);
            }
            Message::TerminalFontSizeDec => {
                let Some(i) = self.active_tab else { return Task::none(); };
                self.terminal_tabs[i].font_size = (self.terminal_tabs[i].font_size - 1.0).max(8.0);
            }
            Message::TerminalFontSizeReset => {
                let Some(i) = self.active_tab else { return Task::none(); };
                self.terminal_tabs[i].font_size = 13.0;
            }
            Message::TerminalSearchToggle => {
                let Some(i) = self.active_tab else { return Task::none(); };
                let was = self.terminal_tabs[i].search_active;
                self.terminal_tabs[i].search_active = !was;
                if was {
                    self.terminal_tabs[i].search_query.clear();
                }
            }
            Message::TerminalSearchChanged(q) => {
                let Some(i) = self.active_tab else { return Task::none(); };
                self.terminal_tabs[i].search_query = q;
            }
            Message::TerminalSearchClose => {
                let Some(i) = self.active_tab else { return Task::none(); };
                self.terminal_tabs[i].search_active = false;
                self.terminal_tabs[i].search_query.clear();
            }
            Message::TerminalQuickCmdsToggle => {
                let Some(i) = self.active_tab else { return Task::none(); };
                let v = self.terminal_tabs[i].quick_cmds_visible;
                self.terminal_tabs[i].quick_cmds_visible = !v;
            }
            Message::TerminalQuickCmd(cmd) => {
                return self.update(Message::TerminalSendBytes(cmd.into_bytes()));
            }
            Message::SettingsLayoutChanged(preset) => {
                if let Some(dialogs::DialogState::Settings(ref mut form)) = self.dialog {
                    form.layout = preset;
                }
            }
            Message::SettingsFontSizeChanged(size) => {
                if let Some(dialogs::DialogState::Settings(ref mut form)) = self.dialog {
                    form.terminal_font_size = size.clamp(8.0, 28.0);
                }
            }
            Message::SettingsShowBordersChanged(val) => {
                if let Some(dialogs::DialogState::Settings(ref mut form)) = self.dialog {
                    form.show_borders = val;
                }
            }
            Message::SettingsSuggestionsChanged(val) => {
                if let Some(dialogs::DialogState::Settings(ref mut form)) = self.dialog {
                    form.suggestions_enabled = val;
                }
            }
            Message::TerminalScrollModeToggle => {
                self.scroll_mode = !self.scroll_mode;
                if !self.scroll_mode {
                    // Re-snap to bottom when leaving scroll mode
                    self.scroll_position = 1.0;
                    return scrollable::snap_to(
                        self.terminal_scroll_id.clone(),
                        scrollable::RelativeOffset { x: 0.0, y: 1.0 },
                    );
                }
            }
            Message::TerminalScrollBy(delta) => {
                self.scroll_position = (self.scroll_position + delta).clamp(0.0, 1.0);
                return scrollable::snap_to(
                    self.terminal_scroll_id.clone(),
                    scrollable::RelativeOffset { x: 0.0, y: self.scroll_position },
                );
            }
            Message::TerminalKeyPressed(key, modifiers) => {
                if self.dialog.is_some() {
                    return Task::none();
                }

                // Scroll mode: intercept arrows for terminal scrolling
                if self.scroll_mode {
                    match &key {
                        Key::Named(Named::ArrowUp) => {
                            return self.update(Message::TerminalScrollBy(-0.05));
                        }
                        Key::Named(Named::ArrowDown) => {
                            return self.update(Message::TerminalScrollBy(0.05));
                        }
                        Key::Named(Named::PageUp) => {
                            return self.update(Message::TerminalScrollBy(-0.20));
                        }
                        Key::Named(Named::PageDown) => {
                            return self.update(Message::TerminalScrollBy(0.20));
                        }
                        Key::Named(Named::Home) => {
                            return self.update(Message::TerminalScrollBy(-1.0));
                        }
                        Key::Named(Named::End) => {
                            return self.update(Message::TerminalScrollBy(1.0));
                        }
                        Key::Named(Named::Escape) | Key::Character(_) => {
                            // Any printable key exits scroll mode and passes through
                            self.scroll_mode = false;
                        }
                        _ => return Task::none(),
                    }
                }

                // Intercept terminal shortcuts before passing to SSH
                if modifiers.control() {
                    if let Key::Character(ref c) = key {
                        match c.as_str() {
                            "f" => return self.update(Message::TerminalSearchToggle),
                            "=" | "+" => return self.update(Message::TerminalFontSizeInc),
                            "-" => return self.update(Message::TerminalFontSizeDec),
                            "0" => return self.update(Message::TerminalFontSizeReset),
                            // Ctrl+V → paste from system clipboard
                            "v" => {
                                return iced::clipboard::read().map(|content| {
                                    Message::TerminalSendBytes(
                                        content.unwrap_or_default().into_bytes(),
                                    )
                                });
                            }
                            _ => {}
                        }
                    }
                    // Ctrl+Space → start/reset suggestion keyboard navigation
                    if matches!(key, Key::Named(Named::Space)) {
                        if let Some(active) = self.active_tab {
                            let triggers: Vec<String> = self.config.custom_commands.iter().map(|c| c.trigger.clone()).collect();
                            let has_suggestions = !compute_suggestions(&self.terminal_tabs[active], &triggers).is_empty();
                            if has_suggestions {
                                return self.update(Message::TerminalSuggestionMove(1));
                            }
                        }
                    }
                }

                // Suggestion panel arrow-key navigation / Tab accept / Esc dismiss
                if let Some(active) = self.active_tab {
                    let sugg_idx = self.terminal_tabs
                        .get(active)
                        .and_then(|t| t.suggestion_index);

                    match &key {
                        // ArrowDown: navigate suggestions only when already in suggestion mode.
                        // When sugg_idx is None, ArrowDown passes through to SSH so bash
                        // history navigation (↓ key) works normally without accidental
                        // suggestion activation that can trigger alias scripts.
                        Key::Named(Named::ArrowDown) if sugg_idx.is_some() => {
                            return self.update(Message::TerminalSuggestionMove(1));
                        }
                        // ArrowUp: navigate up or deselect if at top
                        Key::Named(Named::ArrowUp) if sugg_idx.is_some() => {
                            return self.update(Message::TerminalSuggestionMove(-1));
                        }
                        // Tab: accept highlighted suggestion (if any)
                        Key::Named(Named::Tab) if sugg_idx.is_some() => {
                            let triggers: Vec<String> = self.config.custom_commands.iter().map(|c| c.trigger.clone()).collect();
                            let suggestions = compute_suggestions(&self.terminal_tabs[active], &triggers);
                            if let Some(idx) = sugg_idx {
                                if let Some(cmd) = suggestions.get(idx).cloned() {
                                    return self.update(Message::TerminalSuggestionAccept(cmd));
                                }
                            }
                            // No match — fall through and send Tab to SSH
                        }
                        // Esc: dismiss suggestion selection
                        Key::Named(Named::Escape) if sugg_idx.is_some() => {
                            self.terminal_tabs[active].suggestion_index = None;
                            return Task::none();
                        }
                        _ => {}
                    }
                }

                // Esc → close search (when no suggestion was dismissed above)
                if matches!(key, Key::Named(Named::Escape)) {
                    let searching = self.active_tab
                        .and_then(|i| self.terminal_tabs.get(i))
                        .map(|t| t.search_active)
                        .unwrap_or(false);
                    if searching {
                        return self.update(Message::TerminalSearchClose);
                    }
                }
                if let Some(bytes) = map_key_to_bytes(key, modifiers) {
                    return self.update(Message::TerminalSendBytes(bytes));
                }
            }
            Message::TerminalSendBytes(mut bytes) => {
                if self.dialog.is_some() {
                    return Task::none();
                }

                // Phase 1: Track local input buffer + intercept custom commands
                if let Some(active) = self.active_tab {
                    if bytes.len() == 1 && bytes[0] == 13 {
                        // Enter pressed — check for custom command alias
                        let buffer = self
                            .terminal_tabs
                            .get(active)
                            .map(|t| t.input_buffer.trim().to_string())
                            .unwrap_or_default();

                        if !buffer.is_empty() {
                            let custom = self
                                .config
                                .custom_commands
                                .iter()
                                .find(|c| c.trigger == buffer)
                                .cloned();

                            if let Some(cc) = custom {
                                // Replace with Ctrl+U (clear line) + script + \r
                                let mut replacement = vec![21u8];
                                replacement.extend_from_slice(cc.script.as_bytes());
                                replacement.push(b'\r');
                                bytes = replacement;
                            } else if let Some(tab) = self.terminal_tabs.get_mut(active) {
                                if tab.command_history.last().map(String::as_str) != Some(buffer.as_str()) {
                                    tab.command_history.push(buffer);
                                    if tab.command_history.len() > 50 {
                                        tab.command_history.remove(0);
                                    }
                                }
                            }
                        }
                        if let Some(tab) = self.terminal_tabs.get_mut(active) {
                            tab.input_buffer.clear();
                        }
                    } else if let Some(tab) = self.terminal_tabs.get_mut(active) {
                        if bytes.len() == 1 {
                            match bytes[0] {
                                127 => {
                                    tab.input_buffer.pop();
                                    tab.suggestion_index = None;
                                }
                                3 | 21 | 27 => {
                                    tab.input_buffer.clear();
                                    tab.suggestion_index = None;
                                }
                                b if b >= 32 => {
                                    tab.input_buffer.push(b as char);
                                    tab.suggestion_index = None;
                                }
                                _ => {}
                            }
                        } else if !bytes.is_empty() {
                            if bytes[0] == 27 {
                                // Escape sequence (arrow keys, function keys, cursor movement).
                                // Clear the buffer because these keys can move the cursor or
                                // navigate bash history, making the local buffer unreliable.
                                // Without this, a stale buffer could accidentally match an
                                // alias trigger and run its script unexpectedly.
                                tab.input_buffer.clear();
                                tab.suggestion_index = None;
                            } else if bytes.iter().all(|&b| b >= 32) {
                                // Multi-byte printable text (e.g., UTF-8 from IME or paste)
                                if let Ok(s) = std::str::from_utf8(&bytes) {
                                    tab.input_buffer.push_str(s);
                                    tab.suggestion_index = None;
                                }
                            }
                        }
                    }
                }

                // Phase 2: Send bytes to SSH stdin
                let mut should_snap_bottom = false;
                if let Some(active) = self.active_tab {
                    if let Some(tab) = self.terminal_tabs.get(active) {
                        if let Some(runtime) = self.terminal_runtime.get(&tab.id) {
                            let in_alternate_screen = runtime.parser.screen().alternate_screen();
                            if let Ok(mut stdin) = runtime.stdin.lock() {
                                let _ = stdin.write_all(&bytes);
                                let _ = stdin.flush();
                            }
                            // Only snap to bottom when not in scroll mode
                            should_snap_bottom = !in_alternate_screen && !self.scroll_mode;
                        }
                    }
                }
                if should_snap_bottom {
                    self.scroll_position = 1.0;
                    return scrollable::snap_to(
                        self.terminal_scroll_id.clone(),
                        scrollable::RelativeOffset { x: 0.0, y: 1.0 },
                    );
                }
                return Task::none();
            }
            Message::TerminalSendCtrlC => {
                return self.update(Message::TerminalSendBytes(vec![3]));
            }
            Message::TerminalClear => {
                if let Some(active) = self.active_tab {
                    if let Some(tab) = self.terminal_tabs.get_mut(active) {
                        tab.output.clear();
                        if let Some(runtime) = self.terminal_runtime.get_mut(&tab.id) {
                            runtime.parser =
                                Parser::new(TERMINAL_ROWS, TERMINAL_COLS, 10_000);
                        }
                    }
                }
            }
            Message::TerminalPoll => {
                let ids: Vec<u64> = self.terminal_runtime.keys().copied().collect();
                let mut to_remove: Vec<u64> = Vec::new();
                let mut should_snap_bottom = false;
                let mut should_snap_top = false;
                let active_id = self
                    .active_tab
                    .and_then(|idx| self.terminal_tabs.get(idx))
                    .map(|tab| tab.id);

                for id in ids {
                    let mut changed = false;
                    let mut should_remove = false;

                    if let Some(runtime) = self.terminal_runtime.get_mut(&id) {
                        loop {
                            match runtime.rx.try_recv() {
                                Ok(chunk) => {
                                    runtime.parser.process(&chunk);
                                    changed = true;
                                }
                                Err(mpsc::TryRecvError::Empty) => break,
                                Err(mpsc::TryRecvError::Disconnected) => {
                                    should_remove = true;
                                    break;
                                }
                            }
                        }

                        if let Ok(Some(status)) = runtime.child.try_wait() {
                            let exit_line = format!("\r\n[relay exited: {}]\r\n", status);
                            runtime.parser.process(exit_line.as_bytes());
                            changed = true;
                            should_remove = true;
                        }

                        if changed {
                            if let Some(tab) = self.terminal_tabs.iter_mut().find(|t| t.id == id) {
                                tab.output =
                                    normalized_screen(&runtime.parser.screen().contents());
                                if Some(id) == active_id {
                                    if runtime.parser.screen().alternate_screen() {
                                        should_snap_top = true;
                                        should_snap_bottom = false;
                                    } else if !should_snap_top {
                                        should_snap_bottom = true;
                                    }
                                }
                            }
                        }
                    }

                    if should_remove {
                        to_remove.push(id);
                    }
                }

                for id in to_remove {
                    self.terminal_runtime.remove(&id);
                    if let Some(tab) = self.terminal_tabs.iter_mut().find(|t| t.id == id) {
                        tab.connected = false;
                    }
                }

                if should_snap_top {
                    self.scroll_position = 0.0;
                    return scrollable::snap_to(
                        self.terminal_scroll_id.clone(),
                        scrollable::RelativeOffset { x: 0.0, y: 0.0 },
                    );
                }

                // Only auto-snap to bottom when NOT in scroll mode
                if should_snap_bottom && !self.scroll_mode {
                    self.scroll_position = 1.0;
                    return scrollable::snap_to(
                        self.terminal_scroll_id.clone(),
                        scrollable::RelativeOffset { x: 0.0, y: 1.0 },
                    );
                }
            }
            Message::TerminalEvent(_id, _event) => {
                // TODO: Handle terminal events when iced_term is integrated
            }

            // ── Command suggestions ───────────────────────────────────────
            Message::TerminalSuggestionAccept(cmd) => {
                let Some(i) = self.active_tab else { return Task::none(); };
                // Update local buffer to reflect what we're inserting
                self.terminal_tabs[i].input_buffer = cmd.clone();
                self.terminal_tabs[i].suggestion_index = None;
                // Send Ctrl+U to clear current input, then type the suggestion
                let mut bytes = vec![21u8];
                bytes.extend_from_slice(cmd.as_bytes());
                if let Some(tab) = self.terminal_tabs.get(i) {
                    if let Some(runtime) = self.terminal_runtime.get(&tab.id) {
                        if let Ok(mut stdin) = runtime.stdin.lock() {
                            let _ = stdin.write_all(&bytes);
                            let _ = stdin.flush();
                        }
                    }
                }
            }
            Message::TerminalSuggestionMove(delta) => {
                let Some(i) = self.active_tab else { return Task::none(); };
                let triggers: Vec<String> = self.config.custom_commands.iter().map(|c| c.trigger.clone()).collect();
                let suggestions = compute_suggestions(&self.terminal_tabs[i], &triggers);
                if suggestions.is_empty() {
                    return Task::none();
                }
                let current = self.terminal_tabs[i].suggestion_index;
                let new_idx = match current {
                    None => {
                        if delta > 0 { Some(0) } else { None }
                    }
                    Some(idx) => {
                        let next = idx as i32 + delta;
                        if next < 0 {
                            None
                        } else {
                            Some((next as usize).min(suggestions.len().saturating_sub(1)))
                        }
                    }
                };
                self.terminal_tabs[i].suggestion_index = new_idx;
            }
            Message::TerminalCopyOutput => {
                let Some(i) = self.active_tab else { return Task::none(); };
                let content = self.terminal_tabs[i].output.clone();
                return iced::clipboard::write::<Message>(content);
            }

            // ── Security audit ────────────────────────────────────────────
            Message::OpenSecurityAudit => {
                let findings = run_security_audit(&self.config, &self.api_url);
                self.dialog = Some(dialogs::DialogState::SecurityAudit(findings));
            }

            // ── Custom commands (aliases) ─────────────────────────────────
            Message::OpenCustomCommands => {
                self.dialog = Some(dialogs::DialogState::CustomCommands(
                    dialogs::CustomCommandsForm {
                        commands: self.config.custom_commands.clone(),
                        new_trigger: String::new(),
                        new_script: String::new(),
                        new_description: String::new(),
                    },
                ));
            }
            Message::AddCustomCommand => {
                if let Some(dialogs::DialogState::CustomCommands(ref mut form)) = self.dialog {
                    let trigger = form.new_trigger.trim().to_string();
                    let script = form.new_script.trim().to_string();
                    if !trigger.is_empty() && !script.is_empty() {
                        form.commands.push(config::CustomCommand {
                            trigger,
                            script,
                            description: form.new_description.trim().to_string(),
                        });
                        form.new_trigger.clear();
                        form.new_script.clear();
                        form.new_description.clear();
                    }
                }
            }
            Message::DeleteCustomCommand(idx) => {
                if let Some(dialogs::DialogState::CustomCommands(ref mut form)) = self.dialog {
                    if idx < form.commands.len() {
                        form.commands.remove(idx);
                    }
                }
            }
            Message::SaveCustomCommands => {
                if let Some(dialogs::DialogState::CustomCommands(ref form)) = self.dialog {
                    self.config.custom_commands = form.commands.clone();
                    let _ = config::save_config(&self.config);
                }
                self.dialog = None;
            }

            // ── System Panel ──────────────────────────────────────────────────
            Message::SysPanelOpen(tab_id) => {
                if let Some(tab) = self.terminal_tabs.iter_mut().find(|t| t.id == tab_id) {
                    tab.sys_open = true;
                    tab.sys_state = crate::syspanel::SysState::new();
                    let host = tab.host.clone();
                    return crate::syspanel::fetch_overview(host, tab_id);
                }
            }
            Message::SysPanelClose(tab_id) => {
                if let Some(tab) = self.terminal_tabs.iter_mut().find(|t| t.id == tab_id) {
                    tab.sys_open = false;
                }
            }
            Message::SysPanelTabSwitch(tab_id, tab_name) => {
                if let Some(tab) = self.terminal_tabs.iter_mut().find(|t| t.id == tab_id) {
                    let new_tab = crate::syspanel::SysTab::from_str(&tab_name);
                    tab.sys_state.tab = new_tab.clone();
                    tab.sys_state.loading = true;
                    tab.sys_state.output.clear();
                    tab.sys_state.action_result = None;
                    let host = tab.host.clone();
                    return match new_tab {
                        crate::syspanel::SysTab::Overview => crate::syspanel::fetch_overview(host, tab_id),
                        crate::syspanel::SysTab::Firewall => crate::syspanel::fetch_firewall(host, tab_id),
                        crate::syspanel::SysTab::Packages => crate::syspanel::fetch_packages(host, tab_id),
                        crate::syspanel::SysTab::Logins => crate::syspanel::fetch_logins(host, tab_id),
                        crate::syspanel::SysTab::SshKeys => crate::syspanel::fetch_ssh_keys(host, tab_id),
                        crate::syspanel::SysTab::Extension(ref id) => {
                            crate::syspanel::fetch_extension(host, tab_id, id.clone())
                        }
                    };
                }
            }
            Message::SysPanelInput(tab_id, field, value) => {
                if let Some(tab) = self.terminal_tabs.iter_mut().find(|t| t.id == tab_id) {
                    match field.as_str() {
                        "fw_port"   => tab.sys_state.fw_port = value,
                        "fw_proto"  => tab.sys_state.fw_proto = value,
                        "fw_action" => tab.sys_state.fw_action = value,
                        "pkg_search" => tab.sys_state.pkg_search = value,
                        "key_name"  => tab.sys_state.key_name = value,
                        "key_type"  => tab.sys_state.key_type = value,
                        _ => {}
                    }
                }
            }
            Message::SysPanelFetch(tab_id, kind) => {
                if let Some(tab) = self.terminal_tabs.iter_mut().find(|t| t.id == tab_id) {
                    tab.sys_state.loading = true;
                    tab.sys_state.output.clear();
                    tab.sys_state.action_result = None;
                    let host = tab.host.clone();
                    return match kind.as_str() {
                        "overview"  => crate::syspanel::fetch_overview(host, tab_id),
                        "firewall"  => crate::syspanel::fetch_firewall(host, tab_id),
                        "packages"  => crate::syspanel::fetch_packages(host, tab_id),
                        "logins"    => crate::syspanel::fetch_logins(host, tab_id),
                        "sshkeys"   => crate::syspanel::fetch_ssh_keys(host, tab_id),
                        ext_id      => crate::syspanel::fetch_extension(host, tab_id, ext_id.to_string()),
                    };
                }
            }
            Message::SysPanelAction(tab_id, cmd) => {
                if let Some(tab) = self.terminal_tabs.iter_mut().find(|t| t.id == tab_id) {
                    tab.sys_state.loading = true;
                    tab.sys_state.action_result = None;
                    let host = tab.host.clone();
                    return crate::syspanel::run_action(host, tab_id, cmd);
                }
            }
            Message::SysPanelFetched(tab_id, kind, output) => {
                if let Some(tab) = self.terminal_tabs.iter_mut().find(|t| t.id == tab_id) {
                    tab.sys_state.loading = false;
                    match kind.as_str() {
                        "action" => {
                            tab.sys_state.action_result = Some(output.lines().last().unwrap_or("Done").to_string());
                            // Refresh current panel after action
                            let host = tab.host.clone();
                            let current_tab = tab.sys_state.tab.clone();
                            return match current_tab {
                                crate::syspanel::SysTab::Firewall => crate::syspanel::fetch_firewall(host, tab_id),
                                crate::syspanel::SysTab::Extension(ref id) => crate::syspanel::fetch_extension(host, tab_id, id.clone()),
                                _ => { tab.sys_state.output = output; Task::none() }
                            };
                        }
                        "overview" => {
                            tab.sys_state.extensions = crate::syspanel::parse_extensions(&output);
                            tab.sys_state.output = output;
                        }
                        _ => {
                            tab.sys_state.output = output;
                        }
                    }
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let texts = Texts::get(self.config.language);
        let p = theme::palette(self.theme);
        let lc = theme::layout(self.config.layout);

        let toolbar_view = toolbar::view(&texts, self.theme, lc);
        let tab_bar_view = tab_bar::view(&self.terminal_tabs, self.active_tab, self.theme, lc);
        let structure: &[String] = self
            .active_tab
            .and_then(|i| self.terminal_tabs.get(i))
            .map(|t| t.structure.as_slice())
            .unwrap_or(&[]);
        let sidebar_view = sidebar::view(
            &texts,
            &self.config.hosts,
            &self.search_query,
            self.selected_host,
            &self.ping_results,
            &self.system_info,
            structure,
            self.theme,
            lc,
        );
        let status_view = status_bar::view(
            &texts,
            self.config.api_key.is_some(),
            self.config.language,
            self.theme,
            lc,
        );

        let main_area = self.view_main_area(&texts, lc);

        let pg = lc.panel_gap;
        let cp = lc.container_padding;
        let content = column![
            toolbar_view,
            tab_bar_view,
            row![sidebar_view, main_area].spacing(pg).height(Length::Fill),
            status_view,
        ]
        .spacing(pg)
        .padding(cp);

        let base: Element<'_, Message> = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_t: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(p.bg_primary)),
                ..Default::default()
            })
            .into();

        if let Some(ref dialog_state) = self.dialog {
            let dialog_overlay = dialogs::view_dialog(&texts, dialog_state, self.theme, lc);
            iced::widget::stack![base, dialog_overlay].into()
        } else {
            base
        }
    }

    fn view_main_area(&self, texts: &Texts, lc: theme::LayoutConfig) -> Element<'_, Message> {
        let p = theme::palette(self.theme);
        let cr = lc.corner_radius;

        if let Some(active) = self.active_tab {
            if let Some(tab) = self.terminal_tabs.get(active) {
                // If system panel is open, show it instead of the terminal
                if tab.sys_open {
                    return crate::syspanel::view_sys_panel(
                        tab.id,
                        &tab.sys_state,
                        &tab.host,
                        self.theme,
                        self.config.layout,
                    );
                }

                let status_text = if tab.connected { "connected" } else { "disconnected" };
                let status_color = if tab.connected { p.success } else { p.danger };

                // Build top_bar with optional layout-toggle button (top-right)
                let mut top_bar_row = iced::widget::Row::new()
                    .spacing(4)
                    .align_y(iced::Alignment::Center);
                let scroll_mode = self.scroll_mode;
                top_bar_row = top_bar_row
                    .push(
                        text(format!(
                            "{}@{}:{}",
                            tab.host.username, tab.host.hostname, tab.host.port
                        ))
                        .size(11)
                        .color(p.text_muted),
                    )
                    .push(text(format!("  ·  {}", status_text)).size(11).color(status_color));
                if scroll_mode {
                    top_bar_row = top_bar_row.push(
                        container(
                            text("  SCROLL MODE  ↑↓/PgUp/PgDn · Esc or type to exit")
                                .size(10)
                                .color(p.accent),
                        )
                        .padding([2, 8])
                        .style(move |_: &iced::Theme| container::Style {
                            background: Some(iced::Background::Color(p.bg_tertiary)),
                            border: iced::Border { color: p.accent, width: 1.0, radius: cr.into() },
                            ..Default::default()
                        }),
                    );
                }
                top_bar_row = top_bar_row.push(iced::widget::horizontal_space());
                if tab.ftp.visible {
                    let layout_label = match tab.ftp.layout {
                        FtpLayout::Bottom => "Right Side",  // switch to Right
                        FtpLayout::Right  => "Bottom Side",  // switch to Bottom
                    };
                    top_bar_row =
                        top_bar_row.push(terminal_action_button(layout_label, Message::FtpToggleLayout, p));
                }
                top_bar_row = top_bar_row
                    .push(terminal_action_button(
                        if tab.quick_cmds_visible { "CMD ●" } else { "CMD" },
                        Message::TerminalQuickCmdsToggle, p,
                    ))
                    .push(terminal_action_button(
                        if tab.search_active { "Search ●" } else { "Search" },
                        Message::TerminalSearchToggle, p,
                    ))
                    .push(terminal_action_button(
                        if scroll_mode { "SCROLL ●" } else { "SCROLL" },
                        Message::TerminalScrollModeToggle, p,
                    ))
                    .push(terminal_action_button("A-", Message::TerminalFontSizeDec, p))
                    .push(terminal_action_button("A+", Message::TerminalFontSizeInc, p))
                    .push(terminal_action_button("^C", Message::TerminalSendCtrlC, p))
                    .push(terminal_action_button("Copy", Message::TerminalCopyOutput, p))
                    .push(terminal_action_button("Clear", Message::TerminalClear, p))
                    .push(terminal_action_button("⚙ System", Message::SysPanelOpen(tab.id), p));
                let top_bar = top_bar_row;

                // Terminal spans — with optional search highlight
                let raw_spans = self
                    .terminal_runtime
                    .get(&tab.id)
                    .map(|rt| build_terminal_spans(rt, p.text_primary))
                    .unwrap_or_else(|| {
                        let fallback = if tab.output.is_empty() {
                            " ".to_string()
                        } else {
                            tab.output.clone()
                        };
                        vec![iced::widget::text::Span::new(fallback)]
                    });

                let (terminal_spans, match_count) = if tab.search_active
                    && !tab.search_query.is_empty()
                {
                    apply_search_highlight(
                        raw_spans,
                        &tab.search_query,
                        iced::Color::from_rgb(1.0, 0.85, 0.0),
                        p.text_primary,
                    )
                } else {
                    (raw_spans, 0)
                };

                let in_alternate_screen = self
                    .terminal_runtime
                    .get(&tab.id)
                    .map(|rt| rt.parser.screen().alternate_screen())
                    .unwrap_or(false);

                // Per-tab font size (overrides global default when explicitly changed)
                let font_sz = if (tab.font_size - 13.0).abs() < 0.1 {
                    self.config.terminal_font_size
                } else {
                    tab.font_size
                };
                let terminal_view = container(
                    scrollable(
                        rich_text(terminal_spans)
                            .size(font_sz)
                            .font(Font::MONOSPACE)
                            .wrapping(iced::widget::text::Wrapping::None)
                            .width(Length::Fill),
                    )
                    .id(self.terminal_scroll_id.clone())
                    .style(hidden_scrollbar_style)
                    .height(Length::Fill),
                )
                .padding([8, 10])
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_t: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(p.bg_secondary)),
                    border: iced::Border::default(),
                    ..Default::default()
                });

                // Build panel incrementally for conditional bars
                let mut panel = if in_alternate_screen {
                    Column::new().spacing(0).height(Length::Fill)
                } else {
                    Column::new().spacing(4).height(Length::Fill).push(top_bar)
                };

                // Quick commands bar (with recent history section)
                if tab.quick_cmds_visible && !in_alternate_screen {
                    // Row 1: built-in quick commands
                    let mut cmd_row = iced::widget::Row::new()
                        .spacing(3)
                        .padding([2, 6])
                        .align_y(Alignment::Center);
                    for (label, cmd) in QUICK_CMDS {
                        let cmd_str = (*cmd).to_string();
                        cmd_row = cmd_row.push(
                            button(text(*label).size(10).color(p.text_secondary))
                                .on_press(Message::TerminalQuickCmd(cmd_str))
                                .padding([2, 7])
                                .style(move |_: &iced::Theme, s: button::Status| button::Style {
                                    background: Some(iced::Background::Color(match s {
                                        button::Status::Hovered => p.bg_hover,
                                        _ => p.bg_tertiary,
                                    })),
                                    text_color: p.text_secondary,
                                    border: iced::Border {
                                        color: p.border,
                                        width: 1.0,
                                        radius: cr.into(),
                                    },
                                    ..Default::default()
                                }),
                        );
                    }
                    let qc_bar = container(cmd_row)
                        .width(Length::Fill)
                        .padding([0, 2])
                        .style(move |_: &iced::Theme| container::Style {
                            background: Some(iced::Background::Color(p.bg_tertiary)),
                            border: iced::Border { color: p.border, width: 1.0, radius: cr.into() },
                            ..Default::default()
                        });
                    panel = panel.push(qc_bar);

                    // Row 2: recent command history (last 8, newest first)
                    if !tab.command_history.is_empty() {
                        let mut hist_row = iced::widget::Row::new()
                            .spacing(3)
                            .padding([2, 6])
                            .align_y(Alignment::Center);
                        hist_row = hist_row.push(
                            text("hist:").size(9).color(p.text_muted)
                        );
                        for recent_cmd in tab.command_history.iter().rev().take(8) {
                            let cmd_owned = format!("{}\r", recent_cmd);
                            let label_owned = recent_cmd.clone();
                            hist_row = hist_row.push(
                                button(text(label_owned).size(10).color(p.accent))
                                    .on_press(Message::TerminalQuickCmd(cmd_owned))
                                    .padding([1, 6])
                                    .style(move |_: &iced::Theme, s: button::Status| button::Style {
                                        background: Some(iced::Background::Color(match s {
                                            button::Status::Hovered => p.bg_hover,
                                            _ => p.bg_primary,
                                        })),
                                        text_color: p.accent,
                                        border: iced::Border {
                                            color: p.border,
                                            width: 1.0,
                                            radius: cr.into(),
                                        },
                                        ..Default::default()
                                    }),
                            );
                        }
                        let hist_bar = container(hist_row)
                            .width(Length::Fill)
                            .padding([0, 2])
                            .style(move |_: &iced::Theme| container::Style {
                                background: Some(iced::Background::Color(p.bg_primary)),
                                border: iced::Border {
                                    color: p.border,
                                    width: 1.0,
                                    radius: cr.into(),
                                },
                                ..Default::default()
                            });
                        panel = panel.push(hist_bar);
                    }
                }

                // Search bar
                if tab.search_active && !in_alternate_screen {
                    let sq = tab.search_query.clone();
                    let mc = match_count;
                    let match_text = if sq.is_empty() {
                        "type to search".to_string()
                    } else {
                        format!("{} match{}", mc, if mc == 1 { "" } else { "es" })
                    };
                    let search_bar = container(
                        row![
                            text_input("Search terminal... (Ctrl+F, Esc)", &sq)
                                .on_input(Message::TerminalSearchChanged)
                                .on_submit(Message::TerminalSearchClose)
                                .padding([3, 6])
                                .size(11)
                                .width(Length::Fill)
                                .style(move |_: &iced::Theme, st: text_input::Status| {
                                    text_input::Style {
                                        background: iced::Background::Color(p.bg_primary),
                                        border: iced::Border {
                                            color: match st {
                                                text_input::Status::Focused => p.border_focused,
                                                _ => p.border,
                                            },
                                            width: 1.0,
                                            radius: cr.into(),
                                        },
                                        icon: p.text_muted,
                                        placeholder: p.text_muted,
                                        value: p.text_primary,
                                        selection: p.accent,
                                    }
                                }),
                            text(match_text).size(10).color(p.text_muted),
                            terminal_action_button("✕", Message::TerminalSearchClose, p),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .padding([3, 6])
                    .style(move |_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(p.bg_tertiary)),
                        border: iced::Border { color: p.accent, width: 1.0, radius: cr.into() },
                        ..Default::default()
                    });
                    panel = panel.push(search_bar);
                }

                panel = panel.push(terminal_view);

                if let Some(err) = &tab.relay_error {
                    panel = panel.push(text(format!("⚠ {}", err)).size(10).color(p.danger));
                }

                // Autocomplete panel — shown BELOW the terminal while user is typing
                if !tab.input_buffer.is_empty() && !in_alternate_screen && self.config.suggestions_enabled {
                    let alias_triggers: Vec<String> = self.config.custom_commands.iter().map(|c| c.trigger.clone()).collect();
                    let suggestions = compute_suggestions(tab, &alias_triggers);
                    if !suggestions.is_empty() {
                        let sugg_idx = tab.suggestion_index;
                        let history_set: std::collections::HashSet<String> =
                            tab.command_history.iter().cloned().collect();
                        let alias_set: std::collections::HashSet<String> =
                            alias_triggers.iter().cloned().collect();

                        let mut sugg_col = Column::new().spacing(0).width(Length::Fill);

                        // Hint header
                        sugg_col = sugg_col.push(
                            container(
                                row![
                                    text("Suggestions").size(9).color(p.text_muted),
                                    text("  Click or Ctrl+Space to select").size(9).color(p.text_muted),
                                    text("  ↑↓ navigate").size(9).color(p.text_muted),
                                    text("  Tab accept").size(9).color(p.text_muted),
                                    text("  Esc close").size(9).color(p.text_muted),
                                ]
                                .spacing(4)
                                .align_y(Alignment::Center),
                            )
                            .padding([2, 8])
                            .width(Length::Fill)
                            .style(move |_: &iced::Theme| container::Style {
                                background: Some(iced::Background::Color(p.bg_tertiary)),
                                ..Default::default()
                            }),
                        );

                        for (idx, suggestion) in suggestions.iter().enumerate() {
                            let is_selected = sugg_idx == Some(idx);
                            let is_alias = alias_set.contains(suggestion.as_str());
                            let from_history = history_set.contains(suggestion.as_str());
                            let text_color = if is_alias {
                                p.success
                            } else if from_history {
                                p.accent
                            } else {
                                p.text_secondary
                            };
                            let bg_color = if is_selected { p.bg_hover } else { p.bg_primary };
                            let cmd_str = suggestion.clone();
                            let prefix = if is_selected { "▶ " } else if is_alias { "⚡ " } else { "  " };
                            let label = suggestion.clone();
                            sugg_col = sugg_col.push(
                                button(
                                    row![
                                        text(prefix).size(11).color(if is_alias { p.success } else { p.accent }),
                                        text(label).size(11).color(text_color),
                                    ]
                                    .align_y(Alignment::Center),
                                )
                                .on_press(Message::TerminalSuggestionAccept(cmd_str))
                                .padding([3, 8])
                                .width(Length::Fill)
                                .style(move |_: &iced::Theme, s: button::Status| button::Style {
                                    background: Some(iced::Background::Color(match s {
                                        button::Status::Hovered | button::Status::Pressed => {
                                            p.bg_hover
                                        }
                                        _ => bg_color,
                                    })),
                                    text_color,
                                    border: iced::Border {
                                        color: if is_selected { p.border_focused } else { p.border },
                                        width: if is_selected { 1.0 } else { 0.0 },
                                        radius: 0.0.into(),
                                    },
                                    ..Default::default()
                                }),
                            );
                        }

                        let sugg_panel = container(sugg_col)
                            .width(Length::Fill)
                            .style(move |_: &iced::Theme| container::Style {
                                background: Some(iced::Background::Color(p.bg_primary)),
                                border: iced::Border {
                                    color: p.border_focused,
                                    width: 1.0,
                                    radius: cr.into(),
                                },
                                ..Default::default()
                            });
                        panel = panel.push(sugg_panel);
                    }
                }

                // Terminal container block
                let ftp_theme = self.theme;
                let borders = self.config.show_borders;
                let terminal_block = container(panel)
                    .padding([8, 10])
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(p.bg_secondary)),
                        border: iced::Border {
                            color: p.border,
                            width: if borders { 1.0 } else { 0.0 },
                            radius: cr.into(),
                        },
                        ..Default::default()
                    });

                // Attach FTP panel — position depends on tab.ftp.layout
                let pg = lc.panel_gap;
                let main_content: Element<'_, Message> = if tab.ftp.visible {
                    let ftp_view = ftp_panel::view(&tab.ftp, ftp_theme, lc);
                    match tab.ftp.layout {
                        FtpLayout::Bottom => column![terminal_block, ftp_view]
                            .spacing(pg)
                            .height(Length::Fill)
                            .into(),
                        FtpLayout::Right => row![terminal_block, ftp_view]
                            .spacing(pg)
                            .height(Length::Fill)
                            .into(),
                    }
                } else {
                    terminal_block.into()
                };
                return main_content;
            }
        }

        self.view_welcome(texts)
    }

    fn view_welcome(&self, texts: &Texts) -> Element<'_, Message> {
        let p = theme::palette(self.theme);
        container(
            column![
                text("termissh").size(20).color(p.accent),
                text(texts.welcome_msg).size(12).color(p.text_muted),
            ]
            .spacing(8)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg_secondary)),
            border: iced::Border {
                color: p.border,
                width: 1.0,
                radius: theme::CORNER_RADIUS.into(),
            },
            ..Default::default()
        })
        .into()
    }

    pub fn theme(&self) -> iced::Theme {
        if self.theme.is_light() {
            iced::Theme::Light
        } else {
            iced::Theme::Dark
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            iced::time::every(Duration::from_secs(2)).map(|_| Message::SystemInfoTick),
            iced::time::every(Duration::from_millis(50)).map(|_| Message::TerminalPoll),
            event::listen_with(runtime_event_to_message),
        ])
    }
}

fn runtime_event_to_message(
    event: iced::Event,
    status: iced::event::Status,
    _window: iced::window::Id,
) -> Option<Message> {
    if matches!(status, iced::event::Status::Captured) {
        return None;
    }

    if let iced::Event::Keyboard(keyboard::Event::KeyPressed {
        key: _key,
        modified_key,
        modifiers,
        text,
        ..
    }) = event
    {
        // Ctrl+character combos MUST bypass the `text` shortcut.
        // On Windows, Ctrl+V produces text="\x16" (byte 22) which would be
        // forwarded raw to SSH — our Ctrl+V (paste), Ctrl+F (search), etc.
        // shortcuts would never fire. Route through TerminalKeyPressed;
        // map_key_to_bytes still converts Ctrl+A→\x01, Ctrl+C→\x03, etc.
        if modifiers.control() {
            if let Key::Character(_) = &modified_key {
                return Some(Message::TerminalKeyPressed(modified_key, modifiers));
            }
        }

        if let Some(text) = text {
            if !text.is_empty() {
                return Some(Message::TerminalSendBytes(text.as_bytes().to_vec()));
            }
        }

        return Some(Message::TerminalKeyPressed(modified_key, modifiers));
    }

    None
}

fn map_key_to_bytes(key: Key, modifiers: Modifiers) -> Option<Vec<u8>> {
    let mapped = match key.as_ref() {
        Key::Named(Named::Enter) => Some(vec![b'\r']),
        Key::Named(Named::Tab) => Some(vec![b'\t']),
        Key::Named(Named::Backspace) => Some(vec![127]),
        Key::Named(Named::Escape) => Some(vec![27]),
        Key::Named(Named::ArrowUp) => Some(b"\x1b[A".to_vec()),
        Key::Named(Named::ArrowDown) => Some(b"\x1b[B".to_vec()),
        Key::Named(Named::ArrowRight) => Some(b"\x1b[C".to_vec()),
        Key::Named(Named::ArrowLeft) => Some(b"\x1b[D".to_vec()),
        Key::Named(Named::Home) => Some(b"\x1b[H".to_vec()),
        Key::Named(Named::End) => Some(b"\x1b[F".to_vec()),
        Key::Named(Named::Delete) => Some(b"\x1b[3~".to_vec()),
        Key::Named(Named::Insert) => Some(b"\x1b[2~".to_vec()),
        Key::Named(Named::PageUp) => Some(b"\x1b[5~".to_vec()),
        Key::Named(Named::PageDown) => Some(b"\x1b[6~".to_vec()),
        Key::Named(Named::Space) => Some(vec![b' ']),
        Key::Character(ch) => {
            let mut chars = ch.chars();
            let first = chars.next();
            if modifiers.control() {
                if let Some(c) = first {
                    if c.is_ascii_alphabetic() {
                        let ctrl = (c.to_ascii_lowercase() as u8 - b'a') + 1;
                        Some(vec![ctrl])
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                Some(ch.as_bytes().to_vec())
            }
        }
        _ => None,
    }?;

    if modifiers.alt() {
        let mut with_alt = vec![27];
        with_alt.extend(mapped);
        Some(with_alt)
    } else {
        Some(mapped)
    }
}

#[derive(Clone, Copy, PartialEq)]
struct TermSpanStyle {
    fg: iced::Color,
    bg: Option<iced::Color>,
    bold: bool,
    italic: bool,
    underline: bool,
}

fn build_terminal_spans(runtime: &TerminalRuntime, default_color: iced::Color) -> Vec<iced::widget::text::Span<'static, Message>> {
    let screen = runtime.parser.screen();
    let (rows, cols) = screen.size();

    let mut spans: Vec<iced::widget::text::Span<'static, Message>> = Vec::new();
    let mut current_text = String::new();
    let mut current_style = TermSpanStyle {
        fg: default_color,
        bg: None,
        bold: false,
        italic: false,
        underline: false,
    };

    for row in 0..rows {
        for col in 0..cols {
            let Some(cell) = screen.cell(row, col) else {
                continue;
            };
            if cell.is_wide_continuation() {
                continue;
            }

            let content = {
                let raw = cell.contents();
                if raw.is_empty() { " ".to_string() } else { raw }
            };

            let bg = match cell.bgcolor() {
                vt100::Color::Default => None,
                c => Some(vt_color_to_iced(c, default_color)),
            };
            let style = TermSpanStyle {
                fg: vt_color_to_iced(cell.fgcolor(), default_color),
                bg,
                bold: cell.bold(),
                italic: cell.italic(),
                underline: cell.underline(),
            };

            if style != current_style && !current_text.is_empty() {
                spans.push(span_from_style(&current_text, current_style));
                current_text.clear();
            }

            current_style = style;
            current_text.push_str(&content);
        }

        if row < rows.saturating_sub(1) {
            current_text.push('\n');
        }
    }

    if !current_text.is_empty() {
        spans.push(span_from_style(&current_text, current_style));
    }

    if spans.is_empty() {
        spans.push(iced::widget::text::Span::new(" ".to_string()));
    }

    spans
}

fn span_from_style(text_value: &str, style: TermSpanStyle) -> iced::widget::text::Span<'static, Message> {
    let mut font = Font::MONOSPACE;
    if style.bold {
        font.weight = iced::font::Weight::Bold;
    }
    if style.italic {
        font.style = iced::font::Style::Italic;
    }
    let mut s = iced::widget::text::Span::new(text_value.to_string())
        .color(style.fg)
        .font(font);
    if let Some(bg) = style.bg {
        s = s.background(iced::Background::Color(bg));
    }
    if style.underline {
        s = s.underline(true);
    }
    s
}

fn vt_color_to_iced(color: vt100::Color, default_color: iced::Color) -> iced::Color {
    match color {
        vt100::Color::Default => default_color,
        vt100::Color::Rgb(r, g, b) => iced::Color::from_rgb8(r, g, b),
        vt100::Color::Idx(idx) => ansi_index_to_color(idx),
    }
}

fn ansi_index_to_color(idx: u8) -> iced::Color {
    const ANSI16: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (205, 49, 49),
        (13, 188, 121),
        (229, 229, 16),
        (36, 114, 200),
        (188, 63, 188),
        (17, 168, 205),
        (229, 229, 229),
        (102, 102, 102),
        (241, 76, 76),
        (35, 209, 139),
        (245, 245, 67),
        (59, 142, 234),
        (214, 112, 214),
        (41, 184, 219),
        (255, 255, 255),
    ];

    if idx < 16 {
        let (r, g, b) = ANSI16[idx as usize];
        return iced::Color::from_rgb8(r, g, b);
    }

    if (16..=231).contains(&idx) {
        let v = idx - 16;
        let r = v / 36;
        let g = (v % 36) / 6;
        let b = v % 6;
        let to_255 = |n: u8| -> u8 { if n == 0 { 0 } else { 55 + n * 40 } };
        return iced::Color::from_rgb8(to_255(r), to_255(g), to_255(b));
    }

    let gray = 8 + (idx.saturating_sub(232) * 10);
    iced::Color::from_rgb8(gray, gray, gray)
}

// ─── Quick commands ────────────────────────────────────────────────────────
const QUICK_CMDS: &[(&str, &str)] = &[
    ("ls",      "ls -la\r"),
    ("pwd",     "pwd\r"),
    ("df",      "df -h\r"),
    ("free",    "free -h\r"),
    ("top",     "top\r"),
    ("ps",      "ps aux --sort=-%cpu | head -20\r"),
    ("hist",    "history | tail -30\r"),
    ("who",     "who\r"),
    ("uptime",  "uptime\r"),
    ("net",     "ss -tuln\r"),
    ("env",     "env | sort\r"),
    ("disk",    "du -sh * 2>/dev/null | sort -rh | head -20\r"),
];

// ─── Suggestion helpers ────────────────────────────────────────────────────

fn compute_suggestions(tab: &TerminalTab, alias_triggers: &[String]) -> Vec<String> {
    if tab.input_buffer.is_empty() {
        return vec![];
    }
    let buf_lower = tab.input_buffer.to_lowercase();
    let mut suggestions: Vec<String> = tab
        .command_history
        .iter()
        .rev()
        .filter(|cmd| {
            let cl = cmd.to_lowercase();
            cl.starts_with(&buf_lower) && cl != buf_lower
        })
        .take(4)
        .cloned()
        .collect();
    // Custom alias triggers — shown first so users can discover them
    for trigger in alias_triggers {
        if suggestions.len() >= 8 {
            break;
        }
        let tl = trigger.to_lowercase();
        if tl.starts_with(&buf_lower)
            && trigger.as_str() != tab.input_buffer.as_str()
            && !suggestions.iter().any(|s| s == trigger)
        {
            suggestions.push(trigger.clone());
        }
    }
    for &builtin in BUILT_IN_SUGGESTIONS {
        if suggestions.len() >= 8 {
            break;
        }
        if builtin.to_lowercase().starts_with(&buf_lower)
            && builtin != tab.input_buffer.as_str()
            && !suggestions.iter().any(|s| s == builtin)
        {
            suggestions.push(builtin.to_string());
        }
    }
    suggestions
}

// ─── Built-in suggestions for autocomplete ────────────────────────────────
const BUILT_IN_SUGGESTIONS: &[&str] = &[
    "nano", "vim", "vi", "nvim", "emacs",
    "ls", "la", "ll", "cd", "cp", "mv", "rm", "mkdir", "touch", "cat", "less",
    "head", "tail", "grep", "find", "chmod", "chown", "ln", "stat", "wc", "sort", "uniq",
    "ps", "top", "htop", "kill", "killall", "jobs", "bg", "fg", "nohup",
    "ssh", "scp", "rsync", "curl", "wget", "ping", "netstat", "ss", "ip",
    "apt", "apt-get", "yum", "dnf", "pacman", "pip", "pip3", "npm", "yarn", "cargo",
    "git", "docker", "kubectl", "systemctl", "service", "journalctl",
    "df", "du", "free", "uptime", "who", "whoami", "uname", "hostname",
    "tar", "zip", "unzip", "gzip", "gunzip", "xz",
    "python", "python3", "node", "ruby", "php", "bash", "sh", "zsh", "fish",
    "sudo", "su", "exit", "logout", "clear", "history", "env", "export", "echo",
    "source", "which", "whereis", "man", "alias", "unset", "set",
    "mysql", "psql", "redis-cli", "mongo",
];

// ─── Search highlight ──────────────────────────────────────────────────────
fn apply_search_highlight(
    spans: Vec<iced::widget::text::Span<'static, Message>>,
    query: &str,
    highlight_color: iced::Color,
    default_color: iced::Color,
) -> (Vec<iced::widget::text::Span<'static, Message>>, usize) {
    if query.is_empty() {
        return (spans, 0);
    }
    let ql = query.to_lowercase();
    let mut result = Vec::new();
    let mut count = 0;

    for span in spans {
        let text = span.text.as_ref().to_string();
        let tl = text.to_lowercase();
        let base_color = span.color.unwrap_or(default_color);

        if !tl.contains(ql.as_str()) {
            result.push(span);
            continue;
        }

        let mut pos = 0;
        while pos < text.len() {
            match tl[pos..].find(ql.as_str()) {
                Some(rel) => {
                    let abs = pos + rel;
                    let end = abs + ql.len();
                    // Safety: only slice on valid char boundaries
                    if !text.is_char_boundary(abs) || !text.is_char_boundary(end) || end > text.len() {
                        result.push(iced::widget::text::Span::new(text[pos..].to_string()).color(base_color));
                        break;
                    }
                    if abs > pos {
                        result.push(iced::widget::text::Span::new(text[pos..abs].to_string()).color(base_color));
                    }
                    result.push(iced::widget::text::Span::new(text[abs..end].to_string()).color(highlight_color));
                    count += 1;
                    pos = end;
                }
                None => {
                    if pos < text.len() {
                        result.push(iced::widget::text::Span::new(text[pos..].to_string()).color(base_color));
                    }
                    break;
                }
            }
        }
    }

    (result, count)
}

fn terminal_action_button(
    label: &'static str,
    msg: Message,
    p: theme::Palette,
) -> iced::widget::Button<'static, Message> {
    button(text(label).size(10).color(p.text_secondary))
        .on_press(msg)
        .padding([2, 8])
        .style(move |_t: &iced::Theme, status: button::Status| button::Style {
            background: Some(iced::Background::Color(match status {
                button::Status::Hovered => p.bg_hover,
                _ => iced::Color::TRANSPARENT,
            })),
            text_color: match status {
                button::Status::Hovered => p.text_primary,
                _ => p.text_secondary,
            },
            border: iced::Border {
                radius: theme::CORNER_RADIUS.into(),
                ..Default::default()
            },
            ..Default::default()
        })
}

fn normalized_screen(screen: &str) -> String {
    let mut out = String::new();
    for (line_idx, line) in screen.lines().enumerate() {
        if line_idx > 0 {
            out.push('\n');
        }
        out.push_str(line);
    }
    out
}

fn hidden_scrollbar_style(theme: &iced::Theme, status: scrollable::Status) -> scrollable::Style {
    let mut style = scrollable::default(theme, status);
    let invisible_rail = scrollable::Rail {
        background: None,
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        scroller: scrollable::Scroller {
            color: iced::Color::TRANSPARENT,
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
        },
    };
    style.vertical_rail = invisible_rail;
    style.horizontal_rail = invisible_rail;
    style.gap = None;
    style
}

fn fetch_remote_structure(host: &Host) -> Vec<String> {
    let mut structure: Vec<String> = Vec::new();

    let tcp = match TcpStream::connect(format!("{}:{}", host.hostname, host.port)) {
        Ok(tcp) => tcp,
        Err(err) => return vec![format!("FTP connection failed: {}", err)],
    };

    let mut sess = match ssh2::Session::new() {
        Ok(s) => s,
        Err(err) => return vec![format!("FTP session error: {}", err)],
    };
    sess.set_tcp_stream(tcp);
    if let Err(err) = sess.handshake() {
        return vec![format!("FTP handshake failed: {}", err)];
    }

    let mut authenticated = false;
    if sess.userauth_agent(&host.username).is_ok() {
        authenticated = true;
    } else if let Some(ref pwd) = host.password {
        if sess.userauth_password(&host.username, pwd).is_ok() {
            authenticated = true;
        }
    }

    if !authenticated {
        return vec!["FTP auth failed".to_string()];
    }

    let mut channel = match sess.channel_session() {
        Ok(ch) => ch,
        Err(err) => return vec![format!("FTP channel failed: {}", err)],
    };

    if let Err(err) = channel.exec("pwd && ls -1p 2>/dev/null | head -n 80") {
        return vec![format!("FTP structure command failed: {}", err)];
    }

    let mut output = String::new();
    if channel.read_to_string(&mut output).is_err() {
        return vec!["FTP structure read failed".to_string()];
    }

    let mut lines = output.lines();
    if let Some(root) = lines.next() {
        structure.push(format!("Root: {}", root.trim()));
    }
    for line in lines {
        let entry = line.trim();
        if entry.is_empty() {
            continue;
        }
        if entry.ends_with('/') {
            structure.push(format!("[D] {}", entry.trim_end_matches('/')));
        } else {
            structure.push(format!("[F] {}", entry));
        }
    }

    if structure.is_empty() {
        structure.push("No structure data".to_string());
    }

    structure
}

fn spawn_reader_thread<R>(mut reader: R, tx: mpsc::Sender<Vec<u8>>)
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
}

impl Drop for App {
    fn drop(&mut self) {
        for (_, mut runtime) in self.terminal_runtime.drain() {
            let _ = runtime.child.kill();
            let _ = runtime.child.wait();
        }
    }
}

fn collect_system_info(sys: &System, disks: &Disks) -> LocalSystemInfo {
    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let cpu_count = sys.cpus().len();
    let memory_used_mb = sys.used_memory() / 1024 / 1024;
    let memory_total_mb = sys.total_memory() / 1024 / 1024;
    let memory_usage = if memory_total_mb > 0 {
        (memory_used_mb as f32 / memory_total_mb as f32) * 100.0
    } else {
        0.0
    };

    let mut disk_used: u64 = 0;
    let mut disk_total: u64 = 0;
    for disk in disks.list() {
        disk_total += disk.total_space();
        disk_used += disk.total_space() - disk.available_space();
    }
    let disk_used_gb = disk_used as f64 / 1_073_741_824.0;
    let disk_total_gb = disk_total as f64 / 1_073_741_824.0;
    let disk_usage_percent = if disk_total > 0 {
        (disk_used as f32 / disk_total as f32) * 100.0
    } else {
        0.0
    };

    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
    let uptime_secs = System::uptime();

    LocalSystemInfo {
        cpu_usage,
        cpu_count,
        memory_used_mb,
        memory_total_mb,
        memory_usage,
        disk_used_gb,
        disk_total_gb,
        disk_usage_percent,
        os_name,
        hostname,
        uptime_secs,
    }
}
