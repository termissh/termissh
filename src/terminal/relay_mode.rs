use std::env;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub const INTERNAL_RELAY_ARG: &str = "--relay-internal";

pub fn is_internal_relay_mode() -> bool {
    env::args().any(|arg| arg == INTERNAL_RELAY_ARG)
}

fn fatal(message: &str) -> ! {
    eprintln!("{}", message);
    std::process::exit(1);
}

pub fn run_from_env() {
    let host = env::var("TERMISSH_HOST").unwrap_or_else(|_| fatal("TERMISSH_HOST not set"));
    let port: u16 = env::var("TERMISSH_PORT")
        .unwrap_or_else(|_| "22".to_string())
        .parse()
        .unwrap_or(22);
    let user = env::var("TERMISSH_USER").unwrap_or_else(|_| fatal("TERMISSH_USER not set"));
    let pass = env::var("TERMISSH_PASS").unwrap_or_default();

    let tcp = match TcpStream::connect(format!("{}:{}", host, port)) {
        Ok(tcp) => tcp,
        Err(e) => fatal(&format!("Connection failed: {}", e)),
    };

    let mut sess = ssh2::Session::new().expect("Failed to create SSH session");
    sess.set_tcp_stream(tcp);
    if let Err(e) = sess.handshake() {
        fatal(&format!("SSH handshake failed: {}", e));
    }

    let mut authenticated = false;
    if sess.userauth_agent(&user).is_ok() {
        authenticated = true;
    }
    if !authenticated && !pass.is_empty() {
        if let Err(e) = sess.userauth_password(&user, &pass) {
            fatal(&format!("Password auth failed: {}", e));
        }
    } else if !authenticated {
        fatal("Authentication failed: no password and agent auth failed");
    }

    let mut channel = match sess.channel_session() {
        Ok(ch) => ch,
        Err(e) => fatal(&format!("Channel open failed: {}", e)),
    };

    let cols: u32 = env::var("COLUMNS")
        .unwrap_or_else(|_| "120".to_string())
        .parse()
        .unwrap_or(120);
    let rows: u32 = env::var("LINES")
        .unwrap_or_else(|_| "40".to_string())
        .parse()
        .unwrap_or(40);

    if let Err(e) = channel.request_pty("xterm-256color", None, Some((cols, rows, 0, 0))) {
        fatal(&format!("PTY request failed: {}", e));
    }

    if let Err(e) = channel.shell() {
        fatal(&format!("Shell request failed: {}", e));
    }

    sess.set_blocking(false);

    let channel = Arc::new(Mutex::new(channel));
    let running = Arc::new(AtomicBool::new(true));

    let ch_read = channel.clone();
    let r1 = running.clone();
    let stdout_thread = thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let stdout = io::stdout();
        while r1.load(Ordering::Relaxed) {
            let result = {
                let mut ch = ch_read.lock().unwrap();
                ch.read(&mut buf)
            };
            match result {
                Ok(0) => {
                    r1.store(false, Ordering::Relaxed);
                    break;
                }
                Ok(n) => {
                    let mut out = stdout.lock();
                    let _ = out.write_all(&buf[..n]);
                    let _ = out.flush();
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5));
                }
                Err(_) => {
                    r1.store(false, Ordering::Relaxed);
                    break;
                }
            }

            if ch_read.lock().unwrap().eof() {
                r1.store(false, Ordering::Relaxed);
                break;
            }
        }
    });

    let ch_write = channel.clone();
    let r2 = running.clone();
    let stdin_thread = thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let stdin = io::stdin();
        while r2.load(Ordering::Relaxed) {
            match stdin.lock().read(&mut buf) {
                Ok(0) => {
                    r2.store(false, Ordering::Relaxed);
                    break;
                }
                Ok(n) => {
                    let mut ch = ch_write.lock().unwrap();
                    let _ = ch.write_all(&buf[..n]);
                    let _ = ch.flush();
                }
                Err(_) => {
                    r2.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }
    });

    let _ = stdout_thread.join();
    running.store(false, Ordering::Relaxed);
    let _ = stdin_thread.join();
}
