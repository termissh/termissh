use anyhow::{Context, Result};
use crossterm::{
    cursor::{MoveTo, RestorePosition, SavePosition},
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Color as CColor, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, size},
};
use directories::ProjectDirs;
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fs,
    io::{self, ErrorKind, Read, Write},
    net::TcpStream,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use sysinfo::{System, Disks};

// --- VERİ YAPILARI ---

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Host {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    id: Option<String>,
    alias: String,
    hostname: String,
    port: u16,
    username: String,
    password: Option<String>,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            id: None,
            alias: String::new(),
            hostname: String::new(),
            port: 22,
            username: String::new(),
            password: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
enum Language {
    Turkish,
    English,
}

impl Default for Language {
    fn default() -> Self {
        Language::Turkish
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct AppConfig {
    hosts: Vec<Host>,
    api_key: Option<String>,
    language: Language,
}

// MACRO TANIMLARI
lazy_static::lazy_static! {
    static ref QUICK_COMMANDS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert(":ls", "ls -la");
        m.insert(":p", "pwd");
        m.insert(":top", "htop");
        m.insert(":u", "uptime");
        m.insert(":d", "docker ps");
        m.insert(":dc", "docker compose up -d");
        m.insert(":exit", "exit");
        m
    };
}

struct Texts {
    title: &'static str,
    help: &'static str,
    new_server: &'static str,
    edit_server: &'static str,
    api_key_settings: &'static str,
    command_history: &'static str,
    system_monitor: &'static str,
    connection: &'static str,
    waiting: &'static str,
    no_commands: &'static str,
    quick_commands: &'static str,
    search_placeholder: &'static str,
    info: &'static str,
    ssh_exit: &'static str,
    save_hint: &'static str,
}

impl Texts {
    fn get(lang: Language) -> Self {
        match lang {
            Language::Turkish => Texts {
                title: " SSH Sunucularim",
                help: " Sec: ↑↓ | Baglan: Enter | Yeni: n | Duzenle: e | Sil: d | Ara: / | Ping: p | API: s | Dil: l | q",
                new_server: " Yeni Sunucu Ekle ",
                edit_server: " Sunucu Duzenle ",
                api_key_settings: " API Key Ayarlari ",
                command_history: "Komutlar",
                system_monitor: " Sistem Monitoru ",
                connection: " Baglanti ",
                waiting: "Baglanti bekleniyor...",
                no_commands: "Henuz komut yok",
                quick_commands: " Hizli Komutlar ",
                search_placeholder: "Ara: ",
                info: " Bilgi ",
                ssh_exit: " SSH Terminal (ESC: cikis) ",
                save_hint: "[Enter: kaydet | Esc: iptal]",
            },
            Language::English => Texts {
                title: " My SSH Servers",
                help: " Sel: ↑↓ | Connect: Enter | New: n | Edit: e | Del: d | Search: / | Ping: p | API: s | Lang: l | q",
                new_server: " Add New Server ",
                edit_server: " Edit Server ",
                api_key_settings: " API Key Settings ",
                command_history: "History",
                system_monitor: " System Monitor ",
                connection: " Connection ",
                waiting: "Waiting for connection...",
                no_commands: "No commands yet",
                quick_commands: " Quick Commands ",
                search_placeholder: "Search: ",
                info: " Info ",
                ssh_exit: " SSH Terminal (ESC: exit) ",
                save_hint: "[Enter: save | Esc: cancel]",
            },
        }
    }
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Editing,
    ApiKeySettings,
}

#[derive(PartialEq, Clone, Copy)]
enum EditField {
    Alias,
    Hostname,
    Port,
    Username,
    Password,
    ApiKey,
}

#[derive(Clone)]
struct SessionInfo {
    username: String,
    hostname: String,
    command_history: Vec<String>,
}

impl Default for SessionInfo {
    fn default() -> Self {
        Self {
            username: String::new(),
            hostname: String::new(),
            command_history: Vec::new(),
        }
    }
}

#[derive(Clone, Default)]
struct LocalSystemInfo {
    cpu_usage: f32,
    memory_usage: f32,
    memory_used_mb: u64,
    memory_total_mb: u64,
    disk_usage_percent: f32,
    disk_used_gb: f64,
    disk_total_gb: f64,
    os_name: String,
    hostname: String,
    uptime_secs: u64,
    cpu_count: usize,
}

struct App {
    config: AppConfig,
    state: ListState,
    input_mode: InputMode,
    editing_index: Option<usize>,
    edit_field: EditField,
    // Host input buffers
    input_alias: String,
    input_hostname: String,
    input_port: String,
    input_username: String,
    input_password: String,
    // API config
    api_url: String,
    input_api_key: String,
    // Error display
    error_message: Option<String>,
    // Sidebar
    session_info: Option<SessionInfo>,
    // Local system info
    local_system_info: Arc<Mutex<LocalSystemInfo>>,
    // Search
    search_mode: bool,
    search_query: String,
    // Ping
    ping_results: HashMap<usize, Option<u128>>,
    ping_results_arc: Option<Arc<Mutex<HashMap<usize, Option<u128>>>>>,
}

impl App {
    fn new(config: AppConfig, api_url: String, local_info: Arc<Mutex<LocalSystemInfo>>) -> App {
        let mut state = ListState::default();
        if !config.hosts.is_empty() {
            state.select(Some(0));
        }
        let input_api_key = config.api_key.clone().unwrap_or_default();
        App {
            config,
            state,
            input_mode: InputMode::Normal,
            editing_index: None,
            edit_field: EditField::Alias,
            input_alias: String::new(),
            input_hostname: String::new(),
            input_port: String::new(),
            input_username: String::new(),
            input_password: String::new(),
            api_url,
            input_api_key,
            error_message: None,
            session_info: None,
            local_system_info: local_info,
            search_mode: false,
            search_query: String::new(),
            ping_results: HashMap::new(),
            ping_results_arc: None,
        }
    }

    fn visible_hosts(&self) -> Vec<(usize, &Host)> {
        if self.search_mode && !self.search_query.is_empty() {
            let q = self.search_query.to_lowercase();
            self.config.hosts.iter().enumerate()
                .filter(|(_, h)| {
                    h.alias.to_lowercase().contains(&q) ||
                    h.hostname.to_lowercase().contains(&q) ||
                    h.username.to_lowercase().contains(&q)
                })
                .collect()
        } else {
            self.config.hosts.iter().enumerate().collect()
        }
    }

    fn next(&mut self) {
        let visible = self.visible_hosts();
        if visible.is_empty() { return; }
        let current = self.state.selected().unwrap_or(0);
        let current_pos = visible.iter().position(|(i, _)| *i == current).unwrap_or(0);
        let next_pos = if current_pos >= visible.len() - 1 { 0 } else { current_pos + 1 };
        self.state.select(Some(visible[next_pos].0));
    }

    fn previous(&mut self) {
        let visible = self.visible_hosts();
        if visible.is_empty() { return; }
        let current = self.state.selected().unwrap_or(0);
        let current_pos = visible.iter().position(|(i, _)| *i == current).unwrap_or(0);
        let prev_pos = if current_pos == 0 { visible.len() - 1 } else { current_pos - 1 };
        self.state.select(Some(visible[prev_pos].0));
    }

    fn save_config(&self) -> Result<()> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "rust_ssh", "manager") {
            let config_dir = proj_dirs.config_dir();
            if !config_dir.exists() { fs::create_dir_all(config_dir)?; }
            let data = serde_json::to_string_pretty(&self.config)?;
            fs::write(config_dir.join("config.json"), data)?;
        }
        Ok(())
    }

    fn update_ping_results(&mut self) {
        if let Some(ref arc) = self.ping_results_arc {
            if let Ok(results) = arc.lock() {
                for (k, v) in results.iter() {
                    self.ping_results.insert(*k, *v);
                }
            }
        }
    }
}

