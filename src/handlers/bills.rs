use crate::db::DbPool;
use crate::helpers::get_data_group_url;
use crate::models::Bill;
use crate::services::pdf_converter;
use actix_multipart::Multipart;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use futures_util::TryStreamExt;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

#[derive(Serialize)]
struct UploadResult {
    filename: String,
    bill_id: Option<i32>,
    success: bool,
    error: Option<String>,
}

#[get("/bills")]
pub async fn get_bills(
    pool: web::Data<DbPool>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let data_group = match get_data_group_url(&query) {
        Ok(c) => c,
        Err(response) => return response,
    };

    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return response,
    };

    let result = client
        .query(
            "SELECT id, data_group, filename, amount::text, date, is_cash 
             FROM bills WHERE data_group = $1 ORDER BY id",
            &[&data_group],
        )
        .await;

    match result {
        Ok(rows) => {
            let bills: Vec<Bill> = rows
                .iter()
                .map(|row| Bill {
                    id: row.get(0),
                    data_group: row.get(1),
                    filename: row.get(2),
                    amount: row.get::<_, Option<String>>(3).and_then(|s| s.parse().ok()),
                    date: row.get(4),
                    is_cash: row.get(5),
                })
                .collect();
            HttpResponse::Ok().json(bills)
        }
        Err(e) => {
            crate::db::log_db_error("get_bills", &e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch bills: {}", e)
            }))
        }
    }
}

#[get("/bills/{id}")]
pub async fn get_bill(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let id = path.into_inner();
    let data_group = match get_data_group_url(&query) {
        Ok(c) => c,
        Err(response) => return response,
    };

    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return response,
    };

    let result = client
        .query(
            "SELECT id, data_group, filename, amount::text, date, is_cash 
             FROM bills WHERE id = $1 AND data_group = $2",
            &[&id, &data_group],
        )
        .await;

    match result {
        Ok(rows) => {
            if let Some(row) = rows.first() {
                let bill = Bill {
                    id: row.get(0),
                    data_group: row.get(1),
                    filename: row.get(2),
                    amount: row.get::<_, Option<String>>(3).and_then(|s| s.parse().ok()),
                    date: row.get(4),
                    is_cash: row.get(5),
                };
                HttpResponse::Ok().json(bill)
            } else {
                HttpResponse::NotFound().body("Bill not found")
            }
        }
        Err(e) => {
            crate::db::log_db_error("get_bill", &e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch bill: {}", e)
            }))
        }
    }
}

use crate::models::requests::BillUpdateRequest;

#[put("/bills")]
pub async fn update_bill(
    pool: web::Data<DbPool>,
    bill: web::Json<BillUpdateRequest>,
) -> impl Responder {
    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return response,
    };

    let result = client
        .execute(
            "UPDATE bills SET filename = $1, amount = $2, date = $3, is_cash = $4 
             WHERE id = $5 AND data_group = $6",
            &[
                &bill.filename,
                &bill.amount,
                &bill.date,
                &bill.is_cash,
                &bill.id,
                &bill.data_group,
            ],
        )
        .await;

    match result {
        Ok(rows) => {
            if rows > 0 {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": format!("Updated bill {}", bill.id)
                }))
            } else {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Bill {} not found", bill.id)
                }))
            }
        }
        Err(e) => {
            crate::db::log_db_error("update_bill", &e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to update bill: {}", e)
            }))
        }
    }
}

