use crate::db::DbPool;
use crate::helpers::{get_data_group_req, get_data_group_url};
use crate::models::{BillNumberUpdate, CreateExpenseRequest, CsvImportRequest, Expense};
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

#[get("/expenses")]
pub async fn get_expenses(
    pool: web::Data<DbPool>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let data_group = match get_data_group_url(&query) {
        Ok(c) => c,
        Err(response) => return response,
    };

    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return response,
    };

    let result = client
        .query(
            "SELECT id, data_group, date, partner, amount, expense_type, bill, application, is_cash 
             FROM expenses WHERE data_group = $1 ORDER BY id",
            &[&data_group],
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
                    amount: row.get::<_, Decimal>(4),
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
    let group_id = match get_data_group_url(&query) {
        Ok(c) => c,
        Err(response) => return response,
    };

    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return response,
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
    let group_id = match get_data_group_url(&query) {
        Ok(c) => c,
        Err(response) => return response,
    };

    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return response,
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
    let group_id = match get_data_group_url(&query) {
        Ok(c) => c,
        Err(response) => return response,
    };

    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return response,
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
    let group_id = match get_data_group_url(&query) {
        Ok(c) => c,
        Err(response) => return response,
    };

    let is_cash = params.new_number != 0;

    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return response,
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

#[post("/expenses")]
pub async fn create_expense(
    pool: web::Data<DbPool>,
    data: web::Json<CreateExpenseRequest>,
) -> impl Responder {
    let data_group = match get_data_group_req(data.data_group) {
        Ok(c) => c,
        Err(response) => return response,
    };

    if data.partner.trim().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Partner field is required"
        }));
    }

    // let amount_normalized = normalize_amount(&data.amount);
    // if amount_normalized.is_none() {
    //     return HttpResponse::BadRequest().json(serde_json::json!({
    //         "error": format!("Invalid amount format: {}", data.amount)
    //     }));
    // }
    //
    // let amount_val = amount_normalized.unwrap();

    let amount = match Decimal::from_str(&data.amount) {
        Ok(val) => val,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Error converting expense amount value: {}", e)
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

    let result = client
        .query(
            "INSERT INTO expenses (data_group, date, partner, amount, expense_type, bill, application, is_cash) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
            &[
                &data_group,
                &data.date,
                &data.partner.trim(),
                &amount,
                &data.expense_type.unwrap_or(0),
                &data.bill,
                &data.application,
                &data.is_cash.unwrap_or(false),
            ],
        )
        .await;

    match result {
        Ok(rows) => {
            let new_id = rows.first().map(|r| r.get::<_, i32>(0)).unwrap_or(0);
            HttpResponse::Created().json(Expense {
                id: new_id,
                data_group: data_group,
                date: data.date,
                partner: data.partner.clone(),
                amount: amount,
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

#[post("/expenses/bulk")]
pub async fn bulk_import_expenses(
    pool: web::Data<DbPool>,
    data: web::Json<CsvImportRequest>,
) -> impl Responder {
    let data_group = match crate::helpers::get_data_group_req(data.data_group) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return response,
    };

    let mut inserted = 0;
    let mut duplicates_found = 0;
    let mut duplicates_skipped = 0;
    let mut errors = Vec::new();

    for row in &data.rows {
        // let amount = match normalize_amount(&row.amount) {
        //     Some(a) => a,
        //     None => {
        //         errors.push(CsvImportError {
        //             row: row.row_number,
        //             reason: format!("Invalid amount format: {}", row.amount),
        //         });
        //         continue;
        //     }
        // };

        // let amount = match Decimal::from_str(&row.amount) {
        //     Ok(val) => val,
        //     Err(e) => {
        //         return HttpResponse::InternalServerError().json(serde_json::json!({
        //             "error": format!("Error converting expense amount value: {}", e)
        //         }));
        //     }
        // };

        // let date = if row.date.trim().is_empty() {
        //     None
        // } else {
        //     match parse_date(&row.date, &data.date_format) {
        //         Some(d) => Some(d),
        //         None => {
        //             errors.push(CsvImportError {
        //                 row: row.row_number,
        //                 reason: format!("Invalid date format: {}", row.date),
        //             });
        //             continue;
        //         }
        //     }
        // };

        if row.partner.trim().is_empty() {
            errors.push(CsvImportError {
                row: row.row_number,
                reason: "Partner field is empty".to_string(),
            });
            continue;
        }

        let Some(date) = row.date else {
            errors.push(CsvImportError {
                row: row.row_number,
                reason: format!("Invalid date format: {:?}", row.date),
            });
            continue;
        };
        let exists = client
            .query_opt(
                "SELECT id FROM expenses WHERE partner = $1 AND amount = $2 AND data_group = $3 LIMIT 1",
                &[&row.partner, &row.amount, &data_group],
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
                    &data_group,
                    &date,
                    &row.partner.trim(),
                    &row.amount,
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
    let group_id = match get_data_group_url(&query) {
        Ok(c) => c,
        Err(response) => return response,
    };

    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return response,
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
