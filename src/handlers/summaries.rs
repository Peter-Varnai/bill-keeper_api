use actix_web::{get, web, HttpResponse, Responder};
use crate::db::DbPool;
use crate::models::ApplicationReport;
use crate::services::calculate_summaries;
use std::collections::HashMap;

#[get("/summaries")]
pub async fn get_summaries(
    pool: web::Data<DbPool>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let group_id = query
        .get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);

    let application_reports: Vec<ApplicationReport> = {
        let client = match pool.pool.get().await {
            Ok(c) => c,
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database connection error: {}", e)
                }));
            }
        };

        match client
            .query(
                "SELECT id, data_group, name, amount::text, created_at, submission_deadline 
                 FROM application_reports 
                 WHERE data_group = $1",
                &[&group_id],
            )
            .await
        {
            Ok(rows) => rows
                .iter()
                .map(|row| ApplicationReport {
                    id: row.get(0),
                    data_group: row.get(1),
                    name: row.get(2),
                    amount: row.get::<_, String>(3).parse().unwrap_or(0.0),
                    created_at: row.get(4),
                    submission_deadline: row.get(5),
                })
                .collect(),
            _ => vec![],
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

    let summaries = calculate_summaries(&expenses, &application_reports);
    HttpResponse::Ok().json(summaries)
}