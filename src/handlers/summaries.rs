use actix_web::{get, web, HttpResponse, Responder};
use crate::db::DbPool;
use crate::models::ApplicationReport;
use crate::services::calculate_summaries;
use std::collections::HashMap;

#[get("/summaries")]
pub async fn get_summaries(
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
    
    // Fetch application_reports for this group
    let application_reports: Vec<ApplicationReport> = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        let mut stmt = dg_conn.prepare(
            "SELECT id, data_group_id, name, amount, date_created, submission_deadline 
             FROM application_reports 
             WHERE data_group_id = ?1"
        ).unwrap();
        
        stmt.query_map([group_id], |row| {
            Ok(ApplicationReport {
                id: row.get(0)?,
                data_group_id: row.get(1)?,
                name: row.get(2)?,
                amount: row.get(3)?,
                date_created: row.get(4)?,
                submission_deadline: row.get(5).ok(),
            })
        }).unwrap().filter_map(|r| r.ok()).collect()
    };
    
    let conn = pool.expenses_conn.lock().unwrap();
    
    let query_sql = format!("SELECT * FROM {}", expenses_table);
    let mut stmt = conn.prepare(&query_sql).unwrap();
    let mut rows = stmt.query([]).unwrap();
    let mut expenses = Vec::new();

    while let Some(row) = rows.next().unwrap() {
        expenses.push(crate::models::Expense {
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

    let summaries = calculate_summaries(&expenses, &application_reports);
    HttpResponse::Ok().json(summaries)
}