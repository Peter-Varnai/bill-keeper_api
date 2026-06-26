use super::super::error::TestError;
use super::super::helpers::{cleanup_test_context, new_test_context};

fn to_err<E: std::fmt::Display>(e: E) -> TestError {
    TestError::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
    ))
}

async fn upload_test_bill(
    client: &reqwest::Client,
    base_url: &str,
    dg_id: i32,
) -> Result<i32, TestError> {
    let file_data = std::fs::read("tests/test_docs/table1.jpg").map_err(to_err)?;
    let filename = format!("bill_{}.jpg",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    let file_part = reqwest::multipart::Part::bytes(file_data)
        .file_name(filename)
        .mime_str("image/jpeg")
        .map_err(to_err)?;

    let resp = client
        .post(format!("{}/api/bills/upload", base_url))
        .multipart(
            reqwest::multipart::Form::new()
                .text("data_group", dg_id.to_string())
                .part("files", file_part),
        )
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(resp.status().as_u16(), 200, "upload failed");

    let result: serde_json::Value = resp.json().await.map_err(to_err)?;
    result["results"][0]["bill_id"].as_i64()
        .map(|id| id as i32)
        .ok_or_else(|| to_err("no bill id returned"))
}

pub async fn test_bills_get() -> Result<(), TestError> {
    let (client, base_url, _group_name, dg_id) = new_test_context().await?;

    upload_test_bill(&client, &base_url, dg_id).await?;
    upload_test_bill(&client, &base_url, dg_id).await?;

    let response = client
        .get(format!("{}/api/bills", base_url))
        .query(&[("data_group", &dg_id.to_string())])
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 status");

    let bills: Vec<serde_json::Value> = response.json().await.map_err(to_err)?;
    assert!(!bills.is_empty(), "expected non-empty bills");

    cleanup_test_context(&client, &base_url, dg_id).await
}

pub async fn test_bills_get_by_id() -> Result<(), TestError> {
    let (client, base_url, _group_name, dg_id) = new_test_context().await?;

    let bill_id = upload_test_bill(&client, &base_url, dg_id).await?;

    let response = client
        .get(format!("{}/api/bills/{}", base_url, bill_id))
        .query(&[("data_group", &dg_id.to_string())])
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 status");

    let bill: serde_json::Value = response.json().await.map_err(to_err)?;
    assert_eq!(bill["id"], bill_id, "expected bill id match");

    cleanup_test_context(&client, &base_url, dg_id).await
}

pub async fn test_bills_update() -> Result<(), TestError> {
    let (client, base_url, _group_name, dg_id) = new_test_context().await?;

    let bill_id = upload_test_bill(&client, &base_url, dg_id).await?;

    let update_payload = serde_json::json!({
        "id": bill_id,
        "filename": "test_bill_1.pdf",
        "amount": 999.99,
        "date": "2024-12-31",
        "is_cash": true,
        "data_group": dg_id
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

    let response = client
        .get(format!("{}/api/bills/{}", base_url, bill_id))
        .query(&[("data_group", &dg_id.to_string())])
        .send()
        .await
        .map_err(to_err)?;

    let bill: serde_json::Value = response.json().await.map_err(to_err)?;
    assert_eq!(bill["amount"], 999.99, "expected amount to be updated");

    cleanup_test_context(&client, &base_url, dg_id).await
}

pub async fn test_bills_delete() -> Result<(), TestError> {
    let (client, base_url, _group_name, dg_id) = new_test_context().await?;

    let bill_id = upload_test_bill(&client, &base_url, dg_id).await?;

    let response = client
        .delete(format!("{}/api/bills/{}", base_url, bill_id))
        .query(&[("data_group", &dg_id.to_string())])
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    let response = client
        .get(format!("{}/api/bills/{}", base_url, bill_id))
        .query(&[("data_group", &dg_id.to_string())])
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 404, "expected 404 after delete");

    cleanup_test_context(&client, &base_url, dg_id).await
}

pub async fn test_bills_upload_jpg() -> Result<(), TestError> {
    let (client, base_url, group_name, dg_id) = new_test_context().await?;

    let file_path = "tests/test_docs/table1.jpg";
    let file_data = std::fs::read(file_path).map_err(to_err)?;
    let filename = format!("upload_jpg_{}.jpg", group_name);

    let file_part = reqwest::multipart::Part::bytes(file_data)
        .file_name(filename.clone())
        .mime_str("image/jpeg")
        .map_err(to_err)?;

    let response = client
        .post(format!("{}/api/bills/upload", base_url))
        .multipart(
            reqwest::multipart::Form::new()
                .text("data_group", dg_id.to_string())
                .part("files", file_part),
        )
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    let result: serde_json::Value = response.json().await.map_err(to_err)?;
    assert_eq!(result["success"], true, "expected success");
    assert!(
        result["success_count"].as_i64().unwrap() > 0,
        "expected success_count > 0"
    );

    let storage_path = format!("./public/pdf_imgs/{}/{}", group_name, filename);
    assert!(
        std::path::Path::new(&storage_path).exists(),
        "expected file to exist in storage"
    );

    let _ = std::fs::remove_file(&storage_path);

    cleanup_test_context(&client, &base_url, dg_id).await
}

pub async fn test_bills_upload_png() -> Result<(), TestError> {
    let (client, base_url, group_name, dg_id) = new_test_context().await?;

    let file_path = "tests/test_docs/Brown-Cat-PNG.png";
    let file_data = std::fs::read(file_path).map_err(to_err)?;
    let filename = format!("upload_png_{}.png", group_name);

    let file_part = reqwest::multipart::Part::bytes(file_data)
        .file_name(filename.clone())
        .mime_str("image/png")
        .map_err(to_err)?;

    let response = client
        .post(format!("{}/api/bills/upload", base_url))
        .multipart(
            reqwest::multipart::Form::new()
                .text("data_group", dg_id.to_string())
                .part("files", file_part),
        )
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    let result: serde_json::Value = response.json().await.map_err(to_err)?;
    assert_eq!(result["success"], true, "expected success");

    let storage_path = format!("./public/pdf_imgs/{}/{}", group_name, filename);
    assert!(
        std::path::Path::new(&storage_path).exists(),
        "expected file to exist"
    );

    let _ = std::fs::remove_file(&storage_path);

    cleanup_test_context(&client, &base_url, dg_id).await
}

pub async fn test_bills_upload_pdf() -> Result<(), TestError> {
    let (client, base_url, group_name, dg_id) = new_test_context().await?;

    let file_path = "tests/test_docs/IdentificationBooklet_web.pdf";
    let file_data = std::fs::read(file_path).map_err(to_err)?;
    let pdf_name = format!("upload_pdf_{}.pdf", group_name);

    let file_part = reqwest::multipart::Part::bytes(file_data)
        .file_name(pdf_name.clone())
        .mime_str("application/pdf")
        .map_err(to_err)?;

    let response = client
        .post(format!("{}/api/bills/upload", base_url))
        .multipart(
            reqwest::multipart::Form::new()
                .text("data_group", dg_id.to_string())
                .part("files", file_part),
        )
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    let result: serde_json::Value = response.json().await.map_err(to_err)?;
    assert_eq!(result["success"], true, "expected success");

    let uploaded = result["results"][0]["filename"].as_str().unwrap();
    let storage_path = format!("./public/pdf_imgs/{}/{}", group_name, uploaded);
    assert!(
        std::path::Path::new(&storage_path).exists(),
        "expected file to exist"
    );

    let _ = std::fs::remove_file(&storage_path);

    cleanup_test_context(&client, &base_url, dg_id).await
}

pub async fn test_bills_upload_unsupported() -> Result<(), TestError> {
    let (client, base_url, _group_name, dg_id) = new_test_context().await?;

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
                .text("data_group", dg_id.to_string())
                .part("files", file_part),
        )
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    let result: serde_json::Value = response.json().await.map_err(to_err)?;
    assert!(
        result["error_count"].as_i64().unwrap() > 0,
        "expected error_count > 0"
    );

    cleanup_test_context(&client, &base_url, dg_id).await
}
