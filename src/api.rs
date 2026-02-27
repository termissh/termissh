use crate::config::Host;
use anyhow::{Context, Result};

pub fn fetch_from_api(api_url: &str, api_key: &str) -> Result<Vec<Host>> {
    let url = format!("{}/api/cli/ssh", api_url);
    let resp = ureq::get(&url)
        .set("Authorization", &format!("Bearer {}", api_key))
        .call()
        .context("API fetch failed")?;

    let json: serde_json::Value = resp.into_json()?;
    let connections = json
        .get("connections")
        .and_then(|c| c.as_array())
        .cloned()
        .unwrap_or_default();

    let hosts: Vec<Host> = connections
        .into_iter()
        .filter_map(|c| {
            Some(Host {
                id: c.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()),
                alias: c.get("name").and_then(|v| v.as_str())?.to_string(),
                hostname: c.get("host").and_then(|v| v.as_str())?.to_string(),
                port: c
                    .get("port")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(22) as u16,
                username: c.get("username").and_then(|v| v.as_str())?.to_string(),
                password: c
                    .get("password")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            })
        })
        .collect();

    Ok(hosts)
}

pub fn create_on_api(api_url: &str, api_key: &str, host: &Host) -> Result<String> {
    let url = format!("{}/api/cli/ssh", api_url);
    let body = serde_json::json!({
        "name": host.alias,
        "host": host.hostname,
        "port": host.port,
        "username": host.username,
        "password": host.password.clone().unwrap_or_default(),
    });

    let resp = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(body)
        .context("API create failed")?;

    let json: serde_json::Value = resp.into_json()?;
    let id = json
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok(id)
}

pub fn update_on_api(api_url: &str, api_key: &str, host: &Host) -> Result<()> {
    let id = host.id.as_deref().unwrap_or("");
    let url = format!("{}/api/cli/ssh/{}", api_url, id);
    let body = serde_json::json!({
        "name": host.alias,
        "host": host.hostname,
        "port": host.port,
        "username": host.username,
        "password": host.password.clone().unwrap_or_default(),
    });

    ureq::put(&url)
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(body)
        .context("API update failed")?;

    Ok(())
}

pub fn delete_on_api(api_url: &str, api_key: &str, id: &str) -> Result<()> {
    let url = format!("{}/api/cli/ssh/{}", api_url, id);
    ureq::delete(&url)
        .set("Authorization", &format!("Bearer {}", api_key))
        .call()
        .context("API delete failed")?;
    Ok(())
}
