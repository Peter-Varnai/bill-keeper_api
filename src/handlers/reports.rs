use crate::auth::get_user_id;
use crate::db::DbPool;
use crate::helpers::{get_data_group_url, verify_data_group_ownership};
use crate::models::BelegaufstellungItem;
use crate::services::get_expense_type_name;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use std::collections::HashMap;

#[get("/reports")]
pub async fn get_report(
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

    let application_report_id = query
        .get("application_report_id")
        .and_then(|v| v.parse::<i32>().ok());

    match application_report_id {
        Some(app_id) => {
            let client = match pool.get_client().await {
                Ok(c) => c,
                Err(response) => return response,
            };

            let result = client
                .query(
                    "SELECT e.id, e.partner, e.amount, e.date, e.expense_type, e.bill, e.is_cash
                     FROM expenses e
                     WHERE e.application = $1 AND e.data_group = $2",
                    &[&app_id, &data_group],
                )
                .await;

            let items: Vec<BelegaufstellungItem> = match result {
                Ok(rows) => {
                    let mut result_items = Vec::new();
                    for row in rows {
                        let bill_id: Option<i32> = row.get(5);
                        let mut bill_date: Option<chrono::NaiveDate> = None;
                        let mut bill_filename: Option<String> = None;

                        if let Some(bid) = bill_id {
                            if let Ok(bill_rows) = client
                                .query(
                                    "SELECT date, filename FROM bills WHERE id = $1 AND data_group = $2",
                                    &[&bid, &data_group],
                                )
                                .await
                            {
                                if let Some(bill_row) = bill_rows.first() {
                                    bill_date = bill_row.get(0);
                                    bill_filename = bill_row.get(1);
                                }
                            }
                        }

                        let expense_type: i32 = row.get(4);
                        result_items.push(BelegaufstellungItem {
                            expense_id: row.get(0),
                            partner: row.get(1),
                            amount: row.get(2),
                            date: row.get(3),
                            expense_type_name: get_expense_type_name(expense_type),
                            is_cash: row.get(6),
                            bill_date,
                            bill_filename,
                        });
                    }
                    result_items
                }
                _ => vec![],
            };

            HttpResponse::Ok().json(items)
        }
        None => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "application_report_id is required"
        })),
    }
}
