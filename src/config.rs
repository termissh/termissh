use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Host {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub id: Option<String>,
    pub alias: String,
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub enum Language {
    Turkish,
    #[default]
    English,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub enum AppTheme {
    #[default]
    Dark,
    Light,
    Dracula,
    Nord,
    Solarized,
    MonoDark,
    MonoLight,
    Haki,
    SoftRose,
    SoftSky,
    CyberPunk,
    Mocha,
    Ocean,
    Forest,
    // New themes
    Gruvbox,
    TokyoNight,
    OneDark,
    Ayu,
    Rosepine,
    Kanagawa,
    Everforest,
    Midnight,
}

impl AppTheme {
    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::Dracula => "Dracula",
            Self::Nord => "Nord",
            Self::Solarized => "Solarized",
            Self::MonoDark => "Mono Dark",
            Self::MonoLight => "Mono Light",
            Self::Haki => "Haki",
            Self::SoftRose => "Soft Rose",
            Self::SoftSky => "Soft Sky",
            Self::CyberPunk => "CyberPunk",
            Self::Mocha => "Mocha",
            Self::Ocean => "Ocean",
            Self::Forest => "Forest",
            Self::Gruvbox => "Gruvbox",
            Self::TokyoNight => "Tokyo Night",
            Self::OneDark => "One Dark",
            Self::Ayu => "Ayu Dark",
            Self::Rosepine => "Rosé Pine",
            Self::Kanagawa => "Kanagawa",
            Self::Everforest => "Everforest",
            Self::Midnight => "Midnight",
        }
    }

    pub fn all() -> &'static [AppTheme] {
        &[
            Self::Dark,
            Self::Light,
            Self::Dracula,
            Self::Nord,
            Self::Solarized,
            Self::MonoDark,
            Self::MonoLight,
            Self::Haki,
            Self::SoftRose,
            Self::SoftSky,
            Self::CyberPunk,
            Self::Mocha,
            Self::Ocean,
            Self::Forest,
            Self::Gruvbox,
            Self::TokyoNight,
            Self::OneDark,
            Self::Ayu,
            Self::Rosepine,
            Self::Kanagawa,
            Self::Everforest,
            Self::Midnight,
        ]
    }

    pub fn is_light(self) -> bool {
        matches!(self, Self::Light | Self::MonoLight)
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub enum LayoutPreset {
    #[default]
    Vega,   // clean & familiar
    Nova,   // compact
    Maia,   // soft & rounded
    Lyra,   // boxy & sharp
    Mira,   // ultra dense
    // New presets
    Zeta,   // wide sidebar, spacious cards
    Orion,  // terminal-first, narrow sidebar
    Aria,   // balanced, centered UI
    Dawn,   // extra rounded, airy
    Flux,   // floating panels, large gaps
}

impl LayoutPreset {
    pub fn label(self) -> &'static str {
        match self {
            Self::Vega  => "Vega",
            Self::Nova  => "Nova",
            Self::Maia  => "Maia",
            Self::Lyra  => "Lyra",
            Self::Mira  => "Mira",
            Self::Zeta  => "Zeta",
            Self::Orion => "Orion",
            Self::Aria  => "Aria",
            Self::Dawn  => "Dawn",
            Self::Flux  => "Flux",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Vega  => "Clean, neutral & familiar",
            Self::Nova  => "Reduced padding, compact",
            Self::Maia  => "Soft & rounded, generous",
            Self::Lyra  => "Boxy & sharp, mono-friendly",
            Self::Mira  => "Ultra dense, minimal space",
            Self::Zeta  => "Wide sidebar, spacious cards",
            Self::Orion => "Terminal-first, narrow sidebar",
            Self::Aria  => "Balanced & centered panels",
            Self::Dawn  => "Extra rounded, airy spacing",
            Self::Flux  => "Floating panels, large gaps",
        }
    }

    pub fn all() -> &'static [LayoutPreset] {
        &[
            Self::Vega, Self::Nova, Self::Maia, Self::Lyra, Self::Mira,
            Self::Zeta, Self::Orion, Self::Aria, Self::Dawn, Self::Flux,
        ]
    }
}

impl std::fmt::Display for LayoutPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}  ·  {}", self.label(), self.description())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CustomCommand {
    pub trigger: String,     // e.g., "-runtest"
    pub script: String,      // e.g., "cd /app && npm test"
    pub description: String, // optional description
}

fn default_font_size() -> f32 { 13.0 }
fn default_true() -> bool { true }
fn default_suggestions() -> bool { true }

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AppConfig {
    pub hosts: Vec<Host>,
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_url: Option<String>,
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub theme: AppTheme,
    #[serde(default)]
    pub layout: LayoutPreset,
    #[serde(default)]
    pub custom_commands: Vec<CustomCommand>,
    // Terminal appearance
    #[serde(default = "default_font_size")]
    pub terminal_font_size: f32,
    #[serde(default = "default_true")]
    pub show_borders: bool,
    #[serde(default = "default_suggestions")]
    pub suggestions_enabled: bool,
}

// --- Encryption helpers ---

fn derive_key() -> [u8; 32] {
    let machine_id = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "termissh-default".to_string());
    let input = format!("{}::termissh-cipher-v1", machine_id);
    let hash = Sha256::digest(input.as_bytes());
    hash.into()
}

fn nonce_from_time() -> [u8; 12] {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let mut n = [0u8; 12];
    n[0..8].copy_from_slice(&now.as_secs().to_le_bytes());
    n[8..12].copy_from_slice(&now.subsec_nanos().to_le_bytes());
    n
}

fn bytes_to_hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

fn hex_to_bytes(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

fn encrypt_config(config: &AppConfig) -> Result<String> {
    let key_bytes = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)?;
    let nonce_bytes = nonce_from_time();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let plaintext = serde_json::to_vec(config)?;
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| anyhow::anyhow!("encryption failed: {}", e))?;
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(bytes_to_hex(&combined))
}

fn decrypt_config(hex: &str) -> Result<AppConfig> {
    let bytes = hex_to_bytes(hex.trim()).context("invalid hex in config")?;
    anyhow::ensure!(bytes.len() > 12, "config data too short");
    let (nonce_bytes, ciphertext) = bytes.split_at(12);
    let key_bytes = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("decryption failed: {}", e))?;
    Ok(serde_json::from_slice(&plaintext)?)
}

// --- Config path ---

fn config_path() -> Result<std::path::PathBuf> {
    let proj = ProjectDirs::from("com", "termissh", "manager")
        .context("Could not determine config directory")?;
    let dir = proj.config_dir();
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    Ok(dir.join("config.enc"))
}

fn legacy_config_path() -> Option<std::path::PathBuf> {
    let proj = ProjectDirs::from("com", "termissh", "manager")?;
    let path = proj.config_dir().join("config.json");
    if path.exists() { Some(path) } else { None }
}

// --- Public API ---

pub fn load_config() -> AppConfig {
    // 1. Try encrypted file
    if let Ok(path) = config_path() {
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(cfg) = decrypt_config(&data) {
                    return cfg;
                }
            }
        }
    }
    // 2. Migrate from legacy plain-text JSON
    if let Some(legacy) = legacy_config_path() {
        if let Ok(data) = fs::read_to_string(&legacy) {
            let cfg: AppConfig = serde_json::from_str(&data).unwrap_or_default();
            // Save encrypted version and remove legacy file
            let _ = save_config(&cfg);
            let _ = fs::remove_file(legacy);
            return cfg;
        }
    }
    AppConfig::default()
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_path()?;
    let encrypted = encrypt_config(config)?;
    fs::write(path, encrypted)?;
    Ok(())
}
