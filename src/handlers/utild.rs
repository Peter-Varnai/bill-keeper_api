use crate::auth::get_user_id;
use crate::db::DbPool;
use crate::helpers::{get_data_group_url, verify_data_group_ownership};
use crate::models::Expense;
use crate::services::calculate_ear_totals;
use actix_web::{get, put, web, HttpRequest, HttpResponse, Responder};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Debug)]
struct UtildResponse {
    data_group: i32,
    bank_stand: Option<f64>,
    cash_stand: Option<f64>,
    expense_summary: ExpenseSummary,
    calculated_totals: CalculatedTotals,
}

#[derive(Serialize, Debug)]
struct ExpenseSummary {
    bank_total: f64,
    cash_total: f64,
}

#[derive(Serialize, Debug)]
struct CalculatedTotals {
    bank_with_expenses: f64,
    cash_with_expenses: f64,
}

#[derive(Deserialize)]
struct UtildUpdateRequest {
    data_group: i32,
    bank_stand: Option<f64>,
    cash_stand: Option<f64>,
}

#[get("/utild")]
pub async fn get_utild(
    pool: web::Data<DbPool>,
    query: web::Query<HashMap<String, String>>,
    req: HttpRequest,
) -> impl Responder {
    let group_id = match get_data_group_url(&query) {
        Ok(c) => c,
        Err(response) => return response,
    };

    let user_id = match get_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    if let Err(response) = verify_data_group_ownership(&pool, group_id, user_id).await {
        return response;
    }

    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return response,
    };

    let utility_result = client
        .query(
            "SELECT key, value FROM utility_data WHERE data_group = $1",
            &[&group_id],
        )
        .await;

    let (bank_stand, cash_stand) = match utility_result {
        Ok(rows) => {
            let mut bank = None;
            let mut cash = None;
            for row in rows.iter() {
                let key: String = row.get(0);
                let value: Option<Decimal> = row.get(1);
                match key.as_str() {
                    "bank_stand" => bank = value.and_then(|v| v.to_string().parse().ok()),
                    "cash_stand" => cash = value.and_then(|v| v.to_string().parse().ok()),
                    _ => {}
                }
            }
            (bank, cash)
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }));
        }
    };

    let expense_result = client
        .query(
            "SELECT id, data_group, date, partner, amount, expense_type, bill, application, is_cash 
             FROM expenses WHERE data_group = $1",
            &[&group_id],
        )
        .await;

    let expenses: Vec<Expense> = match expense_result {
        Ok(rows) => rows
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
            .collect(),
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }));
        }
    };

    let totals = calculate_ear_totals(&expenses);
    let bank_expenses: f64 = totals.bank_total.parse().unwrap_or(0.0);
    let cash_expenses: f64 = totals.cash_total.parse().unwrap_or(0.0);

    let response = UtildResponse {
        data_group: group_id,
        bank_stand,
        cash_stand,
        expense_summary: ExpenseSummary {
            bank_total: bank_expenses,
            cash_total: cash_expenses,
        },
        calculated_totals: CalculatedTotals {
            bank_with_expenses: bank_stand.unwrap_or(0.0) + bank_expenses,
            cash_with_expenses: cash_stand.unwrap_or(0.0) + cash_expenses,
        },
    };

    HttpResponse::Ok().json(response)
}

#[put("/utild")]
pub async fn update_utild(
    pool: web::Data<DbPool>,
    body: web::Json<UtildUpdateRequest>,
    req: HttpRequest,
) -> impl Responder {
    let user_id = match get_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    if let Err(response) = verify_data_group_ownership(&pool, body.data_group, user_id).await {
        return response;
    }

    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return response,
    };

    if let Some(bank_val) = body.bank_stand {
        let bank_decimal: Decimal = bank_val.to_string().parse().unwrap_or(Decimal::ZERO);
        let result = client
            .execute(
                "INSERT INTO utility_data (data_group, key, value) 
                 VALUES ($1, 'bank_stand', $2)
                 ON CONFLICT (data_group, key) 
                 DO UPDATE SET value = $2, updated_at = now()",
                &[&body.data_group, &bank_decimal],
            )
            .await;

        dbg!(&result);

        if let Err(e) = result {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }));
        }
    }

    if let Some(cash_val) = body.cash_stand {
        let cash_decimal: Decimal = cash_val.to_string().parse().unwrap_or(Decimal::ZERO);
        let result = client
            .execute(
                "INSERT INTO utility_data (data_group, key, value) 
                 VALUES ($1, 'cash_stand', $2)
                 ON CONFLICT (data_group, key) 
                 DO UPDATE SET value = $2, updated_at = now()",
                &[&body.data_group, &cash_decimal],
            )
            .await;

        if let Err(e) = result {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }));
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data_group": body.data_group
    }))
}
