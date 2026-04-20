use super::super::error::TestError;
use super::super::helpers::{start_test, stop_test};

fn to_err<E: std::fmt::Display>(e: E) -> TestError {
    TestError::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
    ))
}

pub async fn test_expenses_get() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    let response = client
        .get(format!("{}/api/expenses", base_url))
        .query(&[("data_group", "1")])
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 status");

    let expenses: Vec<serde_json::Value> = response.json().await.map_err(to_err)?;
    assert!(!expenses.is_empty(), "expected non-empty expenses");

    stop_test(srv).await
}

pub async fn test_expenses_create() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    let payload = serde_json::json!({
        "partner": "Test Partner Create",
        "amount": "150.50",
        "date": "2024-06-15",
        "expense_type": 1,
        "bill": 1,
        "data_group": 1
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

    stop_test(srv).await
}

pub async fn test_expenses_update_bill() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    let payload = serde_json::json!({
        "partner": "Test Update Bill",
        "amount": "100.00",
        "date": "2024-06-15",
        "expense_type": 1,
        "bill": 1,
        "data_group": 1
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
        .patch(format!("{}/api/expenses/{}/bill", base_url, expense_id))
        .query(&[("data_group", "1")])
        .json(&update_payload)
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    stop_test(srv).await
}

pub async fn test_expenses_update_type() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    let payload = serde_json::json!({
        "partner": "Test Update Type",
        "amount": "200.00",
        "date": "2024-06-15",
        "expense_type": 1,
        "bill": 1,
        "data_group": 1
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
        .query(&[("data_group", "1")])
        .json(&update_payload)
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    stop_test(srv).await
}

pub async fn test_expenses_update_application() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    let payload = serde_json::json!({
        "partner": "Test Update App",
        "amount": "300.00",
        "date": "2024-06-15",
        "expense_type": 1,
        "bill": 1,
        "data_group": 1
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
        .patch(format!(
            "{}/api/expenses/{}/application",
            base_url, expense_id
        ))
        .query(&[("data_group", "1")])
        .json(&update_payload)
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    stop_test(srv).await
}

pub async fn test_expenses_update_cash() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    let payload = serde_json::json!({
        "partner": "Test Update Cash",
        "amount": "400.00",
        "date": "2024-06-15",
        "expense_type": 1,
        "bill": 1,
        "data_group": 1
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
        .query(&[("data_group", "1")])
        .json(&update_payload)
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    stop_test(srv).await
}

pub async fn test_expenses_delete() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    let payload = serde_json::json!({
        "partner": "Test Delete",
        "amount": "500.00",
        "date": "2024-06-15",
        "expense_type": 1,
        "bill": 1,
        "data_group": 1
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
        .query(&[("data_group", "1")])
        .send()
        .await
        .map_err(to_err)?;

    assert_eq!(response.status().as_u16(), 200, "expected 200 OK");

    stop_test(srv).await
}

pub async fn test_expenses_bulk_import() -> Result<(), TestError> {
    let (client, base_url, srv) = start_test().await?;

    let payload = serde_json::json!({
        "date_format": "auto-detect",
        "rows": [
            { "partner": "Bulk Unique 1", "amount": "100.50", "date": "2024-01-15", "row_number": 1 },
            { "partner": "Bulk Unique 2", "amount": "200.75", "date": "2024-01-16", "row_number": 2 },
            { "partner": "Bulk Unique 3", "amount": "300.25", "date": "2024-01-17", "row_number": 3 }
        ],
        "data_group": 1
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

    stop_test(srv).await
}
