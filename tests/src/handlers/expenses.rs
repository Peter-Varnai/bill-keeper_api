use std::time::Duration;
use tokio::time::sleep;

use super::super::config::TEST_PORT;
use super::super::db;
use super::super::server;

pub async fn test_get_expenses() {
    db::setup().await.expect("failed to setup database");
    
    let mut srv = server::start().expect("failed to start server");
    
    sleep(Duration::from_millis(500)).await;
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://localhost:{}/api/expenses", TEST_PORT))
        .query(&[("data_group", "1")])
        .send()
        .await
        .expect("failed to send request");
    
    assert_eq!(response.status(), 200, "expected 200 status");
    
    let expenses: Vec<serde_json::Value> = response.json().await.expect("failed to parse JSON");
    assert!(!expenses.is_empty(), "expected non-empty expenses");
    
    srv.kill().expect("failed to kill server");
    
    db::teardown().await.expect("failed to teardown database");
}