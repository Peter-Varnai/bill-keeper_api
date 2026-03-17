use actix_web::{get, web, HttpResponse, Responder};
use rusqlite::params;
use crate::db::DbPool;
use crate::models::BillToHtml;
use std::collections::HashMap;

fn get_report_data(
    pool: &web::Data<DbPool>, 
    application_report_id: i32,
    group_id: i32
) -> Vec<BillToHtml> {
    // Get table names from data_groups first
    let (expenses_table, bills_table) = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        match DbPool::get_table_names(&dg_conn, group_id) {
            Ok(tables) => tables,
            Err(_) => return Vec::new(), // Return empty if group not found
        }
    };
    
    let conn = pool.expenses_conn.lock().unwrap();
    let bills_conn = pool.bills_conn.lock().unwrap();
    
    let query = format!(
        "SELECT id, partner, amount, date, expense_type, bill, Bargeldabhebung FROM {} WHERE application = {}",
        expenses_table, application_report_id
    );
    
    let mut stmt = conn.prepare(&query).unwrap();
    let mut rows = stmt.query([]).unwrap();
    let mut bills = Vec::new();

    while let Some(row) = rows.next().unwrap() {
        let bill_no: u8 = row.get(5).unwrap();
        
        // Get filename from bills db
        let filename = {
            let bill_sql = format!("SELECT filename FROM {} WHERE no = ?1", bills_table);
            let mut bill_stmt = bills_conn.prepare(&bill_sql).unwrap();
            let mut bill_rows = bill_stmt.query(params![bill_no]).unwrap();
            
            if let Some(bill_row) = bill_rows.next().unwrap() {
                bill_row.get(0).unwrap_or_else(|_| "".to_string())
            } else {
                "".to_string()
            }
        };
        
        bills.push(BillToHtml {
            expense_id: row.get(0).unwrap(),
            partner: row.get(1).unwrap(),
            amount: row.get(2).unwrap(),
            date: row.get(3).unwrap(),
            expense_type: row.get(4).unwrap(),
            filename,
            Bargeldabhebung: row.get(6).unwrap(),
        });
    }

    bills
}

#[get("/reports")]
pub async fn get_report(
    pool: web::Data<DbPool>,
    query: web::Query<HashMap<String, String>>
) -> impl Responder {
    let group_id = query.get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);
    
    let application_report_id = query.get("application_report_id")
        .and_then(|v| v.parse::<i32>().ok());
    
    match application_report_id {
        Some(app_id) => {
            let bills = get_report_data(&pool, app_id, group_id);
            HttpResponse::Ok().json(bills)
        }
        None => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "application_report_id is required"
            }))
        }
    }
}
