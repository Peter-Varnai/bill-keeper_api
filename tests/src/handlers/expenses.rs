use super::super::error::TestError;
use super::super::helpers::{cleanup_test_context, new_test_context};

fn to_err<E: std::fmt::Display>(e: E) -> TestError {
    TestError::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
    ))
}

async fn create_test_bill(
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
        .file_name(filename.clone())
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

    let status = resp.status().as_u16();
    if status != 200 {
        let body = resp.text().await.unwrap_or_default();
        return Err(to_err(format!(
            "create_test_bill failed (status {}): {}", status, body
        )));
    }

    let result: serde_json::Value = resp.json().await.map_err(to_err)?;
    let bill_ids: Vec<i32> = result["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["bill_id"].as_i64().unwrap() as i32)
        .collect();

    bill_ids.first().copied().ok_or_else(|| {
        to_err("no bill id returned from upload")
    })
}

async fn create_test_app_report(
    client: &reqwest::Client,
    base_url: &str,
    dg_id: i32,
) -> Result<i32, TestError> {
    let resp = client
        .post(format!("{}/api/application_reports", base_url))
        .json(&serde_json::json!({
            "name": "Test App",
            "amount": 500.00,
            "data_group": dg_id
        }))
        .send()
        .await
        .map_err(to_err)?;
    if resp.status().as_u16() != 201 {
        let body = resp.text().await.unwrap_or_default();
        return Err(to_err(format!("create app report failed: {}", body)));
    }
    let created: serde_json::Value = resp.json().await.map_err(to_err)?;
    Ok(created["id"].as_i64().unwrap() as i32)
}

pub async fn test_expenses_get() -> Result<(), TestError> {
    let (client, base_url, _group_name, dg_id) = new_test_context().await?;

    let bill_id = create_test_bill(&client, &base_url, dg_id).await?;

    client
        .post(format!("{}/api/expenses", base_url))
        .json(&serde_json::json!({
            "partner": "Test Get 1",
            "amount": "100.00",
            "date": "2024-06-15",
            "expense_type": 1,
            "bill": bill_id,
            "data_group": dg_id
        }))
        .send()
        .await
        .map_err(to_err)?;

    let response = client
        .get(format!("{}/api/expenses", base_url))
        .query(&[("data_group", &dg_id.to_string())])
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 status");

    let expenses: Vec<serde_json::Value> = response.json().await.map_err(to_err)?;
    assert!(!expenses.is_empty(), "expected non-empty expenses");

    cleanup_test_context(&client, &base_url, dg_id).await
}

pub async fn test_expenses_create() -> Result<(), TestError> {
    let (client, base_url, _group_name, dg_id) = new_test_context().await?;

    let bill_id = create_test_bill(&client, &base_url, dg_id).await?;

    let payload = serde_json::json!({
        "partner": "Test Partner Create",
        "amount": "150.50",
        "date": "2024-06-15",
        "expense_type": 1,
        "bill": bill_id,
        "data_group": dg_id
    });

    let response = client
        .post(format!("{}/api/expenses", base_url))
        .json(&payload)
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

    let created: serde_json::Value = serde_json::from_str(&body).map_err(to_err)?;
    assert_eq!(created["partner"], "Test Partner Create");

    cleanup_test_context(&client, &base_url, dg_id).await
}

pub async fn test_expenses_update_bill() -> Result<(), TestError> {
    let (client, base_url, _group_name, dg_id) = new_test_context().await?;

    let bill_id_1 = create_test_bill(&client, &base_url, dg_id).await?;
    let bill_id_2 = create_test_bill(&client, &base_url, dg_id).await?;

    let payload = serde_json::json!({
        "partner": "Test Update Bill",
        "amount": "100.00",
        "date": "2024-06-15",
        "expense_type": 1,
        "bill": bill_id_1,
        "data_group": dg_id
    });

    let response = client
        .post(format!("{}/api/expenses", base_url))
        .json(&payload)
        .send()
        .await
        .map_err(to_err)?;

    let body = response.text().await.map_err(to_err)?;
    let created: serde_json::Value = serde_json::from_str(&body).map_err(to_err)?;
    let expense_id = created["id"].as_i64().unwrap() as i32;

    let update_payload = serde_json::json!({
        "expense_id": expense_id,
        "new_number": bill_id_2
    });

    let response = client
        .patch(format!("{}/api/expenses/{}/bill", base_url, expense_id))
        .query(&[("data_group", &dg_id.to_string())])
        .json(&update_payload)
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    cleanup_test_context(&client, &base_url, dg_id).await
}

pub async fn test_expenses_update_type() -> Result<(), TestError> {
    let (client, base_url, _group_name, dg_id) = new_test_context().await?;

    let bill_id = create_test_bill(&client, &base_url, dg_id).await?;

    let payload = serde_json::json!({
        "partner": "Test Update Type",
        "amount": "200.00",
        "date": "2024-06-15",
        "expense_type": 1,
        "bill": bill_id,
        "data_group": dg_id
    });

    let response = client
        .post(format!("{}/api/expenses", base_url))
        .json(&payload)
        .send()
        .await
        .map_err(to_err)?;

    let body = response.text().await.map_err(to_err)?;
    let created: serde_json::Value = serde_json::from_str(&body).map_err(to_err)?;
    let expense_id = created["id"].as_i64().unwrap() as i32;

    let update_payload = serde_json::json!({
        "expense_id": expense_id,
        "new_number": 2
    });

    let response = client
        .patch(format!("{}/api/expenses/{}/type", base_url, expense_id))
        .query(&[("data_group", &dg_id.to_string())])
        .json(&update_payload)
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    cleanup_test_context(&client, &base_url, dg_id).await
}

pub async fn test_expenses_update_application() -> Result<(), TestError> {
    let (client, base_url, _group_name, dg_id) = new_test_context().await?;

    let bill_id = create_test_bill(&client, &base_url, dg_id).await?;
    let app_id = create_test_app_report(&client, &base_url, dg_id).await?;

    let payload = serde_json::json!({
        "partner": "Test Update App",
        "amount": "300.00",
        "date": "2024-06-15",
        "expense_type": 1,
        "bill": bill_id,
        "data_group": dg_id
    });

    let response = client
        .post(format!("{}/api/expenses", base_url))
        .json(&payload)
        .send()
        .await
        .map_err(to_err)?;

    let body = response.text().await.map_err(to_err)?;
    let created: serde_json::Value = serde_json::from_str(&body).map_err(to_err)?;
    let expense_id = created["id"].as_i64().unwrap() as i32;

    let update_payload = serde_json::json!({
        "expense_id": expense_id,
        "new_number": app_id
    });

    let response = client
        .patch(format!(
            "{}/api/expenses/{}/application",
            base_url, expense_id
        ))
        .query(&[("data_group", &dg_id.to_string())])
        .json(&update_payload)
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    cleanup_test_context(&client, &base_url, dg_id).await
}

pub async fn test_expenses_update_cash() -> Result<(), TestError> {
    let (client, base_url, _group_name, dg_id) = new_test_context().await?;

    let bill_id = create_test_bill(&client, &base_url, dg_id).await?;

    let payload = serde_json::json!({
        "partner": "Test Update Cash",
        "amount": "400.00",
        "date": "2024-06-15",
        "expense_type": 1,
        "bill": bill_id,
        "data_group": dg_id
    });

    let response = client
        .post(format!("{}/api/expenses", base_url))
        .json(&payload)
        .send()
        .await
        .map_err(to_err)?;

    let body = response.text().await.map_err(to_err)?;
    let created: serde_json::Value = serde_json::from_str(&body).map_err(to_err)?;
    let expense_id = created["id"].as_i64().unwrap() as i32;

    let update_payload = serde_json::json!({
        "expense_id": expense_id,
        "new_number": 1
    });

    let response = client
        .patch(format!("{}/api/expenses/{}/cash", base_url, expense_id))
        .query(&[("data_group", &dg_id.to_string())])
        .json(&update_payload)
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    cleanup_test_context(&client, &base_url, dg_id).await
}

pub async fn test_expenses_delete() -> Result<(), TestError> {
    let (client, base_url, _group_name, dg_id) = new_test_context().await?;

    let bill_id = create_test_bill(&client, &base_url, dg_id).await?;

    let payload = serde_json::json!({
        "partner": "Test Delete",
        "amount": "500.00",
        "date": "2024-06-15",
        "expense_type": 1,
        "bill": bill_id,
        "data_group": dg_id
    });

    let response = client
        .post(format!("{}/api/expenses", base_url))
        .json(&payload)
        .send()
        .await
        .map_err(to_err)?;

    let body = response.text().await.map_err(to_err)?;
    let created: serde_json::Value = serde_json::from_str(&body).map_err(to_err)?;
    let expense_id = created["id"].as_i64().unwrap() as i32;

    let response = client
        .delete(format!("{}/api/expenses/{}", base_url, expense_id))
        .query(&[("data_group", &dg_id.to_string())])
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    cleanup_test_context(&client, &base_url, dg_id).await
}

pub async fn test_expenses_bulk_import() -> Result<(), TestError> {
    let (client, base_url, _group_name, dg_id) = new_test_context().await?;

    let payload = serde_json::json!({
        "date_format": "auto-detect",
        "rows": [
            { "partner": "Bulk Unique 1", "amount": "100.50", "date": "2024-01-15", "row_number": 1 },
            { "partner": "Bulk Unique 2", "amount": "200.75", "date": "2024-01-16", "row_number": 2 },
            { "partner": "Bulk Unique 3", "amount": "300.25", "date": "2024-01-17", "row_number": 3 }
        ],
        "data_group": dg_id
    });

    let response = client
        .post(format!("{}/api/expenses/bulk", base_url))
        .json(&payload)
        .send()
        .await
        .map_err(to_err)?;

    let status = response.status().as_u16();
    let body = response.text().await.map_err(to_err)?;

    eprintln!("BULK STATUS: {}", status);
    eprintln!("BULK BODY: {:?}", body);

    assert_eq!(status, 200, "expected 200 OK");

    let result: serde_json::Value = serde_json::from_str(&body).map_err(to_err)?;
    assert_eq!(result["inserted"], 3, "expected 3 inserted");

    cleanup_test_context(&client, &base_url, dg_id).await
}