#[post("/bills/upload")]
pub async fn upload_bills(pool: web::Data<DbPool>, mut payload: Multipart) -> impl Responder {
    let mut data_group: Option<i32> = None;
    let mut files_to_process: Vec<(String, Vec<u8>)> = Vec::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = match field.content_disposition() {
            Some(cd) => cd,
            None => continue,
        };
        let field_name = content_disposition.get_name();

        match field_name {
            Some("data_group") => {
                let mut value = String::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    value.push_str(&String::from_utf8_lossy(&chunk));
                }
                data_group = value.parse::<i32>().ok();
            }
            Some("files") => {
                if let Some(filename) = content_disposition.get_filename() {
                    let filename = sanitize_filename(filename);
                    let mut file_data = Vec::new();

                    while let Ok(Some(chunk)) = field.try_next().await {
                        file_data.extend_from_slice(&chunk);
                        if file_data.len() > MAX_FILE_SIZE {
                            break;
                        }
                    }

                    files_to_process.push((filename, file_data));
                }
            }
            _ => while let Ok(Some(_)) = field.try_next().await {},
        }
    }

    let data_group = match data_group {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "data_group is required"
            }));
        }
    };

    let storage_path = {
        let client = match pool.get_client().await {
            Ok(c) => c,
            Err(response) => return response,
        };

        match client
            .query_opt(
                "SELECT bills_storage_path FROM data_groups WHERE id = $1",
                &[&data_group],
            )
            .await
        {
            Ok(Some(row)) => row.get::<_, String>(0),
            _ => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", data_group)
                }));
            }
        }
    };

    let base_path = format!("./public/{}", storage_path);

    let mut results: Vec<UploadResult> = Vec::new();

    for (filename, file_data) in files_to_process {
        if file_data.len() > MAX_FILE_SIZE {
            results.push(UploadResult {
                filename: filename.clone(),
                bill_id: None,
                success: false,
                error: Some(format!("File '{}' exceeds 10MB limit", filename)),
            });
            continue;
        }

        let ext = filename.split('.').last().unwrap_or("").to_lowercase();
        if !["jpg", "jpeg", "png", "pdf"].contains(&ext.as_str()) {
            results.push(UploadResult {
                filename: filename.clone(),
                bill_id: None,
                success: false,
                error: Some(format!(
                    "File '{}' has unsupported format (use JPG, PNG, or PDF)",
                    filename
                )),
            });
            continue;
        }

        let final_filename;
        let final_file_data;

        if pdf_converter::should_convert_to_jpg(&filename) {
            match pdf_converter::convert_pdf_to_jpg_with_defaults(&file_data) {
                Ok(jpg_data) => {
                    final_filename = pdf_converter::replace_extension_with_jpg(&filename);
                    final_file_data = jpg_data;
                    log::info!(
                        "Successfully converted PDF '{}' to JPG '{}'",
                        filename,
                        final_filename
                    );
                }
                Err(e) => {
                    log::warn!(
                        "PDF conversion failed for '{}': {}, keeping original PDF",
                        filename,
                        e
                    );
                    final_filename = filename.clone();
                    final_file_data = file_data.clone();
                }
            }
        } else {
            final_filename = filename.clone();
            final_file_data = file_data.clone();
        }

        let file_path = format!("{}/{}", base_path, final_filename);

        if Path::new(&file_path).exists() {
            results.push(UploadResult {
                filename: final_filename.clone(),
                bill_id: None,
                success: false,
                error: Some(format!("File '{}' already exists", final_filename)),
            });
            continue;
        }

        if final_file_data.len() > MAX_FILE_SIZE {
            results.push(UploadResult {
                filename: final_filename.clone(),
                bill_id: None,
                success: false,
                error: Some(format!("File '{}' exceeds 10MB limit", final_filename)),
            });
            continue;
        }

        match std::fs::write(&file_path, &final_file_data) {
            Ok(_) => {
                let client = match pool.get_client().await {
                    Ok(c) => c,
                    Err(_response) => {
                        let _ = std::fs::remove_file(&file_path);
                        results.push(UploadResult {
                            filename: final_filename.clone(),
                            bill_id: None,
                            success: false,
                            error: Some("Database connection error".to_string()),
                        });
                        continue;
                    }
                };

                let result = client
                    .query(
                        "INSERT INTO bills (data_group, filename) VALUES ($1, $2) RETURNING id",
                        &[&data_group, &final_filename],
                    )
                    .await;

                match result {
                    Ok(rows) => {
                        let bill_id: i32 = rows.first().map(|r| r.get(0)).unwrap_or(0);
                        results.push(UploadResult {
                            filename: final_filename.clone(),
                            bill_id: Some(bill_id),
                            success: true,
                            error: None,
                        });
                    }
                    Err(e) => {
                        let _ = std::fs::remove_file(&file_path);
                        results.push(UploadResult {
                            filename: final_filename.clone(),
                            bill_id: None,
                            success: false,
                            error: Some(format!("Database error: {}", e)),
                        });
                    }
                }
            }
            Err(e) => {
                results.push(UploadResult {
                    filename: final_filename.clone(),
                    bill_id: None,
                    success: false,
                    error: Some(format!("Failed to save file: {}", e)),
                });
            }
        }
    }

    let success_count = results.iter().filter(|r| r.success).count();
    let error_count = results.len() - success_count;

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data_group": data_group,
        "total_files": results.len(),
        "success_count": success_count,
        "error_count": error_count,
        "results": results,
    }))
}

fn sanitize_filename(filename: &str) -> String {
    let filename = filename.split(&['/', '\\'][..]).last().unwrap_or(filename);
    filename
        .replace("..", "_")
        .replace(' ', "_")
        .replace('\n', "")
        .replace('\r', "")
}

#[delete("/bills/{id}")]
pub async fn delete_bill(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let id = path.into_inner();
    let data_group = match get_data_group_url(&query) {
        Ok(c) => c,
        Err(response) => return response,
    };

    let storage_path = {
        let client = match pool.get_client().await {
            Ok(c) => c,
            Err(response) => return response,
        };

        match client
            .query_opt(
                "SELECT bills_storage_path FROM data_groups WHERE id = $1",
                &[&data_group],
            )
            .await
        {
            Ok(Some(row)) => Some(row.get::<_, String>(0)),
            _ => None,
        }
    };

    let filename_to_delete: Option<String> = {
        let client = match pool.get_client().await {
            Ok(c) => c,
            Err(response) => return response,
        };

        match client
            .query_opt(
                "SELECT filename FROM bills WHERE id = $1 AND data_group = $2",
                &[&id, &data_group],
            )
            .await
        {
            Ok(Some(row)) => row.get(0),
            _ => None,
        }
    };

    {
        let client = match pool.get_client().await {
            Ok(c) => c,
            Err(response) => return response,
        };

        match client
            .execute(
                "UPDATE expenses SET bill = NULL WHERE bill = $1 AND data_group = $2",
                &[&id, &data_group],
            )
            .await
        {
            Ok(_) => {}
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to reset expense bill numbers: {}", e)
                }));
            }
        }
    }

    {
        let client = match pool.get_client().await {
            Ok(c) => c,
            Err(response) => return response,
        };

        match client
            .execute(
                "DELETE FROM bills WHERE id = $1 AND data_group = $2",
                &[&id, &data_group],
            )
            .await
        {
            Ok(rows) => {
                if rows == 0 {
                    return HttpResponse::NotFound().json(serde_json::json!({
                        "error": format!("Bill {} not found", id)
                    }));
                }
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to delete bill: {}", e)
                }));
            }
        }
    }

    if let Some(filename) = filename_to_delete {
        if let Some(path) = storage_path {
            let file_path = format!("./public/{}/{}", path, filename);
            if Path::new(&file_path).exists() {
                if let Err(e) = std::fs::remove_file(&file_path) {
                    log::warn!("Failed to delete file {}: {}", file_path, e);
                }
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Bill {} deleted successfully", id)
    }))
}
