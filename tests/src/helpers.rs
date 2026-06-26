use std::process::Child;

use super::config::allocate_port;
use super::db;
use super::error::TestError;
use super::server;

pub async fn start_test() -> Result<(reqwest::Client, String, Child), TestError> {
    let port = allocate_port();
    db::setup().await?;
    let srv = server::start(port)?;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let base_url = format!("http://localhost:{}", port);

    let login_resp = reqwest::Client::new()
        .post(format!("{}/api/auth/login", base_url))
        .json(&serde_json::json!({"username": "test", "password": "test"}))
        .send()
        .await
        .map_err(|e| TestError::Io(std::io::Error::other(e.to_string())))?;

    let auth: serde_json::Value = login_resp
        .json()
        .await
        .map_err(|e| TestError::Io(std::io::Error::other(e.to_string())))?;

    let token = auth["token"]
        .as_str()
        .ok_or_else(|| TestError::Io(std::io::Error::other("no token in login response")))?
        .to_string();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
            .map_err(|e| TestError::Io(std::io::Error::other(e.to_string())))?,
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| TestError::Io(std::io::Error::other(e.to_string())))?;

    Ok((client, base_url, srv))
}

pub async fn stop_test(mut srv: Child) -> Result<(), TestError> {
    srv.kill().ok();
    srv.wait().ok();
    db::teardown().await?;
    Ok(())
}