// --- YARDIMCI FONKSİYONLAR ---

fn gauge_bar(label: &str, percent: f32, width: u16) -> Line<'static> {
    let bar_width = (width as usize).saturating_sub(label.len() + 8);
    if bar_width == 0 {
        return Line::from(format!("{}{:.0}%", label, percent));
    }
    let filled = ((percent / 100.0) * bar_width as f32).round() as usize;
    let empty = bar_width.saturating_sub(filled);

    let color = if percent < 50.0 {
        Color::Green
    } else if percent < 80.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    let bar_filled: String = "\u{2593}".repeat(filled);
    let bar_empty: String = "\u{2591}".repeat(empty);

    Line::from(vec![
        Span::styled(label.to_string(), Style::default().fg(Color::Cyan)),
        Span::styled(bar_filled, Style::default().fg(color)),
        Span::styled(bar_empty, Style::default().fg(Color::DarkGray)),
        Span::styled(format!(" {:5.1}%", percent), Style::default().fg(color).add_modifier(Modifier::BOLD)),
    ])
}

fn format_bytes_mb(mb: u64) -> String {
    if mb >= 1024 {
        format!("{:.1} GB", mb as f64 / 1024.0)
    } else {
        format!("{} MB", mb)
    }
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

// --- API FONKSİYONLARI ---

fn fetch_from_api(api_url: &str, api_key: &str) -> Result<Vec<Host>> {
    let url = format!("{}/api/cli/ssh", api_url);

    let response = ureq::get(&url)
        .set("Authorization", &format!("Bearer {}", api_key))
        .call()
        .map_err(|e| anyhow::anyhow!("API request failed: {}", e))?;

    let body: serde_json::Value = response.into_json()
        .context("Failed to parse API response")?;

    let connections = body["connections"].as_array()
        .ok_or_else(|| anyhow::anyhow!("Invalid API response"))?;

    let hosts: Vec<Host> = connections.iter().map(|c| Host {
        id: c["id"].as_str().map(String::from),
        alias: c["name"].as_str().unwrap_or("").to_string(),
        hostname: c["host"].as_str().unwrap_or("").to_string(),
        port: c["port"].as_u64().unwrap_or(22) as u16,
        username: c["username"].as_str().unwrap_or("").to_string(),
        password: c["password"].as_str().filter(|s| !s.is_empty()).map(String::from),
    }).collect();

    Ok(hosts)
}

fn create_on_api(api_url: &str, api_key: &str, host: &Host) -> Result<String> {
    let url = format!("{}/api/cli/ssh", api_url);
    let body = serde_json::json!({
        "name": host.alias,
        "host": host.hostname,
        "port": host.port,
        "username": host.username,
        "password": host.password,
    });

    let response = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| anyhow::anyhow!("API request failed: {}", e))?;

    let result: serde_json::Value = response.into_json()
        .context("Failed to parse API response")?;

    let id = result["id"].as_str()
        .ok_or_else(|| anyhow::anyhow!("No ID in API response"))?
        .to_string();

    Ok(id)
}

fn update_on_api(api_url: &str, api_key: &str, host: &Host) -> Result<()> {
    let id = host.id.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Host has no API ID"))?;
    let url = format!("{}/api/cli/ssh/{}", api_url, id);
    let body = serde_json::json!({
        "name": host.alias,
        "host": host.hostname,
        "port": host.port,
        "username": host.username,
        "password": host.password,
    });

    ureq::put(&url)
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| anyhow::anyhow!("API request failed: {}", e))?;

    Ok(())
}

fn delete_on_api(api_url: &str, api_key: &str, id: &str) -> Result<()> {
    let url = format!("{}/api/cli/ssh/{}", api_url, id);
    ureq::delete(&url)
        .set("Authorization", &format!("Bearer {}", api_key))
        .call()
        .map_err(|e| anyhow::anyhow!("API request failed: {}", e))?;
    Ok(())
}

// --- ANA FONKSİYONLAR ---

