use crate::db::DbPool;
use crate::helpers::parse_date_or_panic;
use crate::models::{BillNumberUpdate, Expense};
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::collections::HashMap;

#[get("/expenses")]
pub async fn get_expenses(
    pool: web::Data<DbPool>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let group_id = query
        .get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);

    let client = match pool.pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }));
        }
    };

    let result = client
        .query(
            "SELECT id, data_group, date, partner, amount::text, expense_type, bill, application, is_cash 
             FROM expenses WHERE data_group = $1 ORDER BY id",
            &[&group_id],
        )
        .await;

    match result {
        Ok(rows) => {
            let expenses: Vec<Expense> = rows
                .iter()
                .map(|row| Expense {
                    id: row.get(0),
                    data_group: row.get(1),
                    date: row.get(2),
                    partner: row.get(3),
                    amount: row.get::<_, String>(4).parse().unwrap_or(0.0),
                    expense_type: row.get(5),
                    bill: row.get(6),
                    application: row.get(7),
                    is_cash: row.get(8),
                })
                .collect();
            HttpResponse::Ok().json(expenses)
        }
        Err(e) => {
            crate::db::log_db_error("get_expenses", &e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch expenses: {}", e)
            }))
        }
    }
}

#[patch("/expenses/{id}/bill")]
pub async fn update_expense_bill(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    params: web::Json<BillNumberUpdate>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let id = path.into_inner();
    let group_id = query
        .get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);

    let client = match pool.pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }));
        }
    };

    let result = client
        .execute(
            "UPDATE expenses SET bill = $1 WHERE id = $2 AND data_group = $3",
            &[&params.new_number, &id, &group_id],
        )
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Updated bill number to {} for expense {}", params.new_number, id)
        })),
        Err(e) => {
            crate::db::log_db_error("update_expense_bill", &e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to update bill number: {}", e)
            }))
        }
    }
}

#[patch("/expenses/{id}/type")]
pub async fn update_expense_type(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    params: web::Json<BillNumberUpdate>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let id = path.into_inner();
    let group_id = query
        .get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);

    let client = match pool.pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }));
        }
    };

    let result = client
        .execute(
            "UPDATE expenses SET expense_type = $1 WHERE id = $2 AND data_group = $3",
            &[&params.new_number, &id, &group_id],
        )
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Updated expense type to {} for expense {}", params.new_number, id)
        })),
        Err(e) => {
            crate::db::log_db_error("update_expense_type", &e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to update expense type: {}", e)
            }))
        }
    }
}

#[patch("/expenses/{id}/application")]
pub async fn update_expense_application(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    params: web::Json<BillNumberUpdate>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let id = path.into_inner();
    let group_id = query
        .get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);

    let client = match pool.pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }));
        }
    };

    let result = client
        .execute(
            "UPDATE expenses SET application = $1 WHERE id = $2 AND data_group = $3",
            &[&params.new_number, &id, &group_id],
        )
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Updated application to {} for expense {}", params.new_number, id)
        })),
        Err(e) => {
            crate::db::log_db_error("update_expense_application", &e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to update application: {}", e)
            }))
        }
    }
}

#[patch("/expenses/{id}/cash")]
pub async fn update_expense_cash(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    params: web::Json<BillNumberUpdate>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let id = path.into_inner();
    let group_id = query
        .get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);

    let is_cash = params.new_number != 0;

    let client = match pool.pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }));
        }
    };

    let result = client
        .execute(
            "UPDATE expenses SET is_cash = $1 WHERE id = $2 AND data_group = $3",
            &[&is_cash, &id, &group_id],
        )
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Updated cash status to {} for expense {}", is_cash, id)
        })),
        Err(e) => {
            crate::db::log_db_error("update_expense_cash", &e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to update cash status: {}", e)
            }))
        }
    }
}

#[derive(Deserialize)]
pub struct CreateExpenseRequest {
    partner: String,
    amount: String,
    date: Option<String>,
    expense_type: Option<i32>,
    bill: Option<i32>,
    application: Option<i32>,
    is_cash: Option<bool>,
    group_id: Option<i32>,
}

