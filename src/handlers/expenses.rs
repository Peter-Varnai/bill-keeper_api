use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use rusqlite::params;
use std::collections::HashMap;
use crate::db::DbPool;
use crate::models::{Expense, BillNumberUpdate};
use serde::Deserialize;

#[get("/expenses")]
pub async fn get_expenses(
    pool: web::Data<DbPool>,
    query: web::Query<HashMap<String, String>>
) -> impl Responder {
    // Get group_id from query, default to 1
    let group_id = query.get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);
    
    // Get table names from data_groups
    let expenses_table = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        match DbPool::get_table_names(&dg_conn, group_id) {
            Ok((exp_table, _)) => exp_table,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        }
    };
    
    let conn = pool.expenses_conn.lock().unwrap();
    
    let query_sql = format!("SELECT * FROM {}", expenses_table);
    let mut stmt = conn.prepare(&query_sql).unwrap();
    let mut rows = stmt.query([]).unwrap();
    let mut expenses = Vec::new();

    while let Some(row) = rows.next().unwrap() {
        expenses.push(Expense {
            id: row.get(0).unwrap(),
            date: row.get(1).unwrap(),
            partner: row.get(2).unwrap(),
            amount: {
                let raw: String = row.get(3).unwrap();
                raw.replace(",", ".").parse::<f64>().unwrap_or(0.0)
            },
            expense_type: row.get(4).unwrap(),
            bill: row.get(5).unwrap(),
            application: row.get(6).unwrap(),
            Bargeldabhebung: row.get(7).unwrap(),
        });
    }

    HttpResponse::Ok().json(expenses)
}

#[patch("/expenses/{id}/bill")]
pub async fn update_expense_bill(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    params: web::Json<BillNumberUpdate>,
    query: web::Query<HashMap<String, String>>
) -> impl Responder {
    let id = path.into_inner();
    
    // Get group_id from query, default to 1
    let group_id = query.get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);
    
    // Get table names from data_groups
    let expenses_table = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        match DbPool::get_table_names(&dg_conn, group_id) {
            Ok((exp_table, _)) => exp_table,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        }
    };
    
    let conn = pool.expenses_conn.lock().unwrap();

    let sql = format!("UPDATE {} SET bill = ?1 WHERE id = ?2", expenses_table);
    conn.execute(
        &sql,
        params![params.new_number, id]
    ).expect("failed to update bill number");

    HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Updated bill number to {} for expense {}", params.new_number, id)
    }))
}

#[patch("/expenses/{id}/type")]
pub async fn update_expense_type(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    params: web::Json<BillNumberUpdate>,
    query: web::Query<HashMap<String, String>>
) -> impl Responder {
    let id = path.into_inner();
    
    // Get group_id from query, default to 1
    let group_id = query.get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);
    
    // Get table names from data_groups
    let expenses_table = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        match DbPool::get_table_names(&dg_conn, group_id) {
            Ok((exp_table, _)) => exp_table,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        }
    };
    
    let conn = pool.expenses_conn.lock().unwrap();

    let sql = format!("UPDATE {} SET expense_type = ?1 WHERE id = ?2", expenses_table);
    conn.execute(
        &sql,
        params![params.new_number, id]
    ).expect("failed to update expense type");

    HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Updated expense type to {} for expense {}", params.new_number, id)
    }))
}

#[patch("/expenses/{id}/application")]
pub async fn update_expense_application(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    params: web::Json<BillNumberUpdate>,
    query: web::Query<HashMap<String, String>>
) -> impl Responder {
    let id = path.into_inner();
    
    // Get group_id from query, default to 1
    let group_id = query.get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);
    
    // Get table names from data_groups
    let expenses_table = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        match DbPool::get_table_names(&dg_conn, group_id) {
            Ok((exp_table, _)) => exp_table,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        }
    };
    
    let conn = pool.expenses_conn.lock().unwrap();

    let sql = format!("UPDATE {} SET application = ?1 WHERE id = ?2", expenses_table);
    conn.execute(
        &sql,
        params![params.new_number, id]
    ).expect("failed to update application");

    HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Updated application to {} for expense {}", params.new_number, id)
    }))
}

#[patch("/expenses/{id}/cash")]
pub async fn update_expense_cash(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    params: web::Json<BillNumberUpdate>,
    query: web::Query<HashMap<String, String>>
) -> impl Responder {
    let id = path.into_inner();
    
    // Get group_id from query, default to 1
    let group_id = query.get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);
    
    // Get table names from data_groups
    let expenses_table = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        match DbPool::get_table_names(&dg_conn, group_id) {
            Ok((exp_table, _)) => exp_table,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        }
    };
    
    let conn = pool.expenses_conn.lock().unwrap();

    let sql = format!("UPDATE {} SET Bargeldabhebung = ?1 WHERE id = ?2", expenses_table);
    conn.execute(
        &sql,
        params![params.new_number, id]
    ).expect("failed to update cash status");

    HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Updated cash status to {} for expense {}", params.new_number, id)
    }))
}

