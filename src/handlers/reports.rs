use actix_web::{get, web, HttpResponse, Responder};
use crate::db::DbPool;
use crate::models::BillToHtml;
use std::collections::HashMap;

#[get("/reports")]
pub async fn get_report(
    pool: web::Data<DbPool>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let group_id = query
        .get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);

    let application_report_id = query
        .get("application_report_id")
        .and_then(|v| v.parse::<i32>().ok());

    match application_report_id {
        Some(app_id) => {
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
                    "SELECT e.id, e.partner, e.amount::text, e.date, e.expense_type, e.bill, e.is_cash
                     FROM expenses e
                     WHERE e.application = $1 AND e.data_group = $2",
                    &[&app_id, &group_id],
                )
                .await;

            let bills: Vec<BillToHtml> = match result {
                Ok(rows) => {
                    let mut result_bills = Vec::new();
                    for row in rows {
                        let bill_id: Option<i32> = row.get(5);
                        let mut filename = String::new();

                        if let Some(bid) = bill_id {
                            if let Ok(bill_rows) = client
                                .query(
                                    "SELECT filename FROM bills WHERE id = $1 AND data_group = $2",
                                    &[&bid, &group_id],
                                )
                                .await
                            {
                                if let Some(bill_row) = bill_rows.first() {
                                    filename = bill_row.get::<_, String>(0);
                                }
                            }
                        }

                        result_bills.push(BillToHtml {
                            expense_id: row.get(0),
                            partner: row.get(1),
                            amount: row.get(2),
                            date: row.get(3),
                            expense_type: row.get(4),
                            filename,
                            is_cash: row.get(6),
                        });
                    }
                    result_bills
                }
                _ => vec![],
            };

            HttpResponse::Ok().json(bills)
        }
        None => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "application_report_id is required"
        })),
    }
}