#[post("/expenses")]
pub async fn create_expense(
    pool: web::Data<DbPool>,
    data: web::Json<CreateExpenseRequest>,
) -> impl Responder {
    let group_id = data.group_id.unwrap_or(1);

    if data.partner.trim().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Partner field is required"
        }));
    }

    let amount_normalized = normalize_amount(&data.amount);
    if amount_normalized.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Invalid amount format: {}", data.amount)
        }));
    }

    let amount_val = amount_normalized.unwrap();

    let client = match pool.pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }));
        }
    };

    let result = client
        .query(
            "INSERT INTO expenses (data_group, date, partner, amount, expense_type, bill, application, is_cash) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
            &[
                &group_id,
                &data.date,
                &data.partner.trim(),
                &amount_val,
                &data.expense_type.unwrap_or(0),
                &data.bill,
                &data.application,
                &data.is_cash.unwrap_or(false),
            ],
        )
        .await;

    // let parsed_date = parse_date_or_panic(&data.date);

    match result {
        Ok(rows) => {
            let new_id = rows.first().map(|r| r.get::<_, i32>(0)).unwrap_or(0);
            HttpResponse::Created().json(Expense {
                id: new_id,
                data_group: group_id,
                date: parse_date_or_panic(data.date.clone()),
                partner: data.partner.clone(),
                amount: amount_val,
                expense_type: data.expense_type.unwrap_or(0),
                bill: data.bill,
                application: data.application,
                is_cash: data.is_cash,
            })
        }
        Err(e) => {
            crate::db::log_db_error("create_expense", &e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create expense: {}", e)
            }))
        }
    }
}

fn normalize_amount(value: &str) -> Option<f64> {
    let clean = value.trim();
    if clean.is_empty() {
        return None;
    }

    let last_comma = clean.rfind(',');
    let last_dot = clean.rfind('.');

    let result = match (last_comma, last_dot) {
        (Some(comma_pos), Some(dot_pos)) => {
            if comma_pos > dot_pos {
                clean.replace(".", "").replace(",", ".")
            } else {
                clean.replace(",", "")
            }
        }
        (Some(_), None) => {
            if let Some(comma_pos) = last_comma {
                let after_comma = clean.len() - comma_pos - 1;
                if after_comma <= 2 {
                    clean.replace(",", ".")
                } else {
                    clean.replace(",", "")
                }
            } else {
                clean.to_string()
            }
        }
        (None, Some(_)) => clean.to_string(),
        (None, None) => clean.to_string(),
    };

    result.parse::<f64>().ok()
}

#[derive(Deserialize)]
pub struct CheckDuplicatesRequest {
    expenses: Vec<DuplicateCheckItem>,
    group_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct DuplicateCheckItem {
    partner: String,
    amount: String,
    date: Option<String>,
}

#[derive(serde::Serialize)]
pub struct DuplicateCheckResult {
    index: usize,
    existing: Expense,
}

#[post("/expenses/check-duplicates")]
pub async fn check_duplicates(
    pool: web::Data<DbPool>,
    data: web::Json<CheckDuplicatesRequest>,
) -> impl Responder {
    let group_id = data.group_id.unwrap_or(1);

    let client = match pool.pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }));
        }
    };

    let mut duplicates = Vec::new();

    for (index, item) in data.expenses.iter().enumerate() {
        if let Some(amount) = normalize_amount(&item.amount) {
            let result = client
                .query_opt(
                    "SELECT id, data_group, date, partner, amount::text, expense_type, bill, application, is_cash 
                     FROM expenses 
                     WHERE partner = $1 AND amount::text = $2 AND data_group = $3 
                     LIMIT 1",
                    &[&item.partner, &format!("{:.2}", amount), &group_id],
                )
                .await;

            if let Ok(Some(row)) = result {
                duplicates.push(DuplicateCheckResult {
                    index,
                    existing: Expense {
                        id: row.get(0),
                        data_group: row.get(1),
                        date: row.get(2),
                        partner: row.get(3),
                        amount: row.get::<_, String>(4).parse().unwrap_or(0.0),
                        expense_type: row.get(5),
                        bill: row.get(6),
                        application: row.get(7),
                        is_cash: row.get(8),
                    },
                });
            }
        }
    }

    HttpResponse::Ok().json(duplicates)
}

#[derive(Deserialize)]
pub struct CsvImportRequest {
    #[allow(dead_code)]
    partner_col: Option<String>,
    #[allow(dead_code)]
    amount_col: Option<String>,
    #[allow(dead_code)]
    date_col: Option<String>,
    date_format: String,
    rows: Vec<CsvRow>,
    data_group: Option<i32>,
}

#[derive(Deserialize)]
pub struct CsvRow {
    partner: String,
    amount: String,
    date: String,
    row_number: usize,
}

#[derive(serde::Serialize)]
pub struct CsvImportResult {
    inserted: usize,
    duplicates_found: usize,
    duplicates_skipped: usize,
    errors: Vec<CsvImportError>,
    total_processed: usize,
}

