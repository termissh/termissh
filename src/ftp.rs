use crate::config::Host;
use ssh2::Session;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct FtpEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

fn open_session(host: &Host) -> Result<Session, String> {
    let addr = format!("{}:{}", host.hostname, host.port);
    let tcp = TcpStream::connect_timeout(
        &addr.parse().map_err(|e: std::net::AddrParseError| e.to_string())?,
        Duration::from_secs(10),
    )
    .map_err(|e| format!("Connection failed: {}", e))?;

    let mut sess = Session::new().map_err(|e| e.to_string())?;
    sess.set_tcp_stream(tcp);
    sess.handshake().map_err(|e| format!("Handshake failed: {}", e))?;

    let authed = sess.userauth_agent(&host.username).is_ok()
        || host
            .password
            .as_ref()
            .map(|pw| sess.userauth_password(&host.username, pw).is_ok())
            .unwrap_or(false);

    if !authed || !sess.authenticated() {
        return Err("Authentication failed".to_string());
    }
    Ok(sess)
}

pub fn list_directory(host: &Host, path: &str) -> Result<Vec<FtpEntry>, String> {
    let sess = open_session(host)?;
    let sftp = sess.sftp().map_err(|e| format!("SFTP init failed: {}", e))?;

    let entries = sftp
        .readdir(Path::new(path))
        .map_err(|e| format!("Cannot list {}: {}", path, e))?;

    let mut result: Vec<FtpEntry> = entries
        .into_iter()
        .filter_map(|(pb, stat)| {
            let name = pb.file_name()?.to_string_lossy().to_string();
            if name == "." || name == ".." {
                return None;
            }
            Some(FtpEntry {
                name,
                path: pb.to_string_lossy().replace('\\', "/"),
                is_dir: stat.is_dir(),
                size: stat.size.unwrap_or(0),
            })
        })
        .collect();

    result.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(result)
}

pub fn download_file(host: &Host, remote_path: &str, local_path: &str) -> Result<(), String> {
    let sess = open_session(host)?;
    let sftp = sess.sftp().map_err(|e| e.to_string())?;

    let mut remote = sftp
        .open(Path::new(remote_path))
        .map_err(|e| format!("Cannot open remote file: {}", e))?;

    let mut buf = Vec::new();
    remote
        .read_to_end(&mut buf)
        .map_err(|e| format!("Read error: {}", e))?;

    std::fs::write(local_path, &buf).map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

pub fn upload_file(host: &Host, local_path: &str, remote_path: &str) -> Result<(), String> {
    let sess = open_session(host)?;
    let sftp = sess.sftp().map_err(|e| e.to_string())?;

    let buf = std::fs::read(local_path).map_err(|e| format!("Cannot read file: {}", e))?;
    let size = buf.len() as u64;

    let mut remote = sftp
        .create(Path::new(remote_path))
        .map_err(|e| format!("Cannot create remote file: {}", e))?;

    remote
        .write_all(&buf)
        .map_err(|e| format!("Upload error: {}", e))?;

    drop(remote);
    // Verify size
    if let Ok(stat) = sftp.stat(Path::new(remote_path)) {
        if stat.size.unwrap_or(0) != size {
            return Err("Upload size mismatch".to_string());
        }
    }
    Ok(())
}

pub fn search_files(host: &Host, start_path: &str, query: &str) -> Result<Vec<FtpEntry>, String> {
    let sess = open_session(host)?;
    let mut channel = sess.channel_session().map_err(|e| e.to_string())?;

    // Sanitize query — allow only safe chars
    let safe_q: String = query
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '.' | '_' | '-' | '+'))
        .collect();
    if safe_q.is_empty() {
        return Ok(Vec::new());
    }

    // GNU find -printf gives type + path on each line; fall back gracefully if not supported
    let cmd = format!(
        "find {} -maxdepth 8 -name '*{}*' -printf '%y\\t%p\\n' 2>/dev/null | head -300",
        start_path, safe_q
    );
    channel.exec(&cmd).map_err(|e| e.to_string())?;

    let mut output = String::new();
    channel
        .read_to_string(&mut output)
        .map_err(|e| e.to_string())?;

    let entries: Vec<FtpEntry> = output
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, '\t');
            let kind = parts.next()?;
            let path = parts.next()?.trim();
            if path.is_empty() {
                return None;
            }
            let is_dir = kind == "d";
            let name = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string());
            Some(FtpEntry {
                name,
                path: path.to_string(),
                is_dir,
                size: 0,
            })
        })
        .collect();

    Ok(entries)
}

pub fn parent_path(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return "/".to_string();
    }
    match trimmed.rfind('/') {
        Some(0) => "/".to_string(),
        Some(pos) => trimmed[..pos].to_string(),
        None => "/".to_string(),
    }
}

pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0)
    } else {
        format!("{:.2} GB", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
    }
}
