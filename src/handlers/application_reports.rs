use crate::db::DbPool;
use crate::models::ApplicationReport;
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GetApplicationReportsQuery {
    group_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct CreateApplicationReportRequest {
    name: String,
    amount: f64,
    submission_deadline: Option<String>,
    group_id: i32,
}

#[derive(Deserialize)]
pub struct UpdateApplicationReportRequest {
    name: Option<String>,
    amount: Option<f64>,
    submission_deadline: Option<String>,
}

#[derive(Serialize)]
pub struct ApplicationReportResponse {
    id: i32,
    data_group_id: i32,
    name: String,
    amount: f64,
    date_created: String,
    submission_deadline: Option<String>,
}

#[get("/application_reports")]
pub async fn get_application_reports(
    pool: web::Data<DbPool>,
    query: web::Query<GetApplicationReportsQuery>,
) -> impl Responder {
    let group_id = query.group_id.unwrap_or(1);

    let conn = pool.data_groups_conn.lock().unwrap();

    let mut stmt = conn
        .prepare(
            "SELECT id, data_group_id, name, amount, date_created, submission_deadline 
         FROM application_reports 
         WHERE data_group_id = ?1 
         ORDER BY date_created DESC",
        )
        .unwrap();

    let reports: Vec<ApplicationReport> = stmt
        .query_map([group_id], |row| {
            Ok(ApplicationReport {
                id: row.get(0)?,
                data_group_id: row.get(1)?,
                name: row.get(2)?,
                amount: row.get(3)?,
                date_created: row.get(4)?,
                submission_deadline: row.get(5).ok(),
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    HttpResponse::Ok().json(reports)
}

#[post("/application_reports")]
pub async fn create_application_report(
    pool: web::Data<DbPool>,
    data: web::Json<CreateApplicationReportRequest>,
) -> impl Responder {
    let conn = pool.data_groups_conn.lock().unwrap();

    let result = conn.execute(
        "INSERT INTO application_reports (data_group_id, name, amount, submission_deadline) 
         VALUES (?1, ?2, ?3, ?4)",
        params![
            &data.group_id,
            &data.name,
            &data.amount,
            &data.submission_deadline
        ],
    );

    match result {
        Ok(_) => {
            let id = conn.last_insert_rowid();
            HttpResponse::Created().json(serde_json::json!({
                "id": id,
                "data_group_id": data.group_id,
                "name": data.name,
                "amount": data.amount,
                "submission_deadline": data.submission_deadline,
                "message": format!("Application report '{}' created successfully", data.name)
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create application report: {}", e)
        })),
    }
}

#[patch("/application_reports/{id}")]
pub async fn update_application_report(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    data: web::Json<UpdateApplicationReportRequest>,
) -> impl Responder {
    let id = path.into_inner();

    // Build dynamic update query
    let mut updates = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref name) = data.name {
        updates.push("name = ?");
        values.push(Box::new(name.clone()));
    }

    if let Some(amount) = data.amount {
        updates.push("amount = ?");
        values.push(Box::new(amount));
    }

    if let Some(ref deadline) = data.submission_deadline {
        updates.push("submission_deadline = ?");
        values.push(Box::new(deadline.clone()));
    }

    if updates.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No fields to update"
        }));
    }

    // Add id to values for the WHERE clause
    values.push(Box::new(id));

    let conn = pool.data_groups_conn.lock().unwrap();

    // Get total parameter count
    let param_count = values.len();

    // Build the query dynamically
    let query = format!(
        "UPDATE application_reports SET {} WHERE id = ?",
        updates.join(", ")
    );

    // Use rusqlite's params_from_iter for dynamic parameters
    let result = conn.execute(
        &query,
        rusqlite::params_from_iter(values.iter().map(|v| v.as_ref())),
    );

    match result {
        Ok(rows) => {
            if rows == 0 {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Application report not found"
                }));
            }
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Application report updated successfully"
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update application report: {}", e)
        })),
    }
}

#[delete("/application_reports/{id}")]
pub async fn delete_application_report(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
) -> impl Responder {
    let id = path.into_inner();

    // First, get the data_group_id and expenses_table_name to update expenses
    let (group_id, expenses_table): (i32, String) = {
        let conn = pool.data_groups_conn.lock().unwrap();
        conn.query_row(
            "SELECT ar.data_group_id, dg.expenses_table_name 
             FROM application_reports ar 
             JOIN data_groups dg ON ar.data_group_id = dg.id 
             WHERE ar.id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Application report not found"
            }))
        })
        .unwrap()
    };

    // Update all expenses that have this application to default (NULL or 0)
    {
        let expenses_conn = pool.expenses_conn.lock().unwrap();
        let update_sql = format!(
            "UPDATE {} SET application = 0 WHERE application = ?1",
            expenses_table
        );

        if let Err(e) = expenses_conn.execute(&update_sql, [id]) {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to update expenses: {}", e)
            }));
        }
    }

    // Now delete the application report
    {
        let conn = pool.data_groups_conn.lock().unwrap();
        let result = conn.execute("DELETE FROM application_reports WHERE id = ?1", [id]);

        match result {
            Ok(rows) => {
                if rows == 0 {
                    return HttpResponse::NotFound().json(serde_json::json!({
                        "error": "Application report not found"
                    }));
                }
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Application report deleted successfully"
                }))
            }
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to delete application report: {}", e)
            })),
        }
    }
}
