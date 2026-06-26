use crate::auth::get_user_id;
use crate::db::DbPool;
use crate::helpers::{get_data_group_url, verify_data_group_ownership};
use crate::models::ApplicationReport;
use crate::services::calculate_summaries;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use rust_decimal::Decimal;
use std::collections::HashMap;

#[get("/summaries")]
pub async fn get_summaries(
    pool: web::Data<DbPool>,
    query: web::Query<HashMap<String, String>>,
    req: HttpRequest,
) -> impl Responder {
    let data_group = match get_data_group_url(&query) {
        Ok(c) => c,
        Err(response) => return response,
    };

    let user_id = match get_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    if let Err(response) = verify_data_group_ownership(&pool, data_group, user_id).await {
        return response;
    }

    let application_reports: Vec<ApplicationReport> = {
        let client = match pool.get_client().await {
            Ok(c) => c,
            Err(response) => return response,
        };

        match client
            .query(
                "SELECT id, data_group, name, amount::text, created_at, submission_deadline 
                 FROM application_reports 
                 WHERE data_group = $1",
                &[&data_group],
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

    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return response,
    };

    let result = client
        .query(
            "SELECT id, data_group, date, partner, amount, expense_type, bill, application, is_cash 
             FROM expenses WHERE data_group = $1",
            &[&data_group],
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
        _ => vec![],
    };

    let summaries = calculate_summaries(&expenses, &application_reports);
    HttpResponse::Ok().json(summaries)
}