fn main() {
    dotenv::dotenv().ok();

    if let Err(e) = main_loop() {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        eprintln!("Critical Error: {:?}", e);
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}

fn main_loop() -> Result<()> {
    let mut config = load_config().unwrap_or_default();

    let api_url = env::var("API_URL").unwrap_or_else(|_| "https://termissh.org".to_string());

    if let Some(key) = &config.api_key.clone() {
        match fetch_from_api(&api_url, key) {
            Ok(hosts) => config.hosts = hosts,
            Err(_) => {}
        }
    }

    // Local system info background thread
    let local_info = Arc::new(Mutex::new(LocalSystemInfo::default()));
    let local_info_clone = local_info.clone();
    let local_info_running = Arc::new(AtomicBool::new(true));
    let local_info_running_clone = local_info_running.clone();

    let local_sys_thread = thread::spawn(move || {
        let mut sys = System::new();
        let mut disks = Disks::new_with_refreshed_list();

        // First CPU refresh always returns 0, need a second call after delay
        sys.refresh_all();
        thread::sleep(Duration::from_millis(500));

        while local_info_running_clone.load(Ordering::Relaxed) {
            sys.refresh_all();
            disks.refresh();

            let cpu_usage = if !sys.cpus().is_empty() {
                sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32
            } else {
                0.0
            };

            let mem_total = sys.total_memory();
            let mem_used = sys.used_memory();
            let mem_percent = if mem_total > 0 {
                (mem_used as f64 / mem_total as f64 * 100.0) as f32
            } else {
                0.0
            };

            let (disk_total, disk_used) = disks.list().iter().fold((0u64, 0u64), |(t, u), d| {
                (t + d.total_space(), u + (d.total_space() - d.available_space()))
            });
            let disk_percent = if disk_total > 0 {
                (disk_used as f64 / disk_total as f64 * 100.0) as f32
            } else {
                0.0
            };

            if let Ok(mut info) = local_info_clone.lock() {
                info.cpu_usage = cpu_usage;
                info.memory_usage = mem_percent;
                info.memory_used_mb = mem_used / 1024 / 1024;
                info.memory_total_mb = mem_total / 1024 / 1024;
                info.disk_usage_percent = disk_percent;
                info.disk_used_gb = disk_used as f64 / 1024.0 / 1024.0 / 1024.0;
                info.disk_total_gb = disk_total as f64 / 1024.0 / 1024.0 / 1024.0;
                info.os_name = System::name().unwrap_or_default();
                info.hostname = System::host_name().unwrap_or_default();
                info.uptime_secs = System::uptime();
                info.cpu_count = sys.cpus().len();
            }

            thread::sleep(Duration::from_secs(2));
        }
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config, api_url, local_info);
    let res = run_app(&mut terminal, &mut app);

    local_info_running.store(false, Ordering::Relaxed);
    let _ = local_sys_thread.join();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    res
}

fn load_config() -> Result<AppConfig> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "rust_ssh", "manager") {
        let path = proj_dirs.config_dir().join("config.json");
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: AppConfig = serde_json::from_str(&content).unwrap_or_default();
            return Ok(config);
        }
    }
    Ok(AppConfig::default())
}

