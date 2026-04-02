use crate::db::DbPool;
use crate::helpers::parse_date_or_panic;
use crate::models::ApplicationReport;
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use chrono::NaiveDate;
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
    data_group: i32,
    name: String,
    amount: f64,
    created_at: String,
    submission_deadline: Option<String>,
}

#[get("/application_reports")]
pub async fn get_application_reports(
    pool: web::Data<DbPool>,
    query: web::Query<GetApplicationReportsQuery>,
) -> impl Responder {
    let group_id = query.group_id.unwrap_or(1);

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
            "SELECT id, data_group, name, amount::text, created_at, submission_deadline 
             FROM application_reports 
             WHERE data_group = $1 
             ORDER BY created_at DESC",
            &[&group_id],
        )
        .await;

    match result {
        Ok(rows) => {
            let reports: Vec<ApplicationReport> = rows
                .iter()
                .map(|row| {
                    let deadline: Option<NaiveDate> = row.get(5);
                    dbg!(&deadline);

                    ApplicationReport {
                        id: row.get(0),
                        data_group: row.get(1),
                        name: row.get(2),
                        amount: row.get::<_, String>(3).parse().unwrap_or(0.0),
                        created_at: row.get(4),
                        submission_deadline: deadline,
                    }
                })
                .collect();
            HttpResponse::Ok().json(reports)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch application reports: {}", e)
        })),
    }
}

#[post("/application_reports")]
pub async fn create_application_report(
    pool: web::Data<DbPool>,
    data: web::Json<CreateApplicationReportRequest>,
) -> impl Responder {
    let client = match pool.pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }));
        }
    };

    let amount_val = data.amount;

    let result = client
        .query(
            "INSERT INTO application_reports (data_group, name, amount, submission_deadline) 
             VALUES ($1, $2, $3, $4) RETURNING id",
            &[
                &data.group_id,
                &data.name,
                &amount_val,
                &data.submission_deadline,
            ],
        )
        .await;

    match result {
        Ok(rows) => {
            let id: i32 = rows.first().map(|r| r.get(0)).unwrap_or(0);
            HttpResponse::Created().json(serde_json::json!({
                "id": id,
                "data_group": data.group_id,
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

    if data.name.is_none() && data.amount.is_none() && data.submission_deadline.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No fields to update"
        }));
    }

    let client = match pool.pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }));
        }
    };

    if let Some(ref name) = data.name {
        let result = client
            .execute(
                "UPDATE application_reports SET name = $1 WHERE id = $2",
                &[name, &id],
            )
            .await;

        if let Ok(0) = result {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Application report not found"
            }));
        }
    }

    if let Some(amount) = data.amount {
        let result = client
            .execute(
                "UPDATE application_reports SET amount = $1 WHERE id = $2",
                &[&amount, &id],
            )
            .await;

        if let Ok(0) = result {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Application report not found"
            }));
        }
    }

    if let Some(ref deadline) = data.submission_deadline {
        let result = client
            .execute(
                "UPDATE application_reports SET submission_deadline = $1 WHERE id = $2",
                &[deadline, &id],
            )
            .await;

        if let Ok(0) = result {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Application report not found"
            }));
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Application report updated successfully"
    }))
}

#[delete("/application_reports/{id}")]
pub async fn delete_application_report(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
) -> impl Responder {
    let id = path.into_inner();

    let group_id: Option<i32> = {
        let client = match pool.pool.get().await {
            Ok(c) => c,
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database connection error: {}", e)
                }));
            }
        };

        match client
            .query_opt(
                "SELECT data_group FROM application_reports WHERE id = $1",
                &[&id],
            )
            .await
        {
            Ok(Some(row)) => Some(row.get(0)),
            _ => None,
        }
    };

    let group_id = match group_id {
        Some(g) => g,
        None => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Application report not found"
            }));
        }
    };

    {
        let client = match pool.pool.get().await {
            Ok(c) => c,
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database connection error: {}", e)
                }));
            }
        };

        let result = client
            .execute(
                "UPDATE expenses SET application = NULL WHERE application = $1 AND data_group = $2",
                &[&id, &group_id],
            )
            .await;

        if let Err(e) = result {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to update expenses: {}", e)
            }));
        }
    }

    {
        let client = match pool.pool.get().await {
            Ok(c) => c,
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database connection error: {}", e)
                }));
            }
        };

        let result = client
            .execute("DELETE FROM application_reports WHERE id = $1", &[&id])
            .await;

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