// Request body for creating a single expense
#[derive(Deserialize)]
pub struct CreateExpenseRequest {
    partner: String,
    amount: String,
    date: Option<String>,
    expense_type: Option<i32>,
    bill: Option<i32>,
    application: Option<i32>,
    Bargeldabhebung: Option<bool>,
    group_id: Option<i32>,
}

#[post("/expenses")]
pub async fn create_expense(
    pool: web::Data<DbPool>,
    data: web::Json<CreateExpenseRequest>,
) -> impl Responder {
    let group_id = data.group_id.unwrap_or(1);
    
    // Get table names from data_groups
    let expenses_table = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        match DbPool::get_table_names(&dg_conn, group_id) {
            Ok((exp_table, _)) => exp_table,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        }
    };
    
    // Validate required fields
    if data.partner.trim().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Partner field is required"
        }));
    }
    
    // Parse amount
    let amount_normalized = normalize_amount(&data.amount);
    if amount_normalized.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Invalid amount format: {}", data.amount)
        }));
    }
    
    let conn = pool.expenses_conn.lock().unwrap();
    
    // Insert the expense
    let sql = format!(
        "INSERT INTO {} (date, partner, amount, expense_type, bill, application, Bargeldabhebung) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        expenses_table
    );
    
    let result = conn.execute(
        &sql,
        params![
            data.date,
            data.partner.trim(),
            format!("{:.2}", amount_normalized.unwrap()),
            data.expense_type.unwrap_or(0),
            data.bill.unwrap_or(0),
            data.application.unwrap_or(0),
            if data.Bargeldabhebung.unwrap_or(false) { 1 } else { 0 }
        ],
    );
    
    match result {
        Ok(_) => {
            let new_id = conn.last_insert_rowid() as u16;
            HttpResponse::Created().json(Expense {
                id: new_id,
                date: data.date.clone(),
                partner: data.partner.clone(),
                amount: amount_normalized.unwrap(),
                expense_type: data.expense_type.unwrap_or(0) as u16,
                bill: data.bill.unwrap_or(0) as u16,
                application: data.application.map(|a| a as u8),
                Bargeldabhebung: data.Bargeldabhebung,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create expense: {}", e)
            }))
        }
    }
}

/// Helper function to normalize amount strings
/// Supports formats like: "1234.56", "1,234.56", "1.234,56", "1234,56"
fn normalize_amount(value: &str) -> Option<f64> {
    let clean = value.trim();
    
    if clean.is_empty() {
        return None;
    }
    
    // Find last comma or dot
    let last_comma = clean.rfind(',');
    let last_dot = clean.rfind('.');
    
    let result = match (last_comma, last_dot) {
        (Some(comma_pos), Some(dot_pos)) => {
            if comma_pos > dot_pos {
                // European format: 1.234,56 -> 1234.56
                clean.replace(".", "").replace(",", ".")
            } else {
                // US format: 1,234.56 -> 1234.56
                clean.replace(",", "")
            }
        }
        (Some(_), None) => {
            // Only comma present: might be decimal separator (European)
            // Check if it looks like a decimal (only 1-2 digits after comma)
            if let Some(comma_pos) = last_comma {
                let after_comma = clean.len() - comma_pos - 1;
                if after_comma <= 2 {
                    clean.replace(",", ".")
                } else {
                    // It's a thousands separator
                    clean.replace(",", "")
                }
            } else {
                clean.to_string()
            }
        }
        (None, Some(_)) => {
            // Only dot present: standard decimal
            clean.to_string()
        }
        (None, None) => clean.to_string(),
    };
    
    result.parse::<f64>().ok()
}

