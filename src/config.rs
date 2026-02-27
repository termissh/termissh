use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;

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

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Language {
    Turkish,
    English,
}

impl Default for Language {
    fn default() -> Self {
        Language::Turkish
    }
}

fn default_dark_mode() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub hosts: Vec<Host>,
    pub api_key: Option<String>,
    #[serde(default)]
    pub language: Language,
    #[serde(default = "default_dark_mode")]
    pub dark_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hosts: Vec::new(),
            api_key: None,
            language: Language::default(),
            dark_mode: default_dark_mode(),
        }
    }
}

fn config_path() -> Result<std::path::PathBuf> {
    let proj = ProjectDirs::from("com", "termissh", "manager")
        .context("Could not determine config directory")?;
    let dir = proj.config_dir();
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    Ok(dir.join("config.json"))
}

pub fn load_config() -> AppConfig {
    match config_path() {
        Ok(path) => {
            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
                    Err(_) => AppConfig::default(),
                }
            } else {
                AppConfig::default()
            }
        }
        Err(_) => AppConfig::default(),
    }
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_path()?;
    let data = serde_json::to_string_pretty(config)?;
    fs::write(path, data)?;
    Ok(())
}
