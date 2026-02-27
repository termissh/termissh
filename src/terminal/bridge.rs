use crate::config::Host;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

/// Find the termissh-relay binary next to the main binary
pub fn find_relay_binary() -> Result<String> {
    let relay_name = if cfg!(windows) {
        "termissh-relay.exe"
    } else {
        "termissh-relay"
    };
    let mut candidates: Vec<PathBuf> = Vec::new();

    let exe = std::env::current_exe().context("Cannot get current executable path")?;
    if let Some(exe_dir) = exe.parent() {
        candidates.push(exe_dir.join(relay_name));

        // IDE/debug runs can use target/debug/deps/<app>.exe
        if exe_dir.file_name().and_then(|n| n.to_str()) == Some("deps") {
            if let Some(parent) = exe_dir.parent() {
                candidates.push(parent.join(relay_name));
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join(relay_name));
        candidates.push(cwd.join("target").join("debug").join(relay_name));
        candidates.push(cwd.join("target").join("release").join(relay_name));
        candidates.push(cwd.join("dist").join(relay_name));
    }

    for path in &candidates {
        if path.exists() {
            return Ok(path.to_string_lossy().to_string());
        }
    }

    let searched = candidates
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Err(anyhow::anyhow!(
        "termissh-relay binary not found. Searched: {}",
        searched
    ))
}

/// Build environment variables for the relay binary
pub fn build_relay_env(host: &Host) -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("TERMISSH_HOST".to_string(), host.hostname.clone());
    env.insert("TERMISSH_PORT".to_string(), host.port.to_string());
    env.insert("TERMISSH_USER".to_string(), host.username.clone());
    env.insert(
        "TERMISSH_PASS".to_string(),
        host.password.clone().unwrap_or_default(),
    );
    env.insert("TERM".to_string(), "xterm-256color".to_string());
    env.insert("COLUMNS".to_string(), "132".to_string());
    env.insert("LINES".to_string(), "40".to_string());
    env
}

pub fn spawn_relay_child(relay_path: &str, host: &Host) -> Result<Child> {
    let mut cmd = Command::new(relay_path);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for (key, value) in build_relay_env(host) {
        cmd.env(key, value);
    }

    cmd.spawn()
        .with_context(|| format!("Failed to launch relay process: {}", relay_path))
}
