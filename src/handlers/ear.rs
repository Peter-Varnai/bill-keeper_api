use crate::auth::get_user_id;
use crate::db::DbPool;
use crate::helpers::{get_data_group_url, verify_data_group_ownership};
use crate::services::calculate_ear_totals;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use rust_decimal::Decimal;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct EarResponse {
    expenses: Vec<crate::models::Expense>,
    totals: crate::models::EarTotals,
}

#[get("/ear")]
pub async fn get_ear(
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

    let result = client
        .query(
            "SELECT id, data_group, date, partner, amount, expense_type, bill, application, is_cash 
             FROM expenses WHERE data_group = $1",
            &[&group_id],
        )
        .await;

    let expenses: Vec<crate::models::Expense> = match result {
        Ok(rows) => rows
            .iter()
            .map(|row| crate::models::Expense {
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

    HttpResponse::Ok().json(EarResponse { expenses, totals })
}