// --- TUI LOGIC ---

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        // Update ping results from background thread
        app.update_ping_results();

        terminal.draw(|f| ui(f, app))?;

        // Non-blocking poll so sidebar gauges refresh even without input
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // Search mode input handling
            if app.search_mode {
                match key.code {
                    KeyCode::Esc => {
                        app.search_mode = false;
                        app.search_query.clear();
                    }
                    KeyCode::Enter => {
                        app.search_mode = false;
                    }
                    KeyCode::Char(c) => {
                        app.search_query.push(c);
                        let visible = app.visible_hosts();
                        if let Some((idx, _)) = visible.first() {
                            app.state.select(Some(*idx));
                        }
                    }
                    KeyCode::Backspace => {
                        app.search_query.pop();
                        if app.search_query.is_empty() {
                            app.search_mode = false;
                        } else {
                            let visible = app.visible_hosts();
                            if let Some((idx, _)) = visible.first() {
                                app.state.select(Some(*idx));
                            }
                        }
                    }
                    _ => {}
                }
                continue;
            }

            match app.input_mode {
                InputMode::Normal => {
                    app.error_message = None;
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        KeyCode::Char('l') => {
                            app.config.language = match app.config.language {
                                Language::Turkish => Language::English,
                                Language::English => Language::Turkish,
                            };
                            let _ = app.save_config();
                        },

                        // Search mode
                        KeyCode::Char('/') => {
                            app.search_mode = true;
                            app.search_query.clear();
                        },

                        // Ping all servers
                        KeyCode::Char('p') => {
                            let hosts: Vec<(usize, String, u16)> = app.config.hosts.iter().enumerate()
                                .map(|(i, h)| (i, h.hostname.clone(), h.port))
                                .collect();
                            let ping_results = Arc::new(Mutex::new(HashMap::new()));
                            let results_clone = ping_results.clone();

                            thread::spawn(move || {
                                for (idx, hostname, port) in hosts {
                                    let addr = format!("{}:{}", hostname, port);
                                    if let Ok(addr) = addr.parse() {
                                        let start = std::time::Instant::now();
                                        let reachable = TcpStream::connect_timeout(
                                            &addr,
                                            Duration::from_secs(3)
                                        ).is_ok();
                                        let elapsed = start.elapsed().as_millis();
                                        if let Ok(mut results) = results_clone.lock() {
                                            results.insert(idx, if reachable { Some(elapsed) } else { None });
                                        }
                                    }
                                }
                            });

                            app.ping_results_arc = Some(ping_results);
                            app.ping_results.clear();
                        },

                        // New server
                        KeyCode::Char('n') => {
                            app.input_mode = InputMode::Editing;
                            app.editing_index = None;
                            app.edit_field = EditField::Alias;
                            app.input_alias.clear();
                            app.input_hostname.clear();
                            app.input_port = "22".to_string();
                            app.input_username.clear();
                            app.input_password.clear();
                        },

                        // Edit
                        KeyCode::Char('e') => {
                            if let Some(selected) = app.state.selected() {
                                if selected < app.config.hosts.len() {
                                    let host = &app.config.hosts[selected];
                                    app.input_alias = host.alias.clone();
                                    app.input_hostname = host.hostname.clone();
                                    app.input_port = host.port.to_string();
                                    app.input_username = host.username.clone();
                                    app.input_password = host.password.clone().unwrap_or_default();
                                    app.editing_index = Some(selected);
                                    app.input_mode = InputMode::Editing;
                                    app.edit_field = EditField::Alias;
                                }
                            }
                        },

                        // Delete
                        KeyCode::Char('d') => {
                            if let Some(selected) = app.state.selected() {
                                if selected < app.config.hosts.len() {
                                    if let Some(key) = &app.config.api_key.clone() {
                                        if let Some(ref id) = app.config.hosts[selected].id.clone() {
                                            if let Err(e) = delete_on_api(&app.api_url, key, id) {
                                                app.error_message = Some(format!("API delete error: {}", e));
                                            }
                                        }
                                    }
                                    app.config.hosts.remove(selected);
                                    let _ = app.save_config();
                                    if app.config.hosts.is_empty() { app.state.select(None); }
                                    else if selected >= app.config.hosts.len() { app.state.select(Some(selected - 1)); }
                                }
                            }
                        },

                        // API Key settings
                        KeyCode::Char('s') => {
                            app.input_mode = InputMode::ApiKeySettings;
                            app.input_api_key = app.config.api_key.clone().unwrap_or_default();
                        },

                        // Connect
                        KeyCode::Enter => {
                            if let Some(selected) = app.state.selected() {
                                if selected < app.config.hosts.len() {
                                    let host = app.config.hosts[selected].clone();

                                    // TUI'dan çık ve normal terminal moduna geç
                                    disable_raw_mode()?;
                                    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                                    terminal.show_cursor()?;

                                    println!("\nConnecting to {}@{}:{}...\n", host.username, host.hostname, host.port);

                                    if let Err(e) = start_ssh_session_interactive(&host) {
                                        eprintln!("\nSSH Error: {}", e);
                                        println!("\nPress Enter to return to menu...");
                                        let mut s = String::new();
                                        io::stdin().read_line(&mut s)?;
                                    }

                                    // TUI'ya geri dön
                                    enable_raw_mode()?;
                                    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                                    terminal.hide_cursor()?;
                                    terminal.clear()?;
                                }
                            }
                        }
                        _ => {}
                    }
                },

                InputMode::Editing => match key.code {
                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                    KeyCode::Tab => {
                        app.edit_field = match app.edit_field {
                            EditField::Alias => EditField::Hostname,
                            EditField::Hostname => EditField::Port,
                            EditField::Port => EditField::Username,
                            EditField::Username => EditField::Password,
                            EditField::Password => EditField::Alias,
                            _ => EditField::Alias,
                        }
                    }
                    KeyCode::Enter => {
                        if !app.input_alias.is_empty() && !app.input_hostname.is_empty() {
                            let port = app.input_port.parse::<u16>().unwrap_or(22);
                            let password = if app.input_password.is_empty() { None } else { Some(app.input_password.clone()) };

                            match app.editing_index {
                                Some(index) => {
                                    let updated_host = Host {
                                        id: app.config.hosts[index].id.clone(),
                                        alias: app.input_alias.clone(),
                                        hostname: app.input_hostname.clone(),
                                        port,
                                        username: app.input_username.clone(),
                                        password,
                                    };

                                    if let Some(key) = &app.config.api_key.clone() {
                                        if let Err(e) = update_on_api(&app.api_url, key, &updated_host) {
                                            app.error_message = Some(format!("API update error: {}", e));
                                        }
                                    }

                                    app.config.hosts[index] = updated_host;
                                }
                                None => {
                                    let mut new_host = Host {
                                        id: None,
                                        alias: app.input_alias.clone(),
                                        hostname: app.input_hostname.clone(),
                                        port,
                                        username: app.input_username.clone(),
                                        password,
                                    };

                                    if let Some(key) = &app.config.api_key.clone() {
                                        match create_on_api(&app.api_url, key, &new_host) {
                                            Ok(id) => new_host.id = Some(id),
                                            Err(e) => app.error_message = Some(format!("API create error: {}", e)),
                                        }
                                    }

                                    app.config.hosts.push(new_host);
                                    app.state.select(Some(app.config.hosts.len() - 1));
                                }
                            }

                            let _ = app.save_config();
                            app.input_mode = InputMode::Normal;
                        }
                    }
                    KeyCode::Char(c) => match app.edit_field {
                        EditField::Alias => app.input_alias.push(c),
                        EditField::Hostname => app.input_hostname.push(c),
                        EditField::Port => { if c.is_numeric() { app.input_port.push(c); } },
                        EditField::Username => app.input_username.push(c),
                        EditField::Password => app.input_password.push(c),
                        _ => {}
                    },
                    KeyCode::Backspace => match app.edit_field {
                        EditField::Alias => { app.input_alias.pop(); },
                        EditField::Hostname => { app.input_hostname.pop(); },
                        EditField::Port => { app.input_port.pop(); },
                        EditField::Username => { app.input_username.pop(); },
                        EditField::Password => { app.input_password.pop(); },
                        _ => {}
                    },
                    _ => {}
                },

                InputMode::ApiKeySettings => match key.code {
                    KeyCode::Esc => {
                        app.input_api_key = app.config.api_key.clone().unwrap_or_default();
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Enter => {
                        app.config.api_key = if app.input_api_key.is_empty() { None } else { Some(app.input_api_key.clone()) };
                        let _ = app.save_config();

                        if let Some(key) = &app.config.api_key.clone() {
                            match fetch_from_api(&app.api_url, key) {
                                Ok(hosts) => {
                                    app.config.hosts = hosts;
                                    if app.config.hosts.is_empty() {
                                        app.state.select(None);
                                    } else {
                                        app.state.select(Some(0));
                                    }
                                }
                                Err(e) => {
                                    app.error_message = Some(format!("API error: {}", e));
                                }
                            }
                        }

                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char(c) => {
                        app.input_api_key.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input_api_key.pop();
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let texts = Texts::get(app.config.language);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(f.size());

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(main_chunks[0]);

    // Build list items with visible hosts
    let visible = app.visible_hosts();
    let items: Vec<ListItem> = visible.iter().map(|(i, host)| {
        let dot = if host.id.is_some() {
            Span::styled(" ● ", Style::default().fg(Color::Green))
        } else {
            Span::styled(" ○ ", Style::default().fg(Color::Rgb(70, 70, 80)))
        };

        let mut spans = vec![
            dot,
            Span::styled(
                format!("{:<18}", host.alias),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}@{}:{}", host.username, host.hostname, host.port),
                Style::default().fg(Color::Rgb(100, 110, 130)),
            ),
        ];

        if let Some(ping) = app.ping_results.get(i) {
            let (text, color) = match ping {
                Some(ms) if *ms < 100 => (format!("  {}ms", ms), Color::Green),
                Some(ms) if *ms < 300 => (format!("  {}ms", ms), Color::Yellow),
                Some(ms)              => (format!("  {}ms", ms), Color::Red),
                None                  => ("  timeout".to_string(), Color::Red),
            };
            spans.push(Span::styled(text, Style::default().fg(color)));
        }

        ListItem::new(Line::from(spans))
    }).collect();

    let (sync_symbol, sync_color) = if app.config.api_key.is_some() {
        (" ●", Color::Green)
    } else {
        (" ○", Color::Rgb(70, 70, 80))
    };

    let host_count = format!(" ({}) ", app.config.hosts.len());

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(Line::from(vec![
                Span::styled(texts.title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(host_count, Style::default().fg(Color::Rgb(100, 100, 110))),
                Span::styled(sync_symbol, Style::default().fg(sync_color)),
            ]))
            .border_style(Style::default().fg(Color::Rgb(50, 50, 65))))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(25, 40, 65))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("   ");

    f.render_stateful_widget(list, left_chunks[0], &mut app.state);

    // Sidebar
    render_sidebar(f, app, main_chunks[1]);

    // Bottom bar
    match app.input_mode {
        InputMode::Normal => {
            if app.search_mode {
                let search_text = Line::from(vec![
                    Span::styled(texts.search_placeholder, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(&app.search_query),
                    Span::styled("_", Style::default().fg(Color::Yellow)),
                ]);
                let p = Paragraph::new(search_text)
                    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
                f.render_widget(p, left_chunks[1]);
            } else if let Some(ref err) = app.error_message {
                let p = Paragraph::new(format!(" ✗ {}", err))
                    .style(Style::default().fg(Color::Red))
                    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Red)));
                f.render_widget(p, left_chunks[1]);
            } else {
                let ks = Style::default().fg(Color::Cyan);
                let ds = Style::default().fg(Color::Rgb(90, 90, 100));
                let p = Paragraph::new(Line::from(vec![
                    Span::raw(" "),
                    Span::styled("[↑↓]", ks), Span::styled(" nav  ", ds),
                    Span::styled("[↵]", ks),  Span::styled(" connect  ", ds),
                    Span::styled("[n]", ks),  Span::styled(" new  ", ds),
                    Span::styled("[e]", ks),  Span::styled(" edit  ", ds),
                    Span::styled("[d]", ks),  Span::styled(" del  ", ds),
                    Span::styled("[/]", ks),  Span::styled(" search  ", ds),
                    Span::styled("[p]", ks),  Span::styled(" ping  ", ds),
                    Span::styled("[s]", ks),  Span::styled(" api  ", ds),
                    Span::styled("[l]", ks),  Span::styled(" lang  ", ds),
                    Span::styled("[q]", ks),  Span::styled(" quit", ds),
                ]));
                f.render_widget(p, left_chunks[1]);
            }
        }
        InputMode::Editing => {
            let title = if app.editing_index.is_some() { texts.edit_server } else { texts.new_server };

            let active_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
            let def_style = Style::default().fg(Color::Rgb(80, 80, 90));
            let val_style = Style::default().fg(Color::White);
            let get_style = |field: EditField| if app.edit_field == field { active_style } else { def_style };

            let pwd_masked: String = app.input_password.chars().map(|_| '•').collect();
            let cursor = if true { Span::styled("_", Style::default().fg(Color::Cyan)) } else { Span::raw("") };
            let _ = cursor;

            let mut spans = vec![
                Span::raw(" "),
                Span::styled("alias ", get_style(EditField::Alias)),
                Span::styled(app.input_alias.clone(), val_style), Span::raw("  "),
                Span::styled("host ", get_style(EditField::Hostname)),
                Span::styled(app.input_hostname.clone(), val_style), Span::raw("  "),
                Span::styled("port ", get_style(EditField::Port)),
                Span::styled(app.input_port.clone(), val_style), Span::raw("  "),
                Span::styled("user ", get_style(EditField::Username)),
                Span::styled(app.input_username.clone(), val_style), Span::raw("  "),
                Span::styled("pass ", get_style(EditField::Password)),
                Span::styled(pwd_masked, val_style),
            ];
            // cursor on active field
            if app.edit_field == EditField::Alias || app.edit_field == EditField::Hostname
                || app.edit_field == EditField::Port || app.edit_field == EditField::Username
                || app.edit_field == EditField::Password {
                spans.push(Span::styled("▌", Style::default().fg(Color::Cyan)));
            }

            let p = Paragraph::new(Line::from(spans))
                .block(Block::default().borders(Borders::ALL)
                    .title(Line::from(vec![
                        Span::styled(title, Style::default().fg(Color::Cyan)),
                        Span::styled("  [Tab] next  [↵] save  [Esc] cancel", Style::default().fg(Color::Rgb(80,80,90))),
                    ]))
                    .border_style(Style::default().fg(Color::Cyan)));
            f.render_widget(p, left_chunks[1]);
        }
        InputMode::ApiKeySettings => {
            let texts = Texts::get(app.config.language);
            let key_masked: String = if app.input_api_key.len() > 8 {
                format!("{}...{}", &app.input_api_key[..4], &app.input_api_key[app.input_api_key.len()-4..])
            } else {
                app.input_api_key.chars().map(|_| '•').collect()
            };

            let p = Paragraph::new(Line::from(vec![
                Span::raw(" "),
                Span::styled("key ", Style::default().fg(Color::Cyan)),
                Span::styled(key_masked, Style::default().fg(Color::White)),
                Span::styled("▌  ", Style::default().fg(Color::Cyan)),
                Span::styled("url ", Style::default().fg(Color::Rgb(80,80,90))),
                Span::styled(app.api_url.clone(), Style::default().fg(Color::Rgb(100,100,120))),
            ]))
            .block(Block::default().borders(Borders::ALL)
                .title(Line::from(vec![
                    Span::styled(texts.api_key_settings, Style::default().fg(Color::Cyan)),
                    Span::styled("  [↵] save & sync  [Esc] cancel", Style::default().fg(Color::Rgb(80,80,90))),
                ]))
                .border_style(Style::default().fg(Color::Cyan)));
            f.render_widget(p, left_chunks[1]);
        }
    }
}