#[derive(serde::Serialize)]
pub struct CsvImportError {
    row: usize,
    reason: String,
}

fn parse_date(date_str: &str, format: &str) -> Option<String> {
    use chrono::NaiveDate;

    let date_str = date_str.trim();
    if date_str.is_empty() {
        return None;
    }

    let formats = if format == "auto-detect" {
        vec!["%d.%m.%Y", "%Y-%m-%d", "%d/%m/%Y", "%m/%d/%Y"]
    } else {
        match format {
            "DD.MM.YYYY" => vec!["%d.%m.%Y"],
            "YYYY-MM-DD" => vec!["%Y-%m-%d"],
            "DD/MM/YYYY" => vec!["%d/%m/%Y"],
            "MM/DD/YYYY" => vec!["%m/%d/%Y"],
            _ => vec!["%d.%m.%Y", "%Y-%m-%d", "%d/%m/%Y", "%m/%d/%Y"],
        }
    };

    for fmt in formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, fmt) {
            return Some(date.format("%Y-%m-%d").to_string());
        }
    }

    None
}

#[post("/expenses/bulk")]
pub async fn bulk_import_expenses(
    pool: web::Data<DbPool>,
    data: web::Json<CsvImportRequest>,
) -> impl Responder {
    let group_id = match data.data_group {
        Some(c) => c,
        None => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("No data group found in request!")
            }));
        }
    };

    let client = match pool.pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }));
        }
    };

    let mut inserted = 0;
    let mut duplicates_found = 0;
    let mut duplicates_skipped = 0;
    let mut errors = Vec::new();

    for row in &data.rows {
        let amount = match normalize_amount(&row.amount) {
            Some(a) => a,
            None => {
                errors.push(CsvImportError {
                    row: row.row_number,
                    reason: format!("Invalid amount format: {}", row.amount),
                });
                continue;
            }
        };

        if row.partner.trim().is_empty() {
            errors.push(CsvImportError {
                row: row.row_number,
                reason: "Partner field is empty".to_string(),
            });
            continue;
        }

        let date = if row.date.trim().is_empty() {
            None
        } else {
            match parse_date(&row.date, &data.date_format) {
                Some(d) => Some(d),
                None => {
                    errors.push(CsvImportError {
                        row: row.row_number,
                        reason: format!("Invalid date format: {}", row.date),
                    });
                    continue;
                }
            }
        };

        let exists = client
            .query_opt(
                "SELECT id FROM expenses WHERE partner = $1 AND amount::text = $2 AND data_group = $3 LIMIT 1",
                &[&row.partner, &format!("{:.2}", amount), &group_id],
            )
            .await;

        match exists {
            Ok(Some(_)) => {
                duplicates_found += 1;
                duplicates_skipped += 1;
                continue;
            }
            Ok(None) => {}
            Err(e) => {
                errors.push(CsvImportError {
                    row: row.row_number,
                    reason: format!("Database error checking duplicate: {}", e),
                });
                continue;
            }
        }

        let result = client
            .query(
                "INSERT INTO expenses (data_group, date, partner, amount, expense_type, bill, application, is_cash) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
                &[
                    &group_id,
                    &date,
                    &row.partner.trim(),
                    &amount,
                    &0,
                    &Option::<i32>::None,
                    &Option::<i32>::None,
                    &false,
                ],
            )
            .await;

        match result {
            Ok(_) => inserted += 1,
            Err(e) => {
                errors.push(CsvImportError {
                    row: row.row_number,
                    reason: format!("Failed to insert: {}", e),
                });
            }
        }
    }

    HttpResponse::Ok().json(CsvImportResult {
        inserted,
        duplicates_found,
        duplicates_skipped,
        errors,
        total_processed: data.rows.len(),
    })
}

#[delete("/expenses/{id}")]
pub async fn delete_expense(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let id = path.into_inner();
    let group_id = query
        .get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);

    let client = match pool.pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }));
        }
    };

    let result = client
        .execute(
            "DELETE FROM expenses WHERE id = $1 AND data_group = $2",
            &[&id, &group_id],
        )
        .await;

    match result {
        Ok(rows) => {
            if rows > 0 {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": format!("Expense {} deleted successfully", id)
                }))
            } else {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Expense {} not found in group {}", id, group_id)
                }))
            }
        }
        Err(e) => {
            crate::db::log_db_error("delete_expense", &e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to delete expense: {}", e)
            }))
        }
    }
}