/// Check if an expense already exists (duplicate detection)
fn check_duplicate(
    conn: &rusqlite::Connection,
    table_name: &str,
    partner: &str,
    amount: f64,
    date: &Option<String>,
) -> Result<Option<Expense>, rusqlite::Error> {
    let sql = format!(
        "SELECT id, date, partner, amount, expense_type, bill, application, Bargeldabhebung 
         FROM {} 
         WHERE partner = ?1 AND amount = ?2 AND date = ?3 
         LIMIT 1",
        table_name
    );
    
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params![partner, format!("{:.2}", amount), date])?;
    
    if let Some(row) = rows.next()? {
        Ok(Some(Expense {
            id: row.get(0)?,
            date: row.get(1)?,
            partner: row.get(2)?,
            amount: {
                let raw: String = row.get(3)?;
                raw.replace(",", ".").parse::<f64>().unwrap_or(0.0)
            },
            expense_type: row.get(4)?,
            bill: row.get(5)?,
            application: row.get(6)?,
            Bargeldabhebung: row.get(7)?,
        }))
    } else {
        Ok(None)
    }
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
    
    // Get table names from data_groups
    let expenses_table = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        match DbPool::get_table_names(&dg_conn, group_id) {
            Ok((exp_table, _)) => exp_table,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        }
    };
    
    let conn = pool.expenses_conn.lock().unwrap();
    let mut duplicates = Vec::new();
    
    for (index, item) in data.expenses.iter().enumerate() {
        if let Some(amount) = normalize_amount(&item.amount) {
            if let Ok(Some(existing)) = check_duplicate(&conn, &expenses_table, &item.partner, amount, &item.date) {
                duplicates.push(DuplicateCheckResult {
                    index,
                    existing,
                });
            }
        }
    }
    
    HttpResponse::Ok().json(duplicates)
}

// CSV Import structures
#[derive(Deserialize)]
pub struct CsvImportRequest {
    partner_col: String,
    amount_col: String,
    date_col: String,
    date_format: String,  // "auto-detect", "DD.MM.YYYY", "YYYY-MM-DD", "DD/MM/YYYY", "MM/DD/YYYY"
    rows: Vec<CsvRow>,
    group_id: Option<i32>,
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

/// Parse date string with specified format
fn parse_date(date_str: &str, format: &str) -> Option<String> {
    use chrono::NaiveDate;
    
    let date_str = date_str.trim();
    
    if date_str.is_empty() {
        return None;
    }
    
    let formats = if format == "auto-detect" {
        vec![
            "%d.%m.%Y",      // DD.MM.YYYY
            "%Y-%m-%d",      // YYYY-MM-DD
            "%d/%m/%Y",      // DD/MM/YYYY
            "%m/%d/%Y",      // MM/DD/YYYY
        ]
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
    let group_id = data.group_id.unwrap_or(1);
    
    // Get table names from data_groups
    let expenses_table = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        match DbPool::get_table_names(&dg_conn, group_id) {
            Ok((exp_table, _)) => exp_table,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        }
    };
    
    let mut conn = pool.expenses_conn.lock().unwrap();
    let tx = conn.transaction().unwrap();
    
    let mut inserted = 0;
    let mut duplicates_found = 0;
    let mut duplicates_skipped = 0;
    let mut errors = Vec::new();
    
    for row in &data.rows {
        // Validate and parse amount
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
        
        // Validate partner
        if row.partner.trim().is_empty() {
            errors.push(CsvImportError {
                row: row.row_number,
                reason: "Partner field is empty".to_string(),
            });
            continue;
        }
        
        // Parse date
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
        
        // Check for duplicates
        match check_duplicate(&tx, &expenses_table, &row.partner, amount, &date) {
            Ok(Some(_)) => {
                duplicates_found += 1;
                duplicates_skipped += 1;
                continue;  // Skip duplicate
            }
            Err(e) => {
                errors.push(CsvImportError {
                    row: row.row_number,
                    reason: format!("Database error checking duplicate: {}", e),
                });
                continue;
            }
            _ => {}
        }
        
        // Insert the expense
        let sql = format!(
            "INSERT INTO {} (date, partner, amount, expense_type, bill, application, Bargeldabhebung) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            expenses_table
        );
        
        match tx.execute(
            &sql,
            params![
                date,
                row.partner.trim(),
                format!("{:.2}", amount),
                0,  // expense_type
                0,  // bill
                0,  // application
                0   // Bargeldabhebung (false)
            ],
        ) {
            Ok(_) => inserted += 1,
            Err(e) => {
                errors.push(CsvImportError {
                    row: row.row_number,
                    reason: format!("Failed to insert: {}", e),
                });
            }
        }
    }
    
    if let Err(e) = tx.commit() {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to commit transaction: {}", e)
        }));
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
    query: web::Query<HashMap<String, String>>
) -> impl Responder {
    let id = path.into_inner();
    
    // Get group_id from query, default to 1
    let group_id = query.get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);
    
    // Get table names from data_groups
    let expenses_table = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        match DbPool::get_table_names(&dg_conn, group_id) {
            Ok((exp_table, _)) => exp_table,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        }
    };
    
    let conn = pool.expenses_conn.lock().unwrap();
    
    // Execute DELETE query
    let sql = format!("DELETE FROM {} WHERE id = ?1", expenses_table);
    match conn.execute(&sql, params![id]) {
        Ok(rows_affected) => {
            if rows_affected > 0 {
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
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to delete expense: {}", e)
            }))
        }
    }
}