// --- SIDEBAR RENDERING ---

fn render_sidebar(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ref session) = app.session_info {
        render_remote_sidebar(f, app, session, area);
    } else {
        render_local_sidebar(f, app, area);
    }
}

fn render_local_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // System + Info merged
            Constraint::Length(6), // Quick commands compact
            Constraint::Length(3), // Status bar
        ])
        .split(area);

    let bar_width = area.width.saturating_sub(2);
    let muted = Style::default().fg(Color::Rgb(80, 80, 95));
    let label = Style::default().fg(Color::Rgb(120, 140, 170));

    if let Ok(info) = app.local_system_info.lock() {
        let lines = vec![
            Line::from(""),
            gauge_bar(" CPU ", info.cpu_usage, bar_width),
            gauge_bar(" RAM ", info.memory_usage, bar_width),
            Line::from(Span::styled(
                format!("      {} / {}", format_bytes_mb(info.memory_used_mb), format_bytes_mb(info.memory_total_mb)),
                muted,
            )),
            gauge_bar(" DSK ", info.disk_usage_percent, bar_width),
            Line::from(Span::styled(
                format!("      {:.1} / {:.1} GB", info.disk_used_gb, info.disk_total_gb),
                muted,
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(" os    ", label),
                Span::styled(info.os_name.clone(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled(" host  ", label),
                Span::styled(info.hostname.clone(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled(" up    ", label),
                Span::styled(format_uptime(info.uptime_secs), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled(" cpu   ", label),
                Span::styled(format!("{} cores", info.cpu_count), Style::default().fg(Color::White)),
            ]),
        ];
        let sys_widget = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(" system ", Style::default().fg(Color::Rgb(120, 140, 170))))
                .border_style(Style::default().fg(Color::Rgb(50, 50, 65))));
        f.render_widget(sys_widget, sidebar_chunks[0]);
    } else {
        let empty = Paragraph::new(Line::from(Span::styled(
            " loading...",
            Style::default().fg(Color::Rgb(70, 70, 80)),
        )))
        .block(Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" system ", Style::default().fg(Color::Rgb(120, 140, 170))))
            .border_style(Style::default().fg(Color::Rgb(50, 50, 65))));
        f.render_widget(empty, sidebar_chunks[0]);
    }

    // Compact quick commands (2 columns)
    let mc = Style::default().fg(Color::Rgb(80, 160, 120));
    let dc = Style::default().fg(Color::Rgb(80, 80, 95));
    let hints = vec![
        Line::from(vec![
            Span::styled(" :ls ", mc), Span::styled("ls -la   ", dc),
            Span::styled(":top ", mc), Span::styled("htop", dc),
        ]),
        Line::from(vec![
            Span::styled(" :d  ", mc), Span::styled("docker ps", dc),
            Span::styled("  :dc", mc), Span::styled(" up -d", dc),
        ]),
        Line::from(vec![
            Span::styled(" :u  ", mc), Span::styled("uptime   ", dc),
            Span::styled(":p   ", mc), Span::styled("pwd", dc),
        ]),
        Line::from(vec![
            Span::styled(" :exit ", mc), Span::styled("exit", dc),
        ]),
    ];
    let hints_widget = Paragraph::new(hints)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" macros ", Style::default().fg(Color::Rgb(80, 160, 120))))
            .border_style(Style::default().fg(Color::Rgb(50, 50, 65))));
    f.render_widget(hints_widget, sidebar_chunks[1]);

    // Status bar
    let (api_sym, api_col) = if app.config.api_key.is_some() {
        ("● sync", Color::Green)
    } else {
        ("○ local", Color::Rgb(70, 70, 80))
    };
    let status = Paragraph::new(Line::from(vec![
        Span::styled(" TermiSSH ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("v0.2.0  ", Style::default().fg(Color::Rgb(70, 70, 80))),
        Span::styled(api_sym, Style::default().fg(api_col)),
    ]))
    .block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(50, 50, 65))));
    f.render_widget(status, sidebar_chunks[2]);
}

