//! System Management Panel
//!
//! Provides firewall manager, package manager, login history, SSH key manager,
//! system settings editor, and an auto-detecting extension system (nginx, apache, mysql, etc.)

use std::io::Read;

use iced::widget::{button, column, container, row, scrollable, text, text_input, Column, Row};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::config::{AppTheme, Host, LayoutPreset};
use crate::theme;

// ─── Types ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SysTab {
    #[default]
    Overview,
    Firewall,
    Packages,
    Logins,
    SshKeys,
    Extension(String), // service id: "nginx", "mysql", etc.
}

impl SysTab {
    pub fn label(&self) -> &str {
        match self {
            SysTab::Overview => "Overview",
            SysTab::Firewall => "Firewall",
            SysTab::Packages => "Packages",
            SysTab::Logins => "Login History",
            SysTab::SshKeys => "SSH Keys",
            SysTab::Extension(n) => n.as_str(),
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "Overview" => SysTab::Overview,
            "Firewall" => SysTab::Firewall,
            "Packages" => SysTab::Packages,
            "Login History" => SysTab::Logins,
            "SSH Keys" => SysTab::SshKeys,
            other => SysTab::Extension(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SysState {
    pub tab: SysTab,
    pub output: String,
    pub loading: bool,
    pub action_result: Option<String>,
    pub extensions: Vec<ExtensionInfo>,
    // Firewall form
    pub fw_port: String,
    pub fw_proto: String,
    pub fw_action: String,
    // Package search
    pub pkg_search: String,
    // SSH Key gen
    pub key_name: String,
    pub key_type: String,
}

impl SysState {
    pub fn new() -> Self {
        Self {
            fw_proto: "tcp".into(),
            fw_action: "allow".into(),
            key_name: "id_termissh".into(),
            key_type: "ed25519".into(),
            loading: true,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub active: bool,
}

// ─── SSH Execution ───────────────────────────────────────────────────────────

fn ssh_exec_sync(host: Host, cmd: String) -> String {
    use ssh2::Session;
    use std::net::TcpStream;

    let tcp = match TcpStream::connect(format!("{}:{}", host.hostname, host.port)) {
        Ok(t) => t,
        Err(e) => return format!("[Connection Error] {e}"),
    };
    let mut sess = match Session::new() {
        Ok(s) => s,
        Err(e) => return format!("[Session Error] {e}"),
    };
    sess.set_tcp_stream(tcp);
    if sess.handshake().is_err() {
        return "[Handshake failed — check host/port]".into();
    }

    // Try SSH agent, then password
    let authed = sess.userauth_agent(&host.username).is_ok() && sess.authenticated();
    if !authed {
        let pass = host.password.as_deref().unwrap_or("");
        if pass.is_empty() {
            return "[Auth failed: no password and agent auth failed]".into();
        }
        if sess.userauth_password(&host.username, pass).is_err() {
            return "[Auth failed: wrong password]".into();
        }
    }

    let mut ch = match sess.channel_session() {
        Ok(c) => c,
        Err(e) => return format!("[Channel Error] {e}"),
    };
    if ch.exec(&cmd).is_err() {
        return "[Exec failed]".into();
    }

    let mut out = String::new();
    ch.read_to_string(&mut out).ok();
    let mut err_buf = String::new();
    ch.stderr().read_to_string(&mut err_buf).ok();
    ch.wait_close().ok();

    if out.is_empty() && !err_buf.is_empty() {
        err_buf
    } else if !err_buf.is_empty() {
        format!("{out}\n--- stderr ---\n{err_buf}")
    } else {
        out
    }
}

fn task_fetch(host: Host, tab_id: u64, kind: &'static str, cmd: String) -> iced::Task<Message> {
    iced::Task::perform(
        tokio::task::spawn_blocking(move || ssh_exec_sync(host, cmd)),
        move |res| {
            let output = match res {
                Ok(o) => o,
                Err(e) => format!("[Task Error] {e}"),
            };
            Message::SysPanelFetched(tab_id, kind.to_string(), output)
        },
    )
}

// ─── Fetch Tasks ─────────────────────────────────────────────────────────────

pub fn fetch_overview(host: Host, tab_id: u64) -> iced::Task<Message> {
    task_fetch(
        host,
        tab_id,
        "overview",
        r#"echo "=== HOSTNAME ===" && hostname && \
echo "" && echo "=== OS ===" && (cat /etc/os-release 2>/dev/null | grep -E "PRETTY_NAME|VERSION_ID" || uname -a) && \
echo "" && echo "=== UPTIME ===" && uptime && \
echo "" && echo "=== MEMORY ===" && free -h 2>/dev/null || vm_stat 2>/dev/null | head -10 && \
echo "" && echo "=== DISK ===" && df -h / && \
echo "" && echo "=== EXTENSIONS ===" && \
for s in nginx apache2 httpd mysql mariadb postgresql redis docker pm2 php-fpm; do \
  st=$(systemctl is-active $s 2>/dev/null || echo "inactive"); echo "$s:$st"; \
done"#
            .to_string(),
    )
}

pub fn fetch_firewall(host: Host, tab_id: u64) -> iced::Task<Message> {
    task_fetch(
        host,
        tab_id,
        "firewall",
        r#"echo "=== UFW Status ===" && sudo -n ufw status verbose 2>/dev/null && echo "[ok]" || \
echo "=== IPTables ===" && sudo -n iptables -L -n --line-numbers 2>/dev/null || \
echo "[Info] No accessible firewall tool found. Ensure the user has passwordless sudo for ufw/iptables.""#
            .to_string(),
    )
}

pub fn fetch_packages(host: Host, tab_id: u64) -> iced::Task<Message> {
    task_fetch(
        host,
        tab_id,
        "packages",
        r#"if command -v dpkg >/dev/null 2>&1; then \
  echo "=== Installed Packages (dpkg) ===" && \
  dpkg -l | tail -n +5 | awk '{printf "%-40s %-20s\n", $2, $3}' | head -400; \
elif command -v rpm >/dev/null 2>&1; then \
  echo "=== Installed Packages (rpm) ===" && \
  rpm -qa --qf "%-40{NAME} %-20{VERSION}\n" | sort | head -400; \
elif command -v apk >/dev/null 2>&1; then \
  echo "=== Installed Packages (apk) ===" && \
  apk list --installed 2>/dev/null | head -400; \
elif command -v brew >/dev/null 2>&1; then \
  echo "=== Installed Packages (brew) ===" && \
  brew list --versions 2>/dev/null | head -400; \
else echo "[Package manager not detected]"; fi"#
            .to_string(),
    )
}

pub fn fetch_logins(host: Host, tab_id: u64) -> iced::Task<Message> {
    task_fetch(
        host,
        tab_id,
        "logins",
        r#"echo "=== Currently Logged In ===" && w 2>/dev/null || who && \
echo "" && echo "=== Login History (last 30) ===" && \
last -n 30 2>/dev/null || echo "[last not available]" && \
echo "" && echo "=== Failed Logins (last 10) ===" && \
sudo -n lastb -n 10 2>/dev/null || \
grep "Failed password" /var/log/auth.log 2>/dev/null | tail -10 || \
echo "[no failed login data]""#
            .to_string(),
    )
}

pub fn fetch_ssh_keys(host: Host, tab_id: u64) -> iced::Task<Message> {
    task_fetch(
        host,
        tab_id,
        "sshkeys",
        r#"echo "=== ~/.ssh/ Files ===" && ls -la ~/.ssh/ 2>/dev/null || echo "(empty)" && \
echo "" && echo "=== Key Fingerprints ===" && \
for f in ~/.ssh/*.pub; do [ -f "$f" ] && echo "--- $f ---" && ssh-keygen -lf "$f" 2>/dev/null; done || echo "(no .pub files)" && \
echo "" && echo "=== Authorized Keys ===" && \
cat ~/.ssh/authorized_keys 2>/dev/null | head -15 || echo "(none)" && \
echo "" && echo "=== SSH Client Config ===" && \
cat ~/.ssh/config 2>/dev/null | head -30 || echo "(no config)" && \
echo "" && echo "=== Host Key (server) ===" && \
cat /etc/ssh/ssh_host_ed25519_key.pub 2>/dev/null || \
cat /etc/ssh/ssh_host_rsa_key.pub 2>/dev/null || echo "(no server keys readable)""#
            .to_string(),
    )
}

pub fn fetch_extension(host: Host, tab_id: u64, ext_id: String) -> iced::Task<Message> {
    let cmd = extension_fetch_cmd(&ext_id);
    task_fetch(host, tab_id, "extension", cmd)
}

pub fn run_action(host: Host, tab_id: u64, cmd: String) -> iced::Task<Message> {
    task_fetch(host, tab_id, "action", cmd)
}

fn extension_fetch_cmd(id: &str) -> String {
    match id {
        "nginx" => r#"echo "=== Nginx Status ===" && systemctl status nginx 2>/dev/null | head -20 && \
echo "" && echo "=== Config Test ===" && sudo -n nginx -t 2>&1 && \
echo "" && echo "=== Recent Access Log ===" && sudo -n tail -20 /var/log/nginx/access.log 2>/dev/null || echo "(no access)"
echo "" && echo "=== Recent Error Log ===" && sudo -n tail -10 /var/log/nginx/error.log 2>/dev/null || echo "(no access)""#.to_string(),
        "apache2" | "httpd" => format!(r#"echo "=== {id} Status ===" && systemctl status {id} 2>/dev/null | head -20 && \
echo "" && echo "=== Config Test ===" && sudo -n apachectl -t 2>&1 && \
echo "" && echo "=== Recent Access Log ===" && \
sudo -n tail -20 /var/log/apache2/access.log 2>/dev/null || \
sudo -n tail -20 /var/log/httpd/access_log 2>/dev/null || echo "(no access)""#),
        "mysql" | "mariadb" => format!(r#"echo "=== {id} Status ===" && systemctl status {id} 2>/dev/null | head -15 && \
echo "" && echo "=== Databases ===" && \
mysql -e "SHOW DATABASES;" 2>/dev/null || echo "(no access — set up .my.cnf or add credentials)""#),
        "postgresql" => r#"echo "=== PostgreSQL Status ===" && systemctl status postgresql 2>/dev/null | head -15 && \
echo "" && echo "=== Databases ===" && \
sudo -u postgres psql -l 2>/dev/null || echo "(no access)""#.to_string(),
        "redis" => r#"echo "=== Redis Status ===" && systemctl status redis 2>/dev/null | head -15 && \
echo "" && echo "=== Server Info ===" && \
redis-cli info server 2>/dev/null | head -20 || echo "(redis-cli not accessible)""#.to_string(),
        "docker" => r#"echo "=== Docker Status ===" && systemctl status docker 2>/dev/null | head -10 && \
echo "" && echo "=== Containers ===" && \
docker ps -a --format "table {{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null || echo "(no access — user not in docker group?)" && \
echo "" && echo "=== Images ===" && \
docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}" 2>/dev/null || echo "(no access)""#.to_string(),
        "pm2" => r#"echo "=== PM2 Status ===" && pm2 list 2>/dev/null || echo "(pm2 not found in PATH)" && \
echo "" && echo "=== PM2 Info ===" && pm2 info 2>/dev/null | head -20 || echo "(no info)""#.to_string(),
        other => format!("systemctl status {other} 2>/dev/null | head -25 || echo '[service not found]'"),
    }
}

// ─── Parse Extensions ────────────────────────────────────────────────────────

const KNOWN_EXTENSIONS: &[(&str, &str)] = &[
    ("nginx", "Nginx"),
    ("apache2", "Apache2"),
    ("httpd", "Apache HTTPD"),
    ("mysql", "MySQL"),
    ("mariadb", "MariaDB"),
    ("postgresql", "PostgreSQL"),
    ("redis", "Redis"),
    ("docker", "Docker"),
    ("pm2", "PM2"),
    ("php-fpm", "PHP-FPM"),
];

pub fn parse_extensions(output: &str) -> Vec<ExtensionInfo> {
    let mut exts = Vec::new();
    let mut in_section = false;
    for line in output.lines() {
        if line.contains("=== EXTENSIONS ===") {
            in_section = true;
            continue;
        }
        if !in_section {
            continue;
        }
        if line.starts_with("===") {
            break;
        }
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() == 2 {
            let svc = parts[0].trim();
            let status = parts[1].trim();
            if let Some(&(id, name)) = KNOWN_EXTENSIONS.iter().find(|(k, _)| *k == svc) {
                exts.push(ExtensionInfo {
                    id: id.to_string(),
                    name: name.to_string(),
                    active: status == "active",
                });
            }
        }
    }
    exts
}

// ─── Table Helpers ───────────────────────────────────────────────────────────

fn action_color(value: &str, p: theme::Palette) -> iced::Color {
    let v = value.to_uppercase();
    if v.contains("ALLOW") || v.contains("ACCEPT") {
        p.success
    } else if v.contains("DENY") || v.contains("REJECT") || v.contains("DROP") || v.contains("LIMIT") {
        p.danger
    } else {
        p.text_primary
    }
}

/// Renders a generic styled table. `accent_col` optionally colors one column
/// based on its content (ALLOW→green, DENY/DROP→red).
fn render_table(
    headers: &[(&'static str, u16)],
    rows: Vec<Vec<String>>,
    accent_col: Option<usize>,
    p: theme::Palette,
    cr: f32,
) -> Element<'static, Message> {
    let mut col: Column<'static, Message> = Column::new().spacing(0).width(Length::Fill);

    // Header row
    let mut header_row: Row<'static, Message> = Row::new();
    for &(label, portion) in headers {
        header_row = header_row.push(
            container(text(label).size(10).color(p.text_muted))
                .width(Length::FillPortion(portion))
                .padding([4, 8]),
        );
    }
    col = col.push(
        container(header_row)
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(p.bg_primary)),
                ..Default::default()
            }),
    );

    // Separator under header
    col = col.push(
        container(row![])
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .style(move |_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(p.border)),
                ..Default::default()
            }),
    );

    if rows.is_empty() {
        col = col.push(
            container(text("No data available").size(11).color(p.text_muted))
                .padding([6, 8])
                .width(Length::Fill),
        );
    } else {
        for (i, row_data) in rows.into_iter().enumerate() {
            let bg = if i % 2 == 0 { p.bg_secondary } else { p.bg_tertiary };
            let mut data_row: Row<'static, Message> = Row::new();
            for (j, &(_, portion)) in headers.iter().enumerate() {
                let cell_text = row_data.get(j).cloned().unwrap_or_default();
                let color = if accent_col == Some(j) {
                    action_color(&cell_text, p)
                } else {
                    p.text_primary
                };
                data_row = data_row.push(
                    container(text(cell_text).size(11).color(color))
                        .width(Length::FillPortion(portion))
                        .padding([3, 8]),
                );
            }
            col = col.push(
                container(data_row)
                    .width(Length::Fill)
                    .style(move |_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(bg)),
                        ..Default::default()
                    }),
            );
        }
    }

    container(col)
        .width(Length::Fill)
        .style(move |_: &iced::Theme| container::Style {
            border: iced::Border { color: p.border, width: 1.0, radius: cr.into() },
            ..Default::default()
        })
        .into()
}

fn parse_firewall_rules(output: &str) -> (bool, Vec<Vec<String>>) {
    // Detect by whether the UFW rules header ("To  Action  From") is actually
    // present in the output — NOT just by the "=== UFW Status ===" echo which is
    // always printed before sudo runs (even when sudo fails).
    let is_ufw = output.lines().any(|l| {
        let t = l.trim();
        t.starts_with("To") && t.contains("Action") && t.contains("From")
    });

    let mut rows: Vec<Vec<String>> = Vec::new();

    if is_ufw {
        let mut in_rules = false;
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            // Detect the header line and start collecting rules after it
            if !in_rules && trimmed.starts_with("To") && trimmed.contains("Action") {
                in_rules = true;
                continue;
            }
            if !in_rules { continue; }
            // Skip the dashes separator row
            if trimmed.starts_with("--") { continue; }
            // Skip the trailing "[ok]" marker
            if trimmed == "[ok]" { continue; }
            // UFW columns are separated by 2+ spaces; single spaces inside a
            // column value (e.g. "ALLOW IN", "Anywhere (v6)") are preserved.
            let parts: Vec<&str> = line.split("  ")
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            if parts.len() >= 2 {
                rows.push(vec![
                    parts[0].to_string(),
                    parts.get(1).unwrap_or(&"").to_string(),
                    parts.get(2).unwrap_or(&"*").to_string(),
                ]);
            }
        }
    } else {
        // iptables: num target prot opt source destination [extras]
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("Chain") || trimmed.starts_with("num")
                || trimmed.starts_with("target") || trimmed.starts_with("===") { continue; }
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 5 && parts[0].parse::<u32>().is_ok() {
                rows.push(vec![
                    parts[1].to_string(),                     // target (ACCEPT/DROP…)
                    parts[2].to_string(),                     // protocol
                    parts[4].to_string(),                     // source
                    parts.get(5).unwrap_or(&"*").to_string(), // destination
                ]);
            }
        }
    }

    (is_ufw, rows)
}

fn parse_packages(output: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut in_data = false;
    for line in output.lines() {
        if line.starts_with("===") { in_data = true; continue; }
        if !in_data { continue; }
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if !parts.is_empty() {
            rows.push(vec![
                parts[0].to_string(),
                parts.get(1).cloned().unwrap_or_default().to_string(),
            ]);
        }
    }
    rows
}

fn parse_logins(output: &str) -> (Vec<Vec<String>>, Vec<Vec<String>>, Vec<Vec<String>>) {
    let mut current: Vec<Vec<String>> = Vec::new();
    let mut history: Vec<Vec<String>> = Vec::new();
    let mut failed: Vec<Vec<String>> = Vec::new();
    let mut section: u8 = 0;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("Currently Logged In") { section = 1; continue; }
        if trimmed.contains("Login History") { section = 2; continue; }
        if trimmed.contains("Failed Logins") { section = 3; continue; }
        if trimmed.starts_with("===") || trimmed.is_empty() { continue; }
        if trimmed.starts_with("USER") || trimmed.starts_with("wtmp") || trimmed.starts_with("btmp") { continue; }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 2 { continue; }

        let row = vec![
            parts[0].to_string(),
            parts.get(1).cloned().unwrap_or_default().to_string(),
            parts.get(2).cloned().unwrap_or_default().to_string(),
            if parts.len() > 3 { parts[3..].join(" ") } else { String::new() },
        ];

        match section {
            1 => current.push(row),
            2 => history.push(row),
            3 => failed.push(row),
            _ => {}
        }
    }

    (current, history, failed)
}

// ─── View ────────────────────────────────────────────────────────────────────

fn btn_style(p: theme::Palette, accent: bool, cr: f32) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_: &iced::Theme, s: button::Status| button::Style {
        background: Some(iced::Background::Color(if accent {
            match s {
                button::Status::Hovered | button::Status::Pressed => p.accent_hover,
                _ => p.accent,
            }
        } else {
            match s {
                button::Status::Hovered | button::Status::Pressed => p.bg_hover,
                _ => p.bg_tertiary,
            }
        })),
        text_color: p.text_primary,
        border: iced::Border { color: p.border, width: 1.0, radius: cr.into() },
        ..Default::default()
    }
}

fn input_style(p: theme::Palette, cr: f32) -> impl Fn(&iced::Theme, text_input::Status) -> text_input::Style {
    move |_: &iced::Theme, status: text_input::Status| text_input::Style {
        background: iced::Background::Color(p.bg_tertiary),
        border: iced::Border {
            color: match status {
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
}

pub fn view_sys_panel(
    tab_id: u64,
    state: &SysState,
    _host: &Host,
    theme: AppTheme,
    layout: LayoutPreset,
) -> Element<'static, Message> {
    let p = theme::palette(theme);
    let cr = theme::layout(layout).corner_radius;

    // ── Tab bar ──────────────────────────────────────────────────────────────
    let mut tabs: Vec<SysTab> = vec![
        SysTab::Overview,
        SysTab::Firewall,
        SysTab::Packages,
        SysTab::Logins,
        SysTab::SshKeys,
    ];
    for ext in &state.extensions {
        tabs.push(SysTab::Extension(ext.id.clone()));
    }

    let mut tab_row = Row::new().spacing(2).align_y(Alignment::Center).padding([4, 8]);

    // Back to terminal button
    tab_row = tab_row.push(
        button(text("← Terminal").size(11).color(p.text_muted))
            .on_press(Message::SysPanelClose(tab_id))
            .padding([3, 10])
            .style(btn_style(p, false, cr)),
    );

    for t in &tabs {
        let label = t.label().to_string();
        let is_active = *t == state.tab;
        let tab_label_clone = label.clone();
        let is_ext = matches!(t, SysTab::Extension(_));
        let ext_active = if is_ext {
            state.extensions.iter().find(|e| SysTab::Extension(e.id.clone()) == *t).map(|e| e.active).unwrap_or(false)
        } else {
            false
        };

        tab_row = tab_row.push(
            button(
                row![
                    text(label).size(11).color(if is_active { p.text_primary } else { p.text_muted }),
                    if is_ext && ext_active {
                        text(" ●").size(9).color(p.success)
                    } else if is_ext {
                        text(" ○").size(9).color(p.text_muted)
                    } else {
                        text("").size(9).color(p.text_muted)
                    },
                ]
                .spacing(0)
                .align_y(Alignment::Center),
            )
            .on_press(Message::SysPanelTabSwitch(tab_id, tab_label_clone))
            .padding([3, 10])
            .style(move |_: &iced::Theme, s: button::Status| button::Style {
                background: Some(iced::Background::Color(if is_active {
                    p.accent
                } else {
                    match s {
                        button::Status::Hovered | button::Status::Pressed => p.bg_hover,
                        _ => p.bg_tertiary,
                    }
                })),
                text_color: if is_active { p.text_primary } else { p.text_muted },
                border: iced::Border { color: p.border, width: 1.0, radius: cr.into() },
                ..Default::default()
            }),
        );
    }

    let tab_bar = container(tab_row)
        .width(Length::Fill)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg_tertiary)),
            border: iced::Border { color: p.border, width: 0.0, radius: 0.0.into() },
            ..Default::default()
        });

    // ── Action result banner ─────────────────────────────────────────────────
    let action_banner: Element<'static, Message> = if let Some(msg) = &state.action_result {
        let msg_clone = msg.clone();
        container(text(msg_clone).size(11).color(p.success))
            .padding([3, 12])
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(p.bg_tertiary)),
                border: iced::Border { color: p.success, width: 1.0, radius: cr.into() },
                ..Default::default()
            })
            .into()
    } else {
        container(text("").size(1)).height(Length::Fixed(0.0)).into()
    };

    // ── Content ──────────────────────────────────────────────────────────────
    let content: Element<'static, Message> = if state.loading {
        container(
            column![
                text("⟳  Loading...").size(13).color(p.text_muted),
                text("Connecting via SSH to fetch data").size(11).color(p.text_muted),
            ]
            .spacing(6)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        match &state.tab {
            SysTab::Overview => view_overview(tab_id, state, p, cr),
            SysTab::Firewall => view_firewall(tab_id, state, p, cr),
            SysTab::Packages => view_packages(tab_id, state, p, cr),
            SysTab::Logins => view_logins(tab_id, state, p, cr),
            SysTab::SshKeys => view_ssh_keys(tab_id, state, p, cr),
            SysTab::Extension(id) => view_extension(tab_id, id.clone(), state, p, cr),
        }
    };

    container(
        column![tab_bar, action_banner, content].spacing(0).height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(p.bg_secondary)),
        ..Default::default()
    })
    .into()
}

// ─── Overview ────────────────────────────────────────────────────────────────

fn view_overview(
    tab_id: u64,
    state: &SysState,
    p: theme::Palette,
    cr: f32,
) -> Element<'static, Message> {
    let output = state.output.clone();

    // Extension cards — type annotation needed for Renderer inference
    let mut ext_cards: Row<'static, Message> = Row::new().spacing(6).align_y(Alignment::Start);
    for ext in &state.extensions {
        let ext_id = ext.id.clone();
        let ext_name = ext.name.clone();
        let is_active = ext.active;
        let status_color = if is_active { p.success } else { p.text_muted };
        let status_txt = if is_active { "● active" } else { "○ inactive" };
        let tab_id_inner = tab_id;

        ext_cards = ext_cards.push(
            container(
                column![
                    text(ext_name).size(12).color(p.text_primary),
                    text(status_txt).size(10).color(status_color),
                    button(text("→ Manage").size(10).color(p.accent))
                        .on_press(Message::SysPanelTabSwitch(tab_id_inner, ext_id))
                        .padding([2, 6])
                        .style(move |_: &iced::Theme, s: button::Status| button::Style {
                            background: Some(iced::Background::Color(match s {
                                button::Status::Hovered => p.bg_hover,
                                _ => iced::Color::TRANSPARENT,
                            })),
                            text_color: p.accent,
                            border: iced::Border::default(),
                            ..Default::default()
                        }),
                ]
                .spacing(3),
            )
            .padding([8, 12])
            .style(move |_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(p.bg_tertiary)),
                border: iced::Border {
                    color: if is_active { p.success } else { p.border },
                    width: 1.0,
                    radius: cr.into(),
                },
                ..Default::default()
            }),
        );
    }

    column![
        row![
            text("System Overview").size(14).color(p.text_primary),
            button(text("↻ Refresh").size(11).color(p.text_primary))
                .on_press(Message::SysPanelFetch(tab_id, "overview".into()))
                .padding([3, 10])
                .style(btn_style(p, true, cr)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        if !state.extensions.is_empty() {
            let ext_section: Element<'static, Message> = column![
                text("Detected Services").size(11).color(p.text_secondary),
                ext_cards,
            ]
            .spacing(6)
            .into();
            ext_section
        } else {
            let no_ext: Element<'static, Message> = column![
                text("No services detected (nginx, apache, mysql, redis, docker, etc.)")
                    .size(11)
                    .color(p.text_muted)
            ]
            .into();
            no_ext
        },
        scrollable(
            text(output)
                .size(11)
                .color(p.text_primary)
                .font(iced::Font::MONOSPACE),
        )
        .height(Length::Fill)
        .style(hidden_scrollbar_style),
    ]
    .spacing(10)
    .padding([8, 12])
    .height(Length::Fill)
    .into()
}

// ─── Firewall ────────────────────────────────────────────────────────────────

fn view_firewall(
    tab_id: u64,
    state: &SysState,
    p: theme::Palette,
    cr: f32,
) -> Element<'static, Message> {
    let output = state.output.clone();
    let fw_port = state.fw_port.clone();
    let fw_proto = state.fw_proto.clone();
    let fw_action = state.fw_action.clone();

    let proto_allow = fw_proto == "tcp";
    let action_allow = fw_action == "allow";

    // Port input
    let port_input = text_input("Port (e.g. 80)", &fw_port)
        .on_input(move |v| Message::SysPanelInput(tab_id, "fw_port".into(), v))
        .padding(6)
        .size(12)
        .width(Length::Fixed(120.0))
        .style(input_style(p, cr));

    // Proto selector
    let tcp_btn = button(text("TCP").size(11).color(p.text_primary))
        .on_press(Message::SysPanelInput(tab_id, "fw_proto".into(), "tcp".into()))
        .padding([4, 10])
        .style(move |_: &iced::Theme, s: button::Status| button::Style {
            background: Some(iced::Background::Color(if proto_allow {
                p.accent
            } else {
                match s {
                    button::Status::Hovered => p.bg_hover,
                    _ => p.bg_tertiary,
                }
            })),
            text_color: p.text_primary,
            border: iced::Border { color: p.border, width: 1.0, radius: cr.into() },
            ..Default::default()
        });
    let udp_btn = button(text("UDP").size(11).color(p.text_primary))
        .on_press(Message::SysPanelInput(tab_id, "fw_proto".into(), "udp".into()))
        .padding([4, 10])
        .style(move |_: &iced::Theme, s: button::Status| button::Style {
            background: Some(iced::Background::Color(if !proto_allow {
                p.accent
            } else {
                match s {
                    button::Status::Hovered => p.bg_hover,
                    _ => p.bg_tertiary,
                }
            })),
            text_color: p.text_primary,
            border: iced::Border { color: p.border, width: 1.0, radius: cr.into() },
            ..Default::default()
        });

    // Action selector
    let allow_btn = button(text("Allow").size(11).color(p.text_primary))
        .on_press(Message::SysPanelInput(tab_id, "fw_action".into(), "allow".into()))
        .padding([4, 10])
        .style(move |_: &iced::Theme, s: button::Status| button::Style {
            background: Some(iced::Background::Color(if action_allow {
                p.success
            } else {
                match s {
                    button::Status::Hovered => p.bg_hover,
                    _ => p.bg_tertiary,
                }
            })),
            text_color: p.text_primary,
            border: iced::Border { color: p.border, width: 1.0, radius: cr.into() },
            ..Default::default()
        });
    let deny_btn = button(text("Deny").size(11).color(p.text_primary))
        .on_press(Message::SysPanelInput(tab_id, "fw_action".into(), "deny".into()))
        .padding([4, 10])
        .style(move |_: &iced::Theme, s: button::Status| button::Style {
            background: Some(iced::Background::Color(if !action_allow {
                p.danger
            } else {
                match s {
                    button::Status::Hovered => p.bg_hover,
                    _ => p.bg_tertiary,
                }
            })),
            text_color: p.text_primary,
            border: iced::Border { color: p.border, width: 1.0, radius: cr.into() },
            ..Default::default()
        });

    let port_c = fw_port.clone();
    let proto_c = fw_proto.clone();
    let action_c = fw_action.clone();

    let apply_cmd = format!("sudo -n ufw {action_c} {port_c}/{proto_c}");
    let apply_btn = button(text("Apply Rule").size(11).color(p.text_primary))
        .on_press(Message::SysPanelAction(tab_id, apply_cmd))
        .padding([4, 14])
        .style(btn_style(p, true, cr));

    // Quick action buttons
    let make_quick = |label: &'static str, cmd: &'static str| {
        button(text(label).size(11).color(p.text_primary))
            .on_press(Message::SysPanelAction(tab_id, cmd.to_string()))
            .padding([3, 10])
            .style(btn_style(p, false, cr))
    };

    let (is_ufw, rules) = parse_firewall_rules(&output);
    let rule_count = rules.len();

    let rules_table: Element<'static, Message> = if output.contains("[Info]")
        || (!is_ufw && rules.is_empty() && output.contains('['))
    {
        container(
            text("No accessible firewall found. Install ufw or ensure passwordless sudo for ufw/iptables.")
                .size(11)
                .color(p.text_muted),
        )
        .padding([8, 8])
        .into()
    } else if is_ufw {
        render_table(
            &[("PORT / SERVICE", 3), ("ACTION", 2), ("FROM / SOURCE", 3)],
            rules,
            Some(1), // color the Action column
            p,
            cr,
        )
    } else {
        render_table(
            &[("TARGET", 2), ("PROTOCOL", 1), ("SOURCE", 3), ("DESTINATION", 3)],
            rules,
            Some(0), // color the Target column
            p,
            cr,
        )
    };

    column![
        row![
            text("Firewall Manager").size(14).color(p.text_primary),
            button(text("↻ Refresh").size(11).color(p.text_primary))
                .on_press(Message::SysPanelFetch(tab_id, "firewall".into()))
                .padding([3, 10])
                .style(btn_style(p, true, cr)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        text("Requires passwordless sudo for ufw. Use: sudo visudo → add '<user> ALL=(ALL) NOPASSWD: /usr/sbin/ufw'")
            .size(10)
            .color(p.text_muted),
        // Rule form
        container(
            column![
                text("Add / Remove Rule").size(12).color(p.text_secondary),
                row![
                    port_input,
                    row![tcp_btn, udp_btn].spacing(4),
                    row![allow_btn, deny_btn].spacing(4),
                    apply_btn,
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                // Quick buttons
                row![
                    make_quick("Enable UFW", "sudo -n ufw enable"),
                    make_quick("Disable UFW", "sudo -n ufw disable"),
                    make_quick("Reload", "sudo -n ufw reload"),
                    make_quick("Allow SSH (22)", "sudo -n ufw allow 22/tcp"),
                    make_quick("Allow HTTP (80)", "sudo -n ufw allow 80/tcp"),
                    make_quick("Allow HTTPS (443)", "sudo -n ufw allow 443/tcp"),
                ]
                .spacing(4)
                .wrap(),
            ]
            .spacing(8),
        )
        .padding([10, 12])
        .width(Length::Fill)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg_tertiary)),
            border: iced::Border { color: p.border, width: 1.0, radius: cr.into() },
            ..Default::default()
        }),
        text(format!("{rule_count} active rule(s)")).size(10).color(p.text_muted),
        scrollable(rules_table)
            .height(Length::Fill)
            .style(hidden_scrollbar_style),
    ]
    .spacing(8)
    .padding([8, 12])
    .height(Length::Fill)
    .into()
}

// ─── Packages ────────────────────────────────────────────────────────────────

fn view_packages(
    tab_id: u64,
    state: &SysState,
    p: theme::Palette,
    cr: f32,
) -> Element<'static, Message> {
    let pkg_search = state.pkg_search.clone();
    let output = state.output.clone();

    let all_rows = parse_packages(&output);
    let rows: Vec<Vec<String>> = if pkg_search.is_empty() {
        all_rows
    } else {
        let lower = pkg_search.to_lowercase();
        all_rows
            .into_iter()
            .filter(|r| r.iter().any(|c| c.to_lowercase().contains(&lower)))
            .collect()
    };
    let row_count = rows.len();

    let search_input = text_input("Search packages...", &pkg_search)
        .on_input(move |v| Message::SysPanelInput(tab_id, "pkg_search".into(), v))
        .padding(6)
        .size(12)
        .style(input_style(p, cr));

    column![
        row![
            text("Package Manager").size(14).color(p.text_primary),
            button(text("↻ Refresh").size(11).color(p.text_primary))
                .on_press(Message::SysPanelFetch(tab_id, "packages".into()))
                .padding([3, 10])
                .style(btn_style(p, true, cr)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        row![
            search_input,
            text(format!("{row_count} package(s)")).size(11).color(p.text_muted),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        scrollable(render_table(
            &[("PACKAGE", 3), ("VERSION", 2)],
            rows,
            None,
            p,
            cr,
        ))
        .height(Length::Fill)
        .style(hidden_scrollbar_style),
    ]
    .spacing(8)
    .padding([8, 12])
    .height(Length::Fill)
    .into()
}

// ─── Login History ───────────────────────────────────────────────────────────

fn view_logins(
    tab_id: u64,
    state: &SysState,
    p: theme::Palette,
    cr: f32,
) -> Element<'static, Message> {
    let output = state.output.clone();
    let (current, history, failed) = parse_logins(&output);

    let section_header = |label: &'static str, color: iced::Color| -> Element<'static, Message> {
        text(label).size(12).color(color).into()
    };

    column![
        row![
            text("Login History").size(14).color(p.text_primary),
            button(text("↻ Refresh").size(11).color(p.text_primary))
                .on_press(Message::SysPanelFetch(tab_id, "logins".into()))
                .padding([3, 10])
                .style(btn_style(p, true, cr)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        scrollable(
            column![
                section_header("Currently Logged In", p.text_secondary),
                render_table(
                    &[("USER", 2), ("TTY", 1), ("FROM", 3), ("INFO", 4)],
                    current,
                    None,
                    p,
                    cr,
                ),
                section_header("Login History (last 30)", p.text_secondary),
                render_table(
                    &[("USER", 2), ("TTY", 1), ("FROM", 3), ("DATE / DURATION", 4)],
                    history,
                    None,
                    p,
                    cr,
                ),
                section_header("Failed Login Attempts", p.danger),
                render_table(
                    &[("USER", 2), ("TTY", 1), ("FROM", 3), ("DATE", 4)],
                    failed,
                    None,
                    p,
                    cr,
                ),
            ]
            .spacing(8),
        )
        .height(Length::Fill)
        .style(hidden_scrollbar_style),
    ]
    .spacing(8)
    .padding([8, 12])
    .height(Length::Fill)
    .into()
}

// ─── SSH Keys ────────────────────────────────────────────────────────────────

fn view_ssh_keys(
    tab_id: u64,
    state: &SysState,
    p: theme::Palette,
    cr: f32,
) -> Element<'static, Message> {
    let output = state.output.clone();
    let key_name = state.key_name.clone();
    let key_type = state.key_type.clone();
    let is_ed = key_type == "ed25519";
    let kn = key_name.clone();
    let kt = key_type.clone();

    let keygen_cmd = format!(
        r#"ssh-keygen -t {kt} -N "" -f ~/.ssh/{kn} && echo "Key generated: ~/.ssh/{kn}" && cat ~/.ssh/{kn}.pub"#
    );

    let name_input = text_input("Key filename (e.g. id_termissh)", &key_name)
        .on_input(move |v| Message::SysPanelInput(tab_id, "key_name".into(), v))
        .padding(6)
        .size(12)
        .width(Length::Fixed(200.0))
        .style(input_style(p, cr));

    column![
        row![
            text("SSH Key Manager").size(14).color(p.text_primary),
            button(text("↻ Refresh").size(11).color(p.text_primary))
                .on_press(Message::SysPanelFetch(tab_id, "sshkeys".into()))
                .padding([3, 10])
                .style(btn_style(p, true, cr)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        // Key generation form
        container(
            column![
                text("Generate New Key Pair").size(12).color(p.text_secondary),
                row![
                    // Key type selector
                    button(text("Ed25519").size(11).color(p.text_primary))
                        .on_press(Message::SysPanelInput(tab_id, "key_type".into(), "ed25519".into()))
                        .padding([4, 10])
                        .style(move |_: &iced::Theme, s: button::Status| button::Style {
                            background: Some(iced::Background::Color(if is_ed {
                                p.accent
                            } else {
                                match s {
                                    button::Status::Hovered => p.bg_hover,
                                    _ => p.bg_tertiary,
                                }
                            })),
                            text_color: p.text_primary,
                            border: iced::Border { color: p.border, width: 1.0, radius: cr.into() },
                            ..Default::default()
                        }),
                    button(text("RSA 4096").size(11).color(p.text_primary))
                        .on_press(Message::SysPanelInput(tab_id, "key_type".into(), "rsa".into()))
                        .padding([4, 10])
                        .style(move |_: &iced::Theme, s: button::Status| button::Style {
                            background: Some(iced::Background::Color(if !is_ed {
                                p.accent
                            } else {
                                match s {
                                    button::Status::Hovered => p.bg_hover,
                                    _ => p.bg_tertiary,
                                }
                            })),
                            text_color: p.text_primary,
                            border: iced::Border { color: p.border, width: 1.0, radius: cr.into() },
                            ..Default::default()
                        }),
                    name_input,
                    button(text("Generate").size(11).color(p.text_primary))
                        .on_press(Message::SysPanelAction(tab_id, keygen_cmd))
                        .padding([4, 14])
                        .style(btn_style(p, true, cr)),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text("Keys are generated on the REMOTE machine (the SSH server). Files are saved to ~/.ssh/")
                    .size(10)
                    .color(p.text_muted),
            ]
            .spacing(8),
        )
        .padding([10, 12])
        .width(Length::Fill)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg_tertiary)),
            border: iced::Border { color: p.border, width: 1.0, radius: cr.into() },
            ..Default::default()
        }),
        scrollable(
            text(output)
                .size(11)
                .color(p.text_primary)
                .font(iced::Font::MONOSPACE),
        )
        .height(Length::Fill)
        .style(hidden_scrollbar_style),
    ]
    .spacing(8)
    .padding([8, 12])
    .height(Length::Fill)
    .into()
}

// ─── Extension ───────────────────────────────────────────────────────────────

fn view_extension(
    tab_id: u64,
    ext_id: String,
    state: &SysState,
    p: theme::Palette,
    cr: f32,
) -> Element<'static, Message> {
    let output = state.output.clone();
    let ext_info = state
        .extensions
        .iter()
        .find(|e| e.id == ext_id)
        .cloned();
    let display_name = ext_info
        .as_ref()
        .map(|e| e.name.clone())
        .unwrap_or_else(|| ext_id.clone());
    let is_active = ext_info.map(|e| e.active).unwrap_or(false);

    let id1 = ext_id.clone();
    let id2 = ext_id.clone();
    let id3 = ext_id.clone();
    let id4 = ext_id.clone();

    let make_svc_btn = |label: &'static str, action: String| {
        button(text(label).size(11).color(p.text_primary))
            .on_press(Message::SysPanelAction(
                tab_id,
                format!("sudo -n systemctl {action}"),
            ))
            .padding([3, 10])
            .style(btn_style(p, false, cr))
    };

    // Extra service-specific actions
    let extra_btns: Vec<Element<'static, Message>> = match ext_id.as_str() {
        "nginx" => vec![
            button(text("Config Test").size(11).color(p.text_primary))
                .on_press(Message::SysPanelAction(tab_id, "sudo -n nginx -t 2>&1".into()))
                .padding([3, 10])
                .style(btn_style(p, false, cr))
                .into(),
            button(text("Access Log").size(11).color(p.text_primary))
                .on_press(Message::SysPanelAction(tab_id, "sudo -n tail -30 /var/log/nginx/access.log 2>/dev/null || echo 'no log'".into()))
                .padding([3, 10])
                .style(btn_style(p, false, cr))
                .into(),
            button(text("Error Log").size(11).color(p.text_primary))
                .on_press(Message::SysPanelAction(tab_id, "sudo -n tail -20 /var/log/nginx/error.log 2>/dev/null || echo 'no log'".into()))
                .padding([3, 10])
                .style(btn_style(p, false, cr))
                .into(),
        ],
        "apache2" | "httpd" => vec![
            button(text("Config Test").size(11).color(p.text_primary))
                .on_press(Message::SysPanelAction(tab_id, "sudo -n apachectl -t 2>&1".into()))
                .padding([3, 10])
                .style(btn_style(p, false, cr))
                .into(),
            button(text("Access Log").size(11).color(p.text_primary))
                .on_press(Message::SysPanelAction(
                    tab_id,
                    "sudo -n tail -30 /var/log/apache2/access.log 2>/dev/null || sudo -n tail -30 /var/log/httpd/access_log 2>/dev/null || echo 'no log'"
                        .into(),
                ))
                .padding([3, 10])
                .style(btn_style(p, false, cr))
                .into(),
        ],
        "mysql" | "mariadb" => vec![
            button(text("Show DBs").size(11).color(p.text_primary))
                .on_press(Message::SysPanelAction(tab_id, "mysql -e 'SHOW DATABASES;' 2>/dev/null || echo 'no mysql access'".into()))
                .padding([3, 10])
                .style(btn_style(p, false, cr))
                .into(),
        ],
        "docker" => vec![
            button(text("Containers").size(11).color(p.text_primary))
                .on_press(Message::SysPanelAction(
                    tab_id,
                    "docker ps -a --format 'table {{.Names}}\\t{{.Image}}\\t{{.Status}}' 2>/dev/null || echo 'no docker access'".into(),
                ))
                .padding([3, 10])
                .style(btn_style(p, false, cr))
                .into(),
            button(text("Images").size(11).color(p.text_primary))
                .on_press(Message::SysPanelAction(
                    tab_id,
                    "docker images --format 'table {{.Repository}}\\t{{.Tag}}\\t{{.Size}}' 2>/dev/null || echo 'no docker access'".into(),
                ))
                .padding([3, 10])
                .style(btn_style(p, false, cr))
                .into(),
            button(text("Prune").size(11).color(p.warning))
                .on_press(Message::SysPanelAction(
                    tab_id,
                    "docker system prune -f 2>/dev/null || echo 'no docker access'".into(),
                ))
                .padding([3, 10])
                .style(btn_style(p, false, cr))
                .into(),
        ],
        _ => vec![],
    };

    let status_color = if is_active { p.success } else { p.danger };
    let status_txt = if is_active { "● Running" } else { "○ Stopped" };

    let mut action_row = Row::new().spacing(6).align_y(Alignment::Center);
    action_row = action_row.push(make_svc_btn("Start", format!("start {id1}")));
    action_row = action_row.push(make_svc_btn("Stop", format!("stop {id2}")));
    action_row = action_row.push(make_svc_btn("Restart", format!("restart {id3}")));
    action_row = action_row.push(make_svc_btn("Reload", format!("reload {id4} 2>/dev/null || echo 'reload not supported'")));
    for btn in extra_btns {
        action_row = action_row.push(btn);
    }
    action_row = action_row.push(
        button(text("↻ Refresh").size(11).color(p.text_primary))
            .on_press(Message::SysPanelFetch(tab_id, ext_id))
            .padding([3, 10])
            .style(btn_style(p, true, cr)),
    );

    column![
        row![
            text(display_name).size(14).color(p.text_primary),
            text(status_txt).size(12).color(status_color),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        action_row,
        text("Actions require passwordless sudo. Check server sudo config if buttons don't work.")
            .size(10)
            .color(p.text_muted),
        scrollable(
            text(output)
                .size(11)
                .color(p.text_primary)
                .font(iced::Font::MONOSPACE),
        )
        .height(Length::Fill)
        .style(hidden_scrollbar_style),
    ]
    .spacing(8)
    .padding([8, 12])
    .height(Length::Fill)
    .into()
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
