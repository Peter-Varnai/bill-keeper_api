use super::super::error::TestError;
use super::super::helpers::{start_test, stop_test};

fn to_err<E: std::fmt::Display>(e: E) -> TestError {
    TestError::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
    ))
}

pub async fn test_bills_get() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    let response = client
        .get(format!("{}/api/bills", base_url))
        .query(&[("data_group", "1")])
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 status");

    let bills: Vec<serde_json::Value> = response.json().await.map_err(to_err)?;
    assert!(!bills.is_empty(), "expected non-empty bills");

    stop_test(srv).await
}

pub async fn test_bills_get_by_id() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    // Use bill id=1 from seed data
    let response = client
        .get(format!("{}/api/bills/1", base_url))
        .query(&[("data_group", "1")])
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 status");

    let bill: serde_json::Value = response.json().await.map_err(to_err)?;
    assert_eq!(bill["id"], 1, "expected bill id 1");

    stop_test(srv).await
}

pub async fn test_bills_update() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    // Update bill id=1 with new amount
    let update_payload = serde_json::json!({
        "id": 1,
        "filename": "test_invoice_1.pdf",
        "amount": 999.99,
        "date": "2024-12-31",
        "is_cash": true,
        "data_group": 1
    });

    let response = client
        .put(format!("{}/api/bills", base_url))
        .json(&update_payload)
        .send()
        .await
        .map_err(to_err)?;

    let status = response.status().as_u16();
    let body = response.text().await.map_err(to_err)?;

    if status != 200 {
        eprintln!("UPDATE STATUS: {}", status);
        eprintln!("UPDATE BODY: {:?}", body);
    }
    assert_eq!(status, 200, "expected 200 OK");

    // Verify the update by getting the bill
    let response = client
        .get(format!("{}/api/bills/1", base_url))
        .query(&[("data_group", "1")])
        .send()
        .await
        .map_err(to_err)?;

    let bill: serde_json::Value = response.json().await.map_err(to_err)?;
    assert_eq!(bill["amount"], 999.99, "expected amount to be updated");

    // Restore original amount for other tests
    let restore_payload = serde_json::json!({
        "id": 1,
        "filename": "test_invoice_1.pdf",
        "amount": 100.00,
        "date": "2024-01-15",
        "is_cash": false,
        "data_group": 1
    });

    let _ = client
        .put(format!("{}/api/bills", base_url))
        .json(&restore_payload)
        .send()
        .await;

    stop_test(srv).await
}

pub async fn test_bills_delete() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    let response = client
        .delete(format!("{}/api/bills/2", base_url))
        .query(&[("data_group", "1")])
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    // Verify it's deleted - should return 404
    let response = client
        .get(format!("{}/api/bills/2", base_url))
        .query(&[("data_group", "1")])
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 404, "expected 404 after delete");

    stop_test(srv).await
}

pub async fn test_bills_upload_jpg() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    // Create storage directory if it doesn't exist
    std::fs::create_dir_all("./public/pdf_imgs/2024").map_err(to_err)?;

    let file_path = "tests/test_docs/table1.jpg";
    let file_data = std::fs::read(file_path).map_err(to_err)?;
    let filename = "unique_upload_table1.jpg";

    let file_part = reqwest::multipart::Part::bytes(file_data.clone())
        .file_name(filename.to_string())
        .mime_str("image/jpeg")
        .map_err(to_err)?;

    let response = client
        .post(format!("{}/api/bills/upload", base_url))
        .multipart(
            reqwest::multipart::Form::new()
                .text("data_group", "1")
                .part("files", file_part),
        )
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    let result: serde_json::Value = response.json().await.map_err(to_err)?;
    eprintln!("UPLOAD RESULT: {:?}", result);
    assert_eq!(result["success"], true, "expected success");
    assert!(
        result["success_count"].as_i64().unwrap() > 0,
        "expected success_count > 0"
    );

    // Verify file exists in storage
    let storage_path = format!("./public/pdf_imgs/2024/{}", filename);
    assert!(
        std::path::Path::new(&storage_path).exists(),
        "expected file to exist in storage"
    );

    // Cleanup
    let _ = std::fs::remove_file(storage_path);

    stop_test(srv).await
}

pub async fn test_bills_upload_png() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    // Create storage directory if it doesn't exist
    std::fs::create_dir_all("./public/pdf_imgs/2024").map_err(to_err)?;

    let file_path = "tests/test_docs/Brown-Cat-PNG.png";
    let file_data = std::fs::read(file_path).map_err(to_err)?;
    let filename = "unique_upload_cat.png";

    let file_part = reqwest::multipart::Part::bytes(file_data)
        .file_name(filename.to_string())
        .mime_str("image/png")
        .map_err(to_err)?;

    let response = client
        .post(format!("{}/api/bills/upload", base_url))
        .multipart(
            reqwest::multipart::Form::new()
                .text("data_group", "1")
                .part("files", file_part),
        )
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    let result: serde_json::Value = response.json().await.map_err(to_err)?;
    assert_eq!(result["success"], true, "expected success");

    // Verify file exists
    let storage_path = format!("./public/pdf_imgs/2024/{}", filename);
    assert!(
        std::path::Path::new(&storage_path).exists(),
        "expected file to exist"
    );

    // Cleanup
    let _ = std::fs::remove_file(storage_path);

    stop_test(srv).await
}

pub async fn test_bills_upload_pdf() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    // Create storage directory if it doesn't exist
    std::fs::create_dir_all("./public/pdf_imgs/2024").map_err(to_err)?;

    let file_path = "tests/test_docs/IdentificationBooklet_web.pdf";
    let file_data = std::fs::read(file_path).map_err(to_err)?;

    let file_part = reqwest::multipart::Part::bytes(file_data)
        .file_name("unique_test_doc.pdf")
        .mime_str("application/pdf")
        .map_err(to_err)?;

    let response = client
        .post(format!("{}/api/bills/upload", base_url))
        .multipart(
            reqwest::multipart::Form::new()
                .text("data_group", "1")
                .part("files", file_part),
        )
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    let result: serde_json::Value = response.json().await.map_err(to_err)?;
    assert_eq!(result["success"], true, "expected success");

    let uploaded = &result["results"][0]["filename"].as_str().unwrap();
    let storage_path = format!("./public/pdf_imgs/2024/{}", uploaded);
    assert!(
        std::path::Path::new(&storage_path).exists(),
        "expected file to exist"
    );

    // Cleanup
    let _ = std::fs::remove_file(storage_path);

    stop_test(srv).await
}

pub async fn test_bills_upload_unsupported() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    let file_path = "tests/test_docs/cats.webp";
    let file_data = std::fs::read(file_path).map_err(to_err)?;

    let file_part = reqwest::multipart::Part::bytes(file_data)
        .file_name("test.webp")
        .mime_str("image/webp")
        .map_err(to_err)?;

    let response = client
        .post(format!("{}/api/bills/upload", base_url))
        .multipart(
            reqwest::multipart::Form::new()
                .text("data_group", "1")
                .part("files", file_part),
        )
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    let result: serde_json::Value = response.json().await.map_err(to_err)?;
    // Should return error for unsupported format
    assert!(
        result["error_count"].as_i64().unwrap() > 0,
        "expected error_count > 0"
    );

    stop_test(srv).await
}
