use actix_web::{get, web, HttpResponse, Responder};
use crate::db::DbPool;
use crate::services::calculate_ear_totals;
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
                amount: row.get::<_, String>(4).parse().unwrap_or(0.0),
                expense_type: row.get(5),
                bill: row.get(6),
                application: row.get(7),
                is_cash: row.get(8),
            })
            .collect(),
        _ => vec![],
    };

    let totals = calculate_ear_totals(&expenses);

    HttpResponse::Ok().json(EarResponse {
        expenses,
        totals,
    })
}