use super::super::error::TestError;
use super::super::helpers::{start_test, stop_test};

fn to_err<E: std::fmt::Display>(e: E) -> TestError {
    TestError::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
    ))
}

pub async fn test_data_groups_get() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    let response = client
        .get(format!("{}/api/data_groups", base_url))
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 status");

    let groups: Vec<serde_json::Value> = response.json().await.map_err(to_err)?;
    assert!(!groups.is_empty(), "expected non-empty data groups");

    stop_test(srv).await
}

pub async fn test_data_groups_create() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let unique_name = format!("test_group_{}", timestamp);
    let create_payload = serde_json::json!({
        "name": unique_name,
        "type": "project"
    });

    let response = client
        .post(format!("{}/api/data_groups", base_url))
        .json(&create_payload)
        .send()
        .await
        .map_err(to_err)?;

    let status = response.status().as_u16();
    let body = response.text().await.map_err(to_err)?;

    if status != 201 {
        eprintln!("CREATE STATUS: {}", status);
        eprintln!("CREATE BODY: {:?}", body);
    }
    assert_eq!(status, 201, "expected 201 Created");

    let result: serde_json::Value = serde_json::from_str(&body).map_err(to_err)?;
    assert!(result["id"].as_i64().is_some(), "expected id in response");
    assert_eq!(result["name"], unique_name, "expected name match");

    let storage_path = format!("./public/pdf_imgs/{}", unique_name);
    assert!(
        std::path::Path::new(&storage_path).exists(),
        "expected storage directory to exist"
    );

    // Cleanup: remove directory (DB record left for cleanup)
    let _ = std::fs::remove_dir_all(&storage_path);

    stop_test(srv).await
}
