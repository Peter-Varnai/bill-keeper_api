use std::process::Child;

use super::config::TEST_PORT;
use super::db;
use super::error::TestError;
use super::server;

pub async fn start_test() -> Result<(reqwest::Client, String, Child), TestError> {
    db::setup().await?;
    let srv = server::start()?;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let client = reqwest::Client::new();
    let base_url = format!("http://localhost:{}", TEST_PORT);
    Ok((client, base_url, srv))
}

pub async fn stop_test(mut srv: Child) -> Result<(), TestError> {
    srv.kill().ok();
    db::teardown().await?;
    Ok(())
}
