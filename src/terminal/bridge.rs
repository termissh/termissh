use crate::config::Host;
use crate::terminal::relay_mode::INTERNAL_RELAY_ARG;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Resolve relay launcher path (single-binary mode uses current executable).
pub fn find_relay_binary() -> Result<String> {
    let exe = std::env::current_exe().context("Cannot get current executable path")?;
    Ok(exe.to_string_lossy().to_string())
}

/// Build environment variables for relay mode.
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
    cmd.arg(INTERNAL_RELAY_ARG)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    for (key, value) in build_relay_env(host) {
        cmd.env(key, value);
    }

    cmd.spawn()
        .with_context(|| format!("Failed to launch internal relay process: {}", relay_path))
}
