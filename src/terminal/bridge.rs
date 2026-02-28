use crate::config::Host;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

fn relay_name() -> &'static str {
    if cfg!(windows) {
        "termissh-relay.exe"
    } else {
        "termissh-relay"
    }
}

fn push_unique_path(paths: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !paths.iter().any(|existing| existing == &candidate) {
        paths.push(candidate);
    }
}

fn collect_candidates(relay_name: &str, exe: Option<&Path>, cwd: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(exe) = exe {
        if let Some(exe_dir) = exe.parent() {
            push_unique_path(&mut candidates, exe_dir.join(relay_name));

            // IDE/debug runs can use target/debug/deps/<app>.exe
            if exe_dir.file_name().and_then(|n| n.to_str()) == Some("deps") {
                if let Some(parent) = exe_dir.parent() {
                    push_unique_path(&mut candidates, parent.join(relay_name));
                }
            }
        }
    }

    if let Some(cwd) = cwd {
        push_unique_path(&mut candidates, cwd.join(relay_name));
        push_unique_path(
            &mut candidates,
            cwd.join("target").join("debug").join(relay_name),
        );
        push_unique_path(
            &mut candidates,
            cwd.join("target").join("release").join(relay_name),
        );
        push_unique_path(&mut candidates, cwd.join("dist").join(relay_name));
    }

    candidates
}

fn first_existing(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates
        .iter()
        .find(|path| path.is_file())
        .cloned()
}

fn build_profile_from_exe(exe: Option<&Path>) -> &'static str {
    if let Some(exe) = exe {
        if exe
            .components()
            .any(|component| component.as_os_str() == OsStr::new("release"))
        {
            return "release";
        }
    }

    "debug"
}

fn detect_project_roots(exe: Option<&Path>, cwd: Option<&Path>) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(manifest_dir);
        if manifest_path.join("Cargo.toml").is_file() {
            push_unique_path(&mut roots, manifest_path);
        }
    }

    if let Some(cwd) = cwd {
        if cwd.join("Cargo.toml").is_file() {
            push_unique_path(&mut roots, cwd.to_path_buf());
        }
    }

    if let Some(exe) = exe {
        for ancestor in exe.ancestors() {
            if ancestor.join("Cargo.toml").is_file() {
                push_unique_path(&mut roots, ancestor.to_path_buf());
            }
        }
    }

    roots
}

fn build_relay(project_root: &Path, profile: &str) -> std::result::Result<(), String> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--bin").arg("termissh-relay");

    if profile == "release" {
        cmd.arg("--release");
    }

    cmd.current_dir(project_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    match cmd.output() {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let details = if !stderr.is_empty() {
                stderr
            } else if !stdout.is_empty() {
                stdout
            } else {
                format!("cargo exited with status {}", output.status)
            };
            Err(details)
        }
        Err(err) => Err(err.to_string()),
    }
}

/// Find the termissh-relay binary next to the main binary
pub fn find_relay_binary() -> Result<String> {
    let relay_name = relay_name();
    let exe = std::env::current_exe()
        .context("Cannot get current executable path")
        .ok();
    let cwd = std::env::current_dir().ok();
    let mut candidates = collect_candidates(relay_name, exe.as_deref(), cwd.as_deref());

    if let Some(path) = first_existing(&candidates) {
        return Ok(path.to_string_lossy().to_string());
    }

    // Dev fallback: when running only the GUI target, build relay on demand.
    let profile = build_profile_from_exe(exe.as_deref());
    let mut build_attempts = Vec::new();
    for project_root in detect_project_roots(exe.as_deref(), cwd.as_deref()) {
        match build_relay(&project_root, profile) {
            Ok(()) => {
                push_unique_path(
                    &mut candidates,
                    project_root.join("target").join(profile).join(relay_name),
                );
                if let Some(path) = first_existing(&candidates) {
                    return Ok(path.to_string_lossy().to_string());
                }
            }
            Err(err) => {
                build_attempts.push(format!("{} => {}", project_root.display(), err));
            }
        }
    }

    let searched = candidates
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    if build_attempts.is_empty() {
        Err(anyhow::anyhow!(
            "termissh-relay binary not found. Searched: {}",
            searched
        ))
    } else {
        Err(anyhow::anyhow!(
            "termissh-relay binary not found. Searched: {}. Auto-build attempts failed: {}",
            searched,
            build_attempts.join(" | ")
        ))
    }
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

    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    for (key, value) in build_relay_env(host) {
        cmd.env(key, value);
    }

    cmd.spawn()
        .with_context(|| format!("Failed to launch relay process: {}", relay_path))
}