fn render_remote_sidebar(f: &mut Frame, app: &App, info: &SessionInfo, area: Rect) {
    let texts = Texts::get(app.config.language);
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),   // Connection info
            Constraint::Min(0),      // Command history
            Constraint::Length(11),  // Quick commands
        ])
        .split(area);

    // Connection info
    let conn_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  \u{25CF} ", Style::default().fg(Color::Green)),
            Span::styled(
                format!("{}@{}", info.username, info.hostname),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    let conn_widget = Paragraph::new(conn_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(texts.connection)
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(Color::Green)));
    f.render_widget(conn_widget, sidebar_chunks[0]);

    // Command history
    let history_lines: Vec<Line> = if info.command_history.is_empty() {
        vec![Line::from(Span::styled(
            format!("  {}", texts.no_commands),
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        info.command_history.iter().rev().take(20).map(|c| {
            Line::from(vec![
                Span::styled("  $ ", Style::default().fg(Color::Yellow)),
                Span::styled(c.to_string(), Style::default().fg(Color::White)),
            ])
        }).collect()
    };
    let history_widget = Paragraph::new(history_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", texts.command_history))
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(history_widget, sidebar_chunks[1]);

    // Quick commands
    let hints = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  :ls  ", Style::default().fg(Color::Green)),
            Span::styled("ls -la", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  :top ", Style::default().fg(Color::Green)),
            Span::styled("htop", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  :d   ", Style::default().fg(Color::Green)),
            Span::styled("docker ps", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  :dc  ", Style::default().fg(Color::Green)),
            Span::styled("docker compose up", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  :u   ", Style::default().fg(Color::Green)),
            Span::styled("uptime", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  :p   ", Style::default().fg(Color::Green)),
            Span::styled("pwd", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  :exit", Style::default().fg(Color::Green)),
            Span::styled("exit", Style::default().fg(Color::DarkGray)),
        ]),
    ];
    let hints_widget = Paragraph::new(hints)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(texts.quick_commands)
            .title_style(Style::default().fg(Color::Cyan))
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(hints_widget, sidebar_chunks[2]);
}

// --- SSH LOGIC ---

fn start_ssh_session_in_tui(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App, host: &Host) -> Result<()> {
    let tcp = TcpStream::connect(format!("{}:{}", host.hostname, host.port))
        .context("TCP connection failed")?;
    let mut sess = ssh2::Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake().context("SSH handshake failed")?;

    // Authentication
    let mut authenticated = false;
    if sess.userauth_agent(&host.username).is_ok() {
        authenticated = true;
    } else if let Some(ref pwd) = host.password {
        if sess.userauth_password(&host.username, pwd).is_ok() {
            authenticated = true;
        }
    }

    if !authenticated {
        return Err(anyhow::anyhow!("Authentication failed"));
    }

    // Open SSH channel
    let mut channel = sess.channel_session()?;
    let (term_width, term_height) = size()?;
    let ssh_width = (term_width as f32 * 0.7) as u32;
    channel.request_pty("xterm-256color", None, Some((ssh_width, term_height as u32 - 2, 0, 0)))?;
    channel.shell()?;
    sess.set_blocking(false);

    // Keep session alive until function returns (channel depends on it)
    let _sess = sess;

    // Create session info
    app.session_info = Some(SessionInfo {
        username: host.username.clone(),
        hostname: host.hostname.clone(),
        ..Default::default()
    });

    let running = Arc::new(AtomicBool::new(true));
    let output_buffer = Arc::new(Mutex::new(Vec::<u8>::new()));

    // Output reading thread
    let running_clone = running.clone();
    let output_clone = output_buffer.clone();
    let channel_clone = Arc::new(Mutex::new(channel));
    let channel_read = channel_clone.clone();

    let read_thread = thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        while running_clone.load(Ordering::Relaxed) {
            let result = {
                let mut ch = channel_read.lock().unwrap();
                ch.read(&mut buffer)
            };
            match result {
                Ok(n) if n > 0 => {
                    let mut out = output_clone.lock().unwrap();
                    out.extend_from_slice(&buffer[..n]);
                    if out.len() > 50000 {
                        let drain_len = out.len() - 50000;
                        out.drain(0..drain_len);
                    }
                }
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => {
                    running_clone.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }
    });

    let mut input_buffer = String::new();

    // Main loop
    loop {
        terminal.draw(|f| {
            render_ssh_session(f, app, &output_buffer, &input_buffer);
        })?;

        if !running.load(Ordering::Relaxed) {
            break;
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Esc => {
                        running.store(false, Ordering::Relaxed);
                        break;
                    }
                    KeyCode::Char(c) => {
                        input_buffer.push(c);
                        let mut ch = channel_clone.lock().unwrap();
                        let _ = ch.write_all(&[c as u8]);
                    }
                    KeyCode::Enter => {
                        if !input_buffer.is_empty() {
                            if let Some(ref mut info) = app.session_info {
                                info.command_history.push(input_buffer.clone());
                                if info.command_history.len() > 50 {
                                    info.command_history.remove(0);
                                }
                            }
                            input_buffer.clear();
                        }
                        let mut ch = channel_clone.lock().unwrap();
                        let _ = ch.write_all(b"\r");
                    }
                    KeyCode::Backspace => {
                        if !input_buffer.is_empty() {
                            input_buffer.pop();
                        }
                        let mut ch = channel_clone.lock().unwrap();
                        let _ = ch.write_all(&[127u8]);
                    }
                    KeyCode::Tab => {
                        let mut ch = channel_clone.lock().unwrap();
                        let _ = ch.write_all(&[9u8]);
                    }
                    KeyCode::Up => {
                        let mut ch = channel_clone.lock().unwrap();
                        let _ = ch.write_all(b"\x1b[A");
                    }
                    KeyCode::Down => {
                        let mut ch = channel_clone.lock().unwrap();
                        let _ = ch.write_all(b"\x1b[B");
                    }
                    KeyCode::Right => {
                        let mut ch = channel_clone.lock().unwrap();
                        let _ = ch.write_all(b"\x1b[C");
                    }
                    KeyCode::Left => {
                        let mut ch = channel_clone.lock().unwrap();
                        let _ = ch.write_all(b"\x1b[D");
                    }
                    _ => {}
                }
            }
        }
    }

    running.store(false, Ordering::Relaxed);
    let _ = read_thread.join();
    app.session_info = None;

    Ok(())
}

fn render_ssh_session(f: &mut Frame, app: &App, output_buffer: &Arc<Mutex<Vec<u8>>>, _input_buffer: &str) {
    let texts = Texts::get(app.config.language);
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(f.size());

    // Terminal output - ham çıktıyı göster
    let output = output_buffer.lock().unwrap();
    let output_str = String::from_utf8_lossy(&*output);

    let lines: Vec<&str> = output_str.lines().collect();
    let visible_lines = (main_chunks[0].height as usize).saturating_sub(2);
    let start_line = lines.len().saturating_sub(visible_lines);
    let visible_text: Vec<Line> = lines[start_line..]
        .iter()
        .map(|l| Line::from(*l))
        .collect();

    let terminal_widget = Paragraph::new(visible_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(texts.ssh_exit)
            .title_style(Style::default().fg(Color::Red))
            .border_style(Style::default().fg(Color::DarkGray)))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });

    f.render_widget(terminal_widget, main_chunks[0]);

    // Sidebar
    render_sidebar(f, app, main_chunks[1]);
}

fn start_ssh_session_interactive(host: &Host) -> Result<()> {
    let tcp = TcpStream::connect(format!("{}:{}", host.hostname, host.port)).context("TCP Connection Failed")?;
    let mut sess = ssh2::Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake().context("SSH Handshake Failed")?;

    let mut authenticated = false;

    if sess.userauth_agent(&host.username).is_ok() {
        authenticated = true;
    } else if let Some(ref pwd) = host.password {
        if sess.userauth_password(&host.username, pwd).is_ok() {
            authenticated = true;
        }
    }

    if !authenticated {
        println!("Auth methods failed. Please enter password manually.");
        print!("Password for {}@{}: ", host.username, host.hostname);
        io::stdout().flush()?;

        let mut password = String::new();
        enable_raw_mode()?;
        loop {
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press { continue; }
                    match key.code {
                        KeyCode::Enter => break,
                        KeyCode::Char(c) => { password.push(c); print!("*"); io::stdout().flush()?; },
                        KeyCode::Backspace => { if !password.is_empty() { password.pop(); print!("\x08 \x08"); io::stdout().flush()?; } },
                        KeyCode::Esc => { disable_raw_mode()?; return Err(anyhow::anyhow!("Cancelled")); }
                        _ => {}
                    }
                }
            }
        }
        disable_raw_mode()?;
        println!();

        sess.userauth_password(&host.username, &password).context("Authentication failed")?;
    }

    let mut channel = sess.channel_session()?;
    channel.request_pty("xterm", None, Some((80, 24, 0, 0)))?;
    channel.shell()?;
    sess.set_blocking(false);

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let channel = Arc::new(Mutex::new(channel));
    let channel_read = channel.clone();

    let read_thread = thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        while running_clone.load(Ordering::Relaxed) {
            let result = {
                let mut ch = channel_read.lock().unwrap();
                ch.read(&mut buffer)
            };
            match result {
                Ok(n) if n > 0 => {
                    let _ = io::stdout().write_all(&buffer[..n]);
                    let _ = io::stdout().flush();
                }
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::WouldBlock => { thread::sleep(Duration::from_millis(10)); }
                Err(_) => { running_clone.store(false, Ordering::Relaxed); break; }
            }
        }
    });

    println!("\r\n=== Connected (Type : for macros like :p, :dc) ===\r\n");
    enable_raw_mode()?;

    let mut macro_buffer = String::new();
    let mut in_macro = false;

    loop {
        if !running.load(Ordering::Relaxed) { break; }

        if event::poll(Duration::from_millis(30))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press { continue; }

                match key.code {
                    KeyCode::Char(ch) => {
                        if ch == ':' && !in_macro {
                            in_macro = true;
                            macro_buffer.push(':');
                            draw_command_bar(&macro_buffer)?;
                        } else if in_macro {
                            macro_buffer.push(ch);
                            draw_command_bar(&macro_buffer)?;
                        } else {
                            let mut ch_lock = channel.lock().unwrap();
                            let _ = ch_lock.write_all(&[ch as u8]);
                        }
                    }
                    KeyCode::Enter => {
                        if in_macro {
                            clear_command_bar()?;
                            if let Some(cmd) = QUICK_COMMANDS.get(macro_buffer.as_str()) {
                                let mut ch_lock = channel.lock().unwrap();
                                let _ = ch_lock.write_all(cmd.as_bytes());
                                let _ = ch_lock.write_all(b"\n");
                            }
                            macro_buffer.clear();
                            in_macro = false;
                        } else {
                            let mut ch_lock = channel.lock().unwrap();
                            let _ = ch_lock.write_all(b"\r");
                        }
                    }
                    KeyCode::Backspace => {
                        if in_macro {
                            if !macro_buffer.is_empty() {
                                macro_buffer.pop();
                                if macro_buffer.is_empty() {
                                    in_macro = false;
                                    clear_command_bar()?;
                                } else {
                                    draw_command_bar(&macro_buffer)?;
                                }
                            }
                        } else {
                            let mut ch_lock = channel.lock().unwrap();
                            let _ = ch_lock.write_all(&[127u8]);
                        }
                    }
                    KeyCode::Esc => {
                        if in_macro {
                            in_macro = false;
                            macro_buffer.clear();
                            clear_command_bar()?;
                        } else {
                            let mut ch_lock = channel.lock().unwrap();
                            let _ = ch_lock.write_all(&[27u8]);
                        }
                    }
                    KeyCode::Tab => { let _ = channel.lock().unwrap().write_all(&[9u8]); }
                    KeyCode::Up => { let _ = channel.lock().unwrap().write_all(b"\x1b[A"); }
                    KeyCode::Down => { let _ = channel.lock().unwrap().write_all(b"\x1b[B"); }
                    KeyCode::Right => { let _ = channel.lock().unwrap().write_all(b"\x1b[C"); }
                    KeyCode::Left => { let _ = channel.lock().unwrap().write_all(b"\x1b[D"); }
                    _ => {}
                }
            }
        }

        if channel.lock().unwrap().eof() {
            running.store(false, Ordering::Relaxed);
            break;
        }
    }

    disable_raw_mode()?;
    running.store(false, Ordering::Relaxed);
    let _ = read_thread.join();
    Ok(())
}

// --- COMMAND BAR ---

fn draw_command_bar(text: &str) -> Result<()> {
    let (_cols, rows) = size()?;
    execute!(
        io::stdout(),
        SavePosition,
        MoveTo(0, rows - 1),
        SetBackgroundColor(CColor::Blue),
        SetForegroundColor(CColor::White),
        Clear(ClearType::CurrentLine),
        Print(format!("COMMAND MODE: {}", text)),
        SetBackgroundColor(CColor::Reset),
        SetForegroundColor(CColor::Reset),
        RestorePosition
    )?;
    io::stdout().flush()?;
    Ok(())
}

fn clear_command_bar() -> Result<()> {
    let (_cols, rows) = size()?;
    execute!(
        io::stdout(),
        SavePosition,
        MoveTo(0, rows - 1),
        Clear(ClearType::CurrentLine),
        RestorePosition
    )?;
    io::stdout().flush()?;
    Ok(())
}
