use crate::db::DbPool;
use crate::models::{Bill, BillQuery};
use crate::services::pdf_converter;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use rusqlite::params;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB

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
    query: web::Query<HashMap<String, String>>
) -> impl Responder {
    // Get group_id from query, default to 1
    let group_id = query.get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);
    
    // Get table names from data_groups
    let bills_table = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        match DbPool::get_table_names(&dg_conn, group_id) {
            Ok((_, bills_table)) => bills_table,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        }
    };
    
    let conn = pool.bills_conn.lock().unwrap();

    let sql = format!("SELECT no, filename, amount, date, Bargeldabhebung FROM {}", bills_table);
    let mut stmt = conn.prepare(&sql).unwrap();

    let mut rows = stmt.query([]).unwrap();
    let mut bills = Vec::new();

    while let Some(row) = rows.next().unwrap() {
        let amount: Option<f32> = row.get(2).unwrap();
        bills.push(Bill {
            id: row.get(0).unwrap(),
            filename: row.get(1).unwrap(),
            amount,
            date: row.get(3).unwrap(),
            Bargeldabhebung: row.get(4).unwrap(),
        });
    }

    HttpResponse::Ok().json(bills)
}

#[get("/bills/{id}")]
pub async fn get_bill(
    pool: web::Data<DbPool>, 
    path: web::Path<u8>,
    query: web::Query<HashMap<String, String>>
) -> impl Responder {
    let id = path.into_inner();
    
    // Get group_id from query, default to 1
    let group_id = query.get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);
    
    // Get table names from data_groups
    let bills_table = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        match DbPool::get_table_names(&dg_conn, group_id) {
            Ok((_, bills_table)) => bills_table,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        }
    };
    
    let conn = pool.bills_conn.lock().unwrap();

    let sql = format!("SELECT no, filename, amount, date, Bargeldabhebung FROM {} WHERE no = ?1", bills_table);
    let mut stmt = conn.prepare(&sql).unwrap();

    let mut rows = stmt.query(params![id]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let amount: Option<f32> = row.get(2).unwrap();
        let bill = Bill {
            id: row.get(0).unwrap(),
            filename: row.get(1).unwrap(),
            amount,
            date: row.get(3).unwrap(),
            Bargeldabhebung: row.get(4).unwrap(),
        };
        HttpResponse::Ok().json(bill)
    } else {
        HttpResponse::NotFound().body("Bill not found")
    }
}

#[put("/bills/{id}")]
pub async fn update_bill(
    pool: web::Data<DbPool>,
    path: web::Path<u8>,
    bill: web::Json<Bill>,
    query: web::Query<HashMap<String, String>>
) -> impl Responder {
    print!("PUT bills request");
    let id = path.into_inner();
    
    // Get group_id from query, default to 1
    let group_id = query.get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);
    
    // Get table names from data_groups
    let bills_table = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        match DbPool::get_table_names(&dg_conn, group_id) {
            Ok((_, bills_table)) => bills_table,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        }
    };
    
    let conn = pool.bills_conn.lock().unwrap();

    let sql = format!("UPDATE {} SET filename = ?1, amount = ?2, date = ?3 WHERE no = ?4", bills_table);
    conn.execute(
        &sql,
        params![&bill.filename, &bill.amount, &bill.date, id],
    )
    .expect("error updating db");

    HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Updated bill {}", id)
    }))
}

#[post("/bills/upload")]
pub async fn upload_bills(
    pool: web::Data<DbPool>,
    mut payload: Multipart,
) -> impl Responder {
    let mut group_id: Option<i32> = None;
    let mut files_to_process: Vec<(String, Vec<u8>)> = Vec::new();
    
    // Process all multipart fields in one pass
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = match field.content_disposition() {
            Some(cd) => cd,
            None => continue,
        };
        let field_name = content_disposition.get_name();
        
        match field_name {
            Some("group_id") => {
                // Read group_id value
                let mut value = String::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    value.push_str(&String::from_utf8_lossy(&chunk));
                }
                group_id = value.parse::<i32>().ok();
            }
            Some("files") => {
                // This is a file field
                if let Some(filename) = content_disposition.get_filename() {
                    let filename = sanitize_filename(filename);
                    let mut file_data = Vec::new();
                    
                    // Read file data
                    while let Ok(Some(chunk)) = field.try_next().await {
                        file_data.extend_from_slice(&chunk);
                        
                        // Check file size limit
                        if file_data.len() > MAX_FILE_SIZE {
                            break;
                        }
                    }
                    
                    files_to_process.push((filename, file_data));
                }
            }
            _ => {
                // Skip unknown fields
                while let Ok(Some(_)) = field.try_next().await {}
            }
        }
    }
    
    let group_id = match group_id {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "group_id is required"
            }));
        }
    };
    
    // Get storage path and bills table for this group
    let (storage_path, bills_table) = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        let path = match DbPool::get_bills_storage_path(&dg_conn, group_id) {
            Ok(p) => p,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        };
        let (_, table) = match DbPool::get_table_names(&dg_conn, group_id) {
            Ok(t) => t,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        };
        (path, table)
    };
    
    let base_path = format!("./public/{}", storage_path);
    
    // Process each file
    let mut results: Vec<UploadResult> = Vec::new();
    let mut next_bill_id: i32 = 1;
    
    // Get next available bill number
    {
        let conn = pool.bills_conn.lock().unwrap();
        let sql = format!("SELECT MAX(no) FROM {}", bills_table);
        if let Ok(max_id) = conn.query_row(&sql, [], |row| row.get::<_, Option<i32>>(0)) {
            if let Some(id) = max_id {
                next_bill_id = id + 1;
            }
        }
    }
    
    for (filename, file_data) in files_to_process {
        // Check file size
        if file_data.len() > MAX_FILE_SIZE {
            results.push(UploadResult {
                filename: filename.clone(),
                bill_id: None,
                success: false,
                error: Some(format!("File '{}' exceeds 10MB limit", filename)),
            });
            continue;
        }
        
        // Check file type (JPG, PNG, PDF)
        let ext = filename.split('.').last().unwrap_or("").to_lowercase();
        if !["jpg", "jpeg", "png", "pdf"].contains(&ext.as_str()) {
            results.push(UploadResult {
                filename: filename.clone(),
                bill_id: None,
                success: false,
                error: Some(format!("File '{}' has unsupported format (use JPG, PNG, or PDF)", filename)),
            });
            continue;
        }

        // Auto-convert PDF to JPG
        let final_filename;
        let final_file_data;
        
        if pdf_converter::should_convert_to_jpg(&filename) {
            match pdf_converter::convert_pdf_to_jpg_with_defaults(&file_data) {
                Ok(jpg_data) => {
                    final_filename = pdf_converter::replace_extension_with_jpg(&filename);
                    final_file_data = jpg_data;
                    log::info!("Successfully converted PDF '{}' to JPG '{}'", filename, final_filename);
                }
                Err(e) => {
                    log::warn!("PDF conversion failed for '{}': {}, keeping original PDF", filename, e);
                    final_filename = filename.clone();
                    final_file_data = file_data.clone();
                }
            }
        } else {
            final_filename = filename.clone();
            final_file_data = file_data.clone();
        }

        let file_path = format!("{}/{}", base_path, final_filename);
        
        // Check if file already exists (duplicate) - check final filename
        if Path::new(&file_path).exists() {
            results.push(UploadResult {
                filename: final_filename.clone(),
                bill_id: None,
                success: false,
                error: Some(format!("File '{}' already exists", final_filename)),
            });
            continue;
        }
        
        // Check file size
        if final_file_data.len() > MAX_FILE_SIZE {
            results.push(UploadResult {
                filename: final_filename.clone(),
                bill_id: None,
                success: false,
                error: Some(format!("File '{}' exceeds 10MB limit", final_filename)),
            });
            continue;
        }
        
        // Save file
        match std::fs::write(&file_path, &final_file_data) {
            Ok(_) => {
                // Insert into database
                let bill_id = next_bill_id;
                let conn = pool.bills_conn.lock().unwrap();
                let sql = format!(
                    "INSERT INTO {} (no, filename) VALUES (?1, ?2)",
                    bills_table
                );
                
                match conn.execute(&sql, params![bill_id, &final_filename]) {
                    Ok(_) => {
                        results.push(UploadResult {
                            filename: final_filename.clone(),
                            bill_id: Some(bill_id),
                            success: true,
                            error: None,
                        });
                        next_bill_id += 1;
                    }
                    Err(e) => {
                        // Rollback file creation
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
    
    // Calculate summary
    let success_count = results.iter().filter(|r| r.success).count();
    let error_count = results.len() - success_count;
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "group_id": group_id,
        "total_files": results.len(),
        "success_count": success_count,
        "error_count": error_count,
        "results": results,
    }))
}

/// Sanitize filename to prevent directory traversal and ensure safe names
fn sanitize_filename(filename: &str) -> String {
    // Remove path components (keep only filename)
    let filename = filename.split(&['/', '\\'][..]).last().unwrap_or(filename);
    
    // Replace problematic characters
    filename
        .replace("..", "_")
        .replace(' ', "_")
        .replace('\n', "")
        .replace('\r', "")
}

#[delete("/bills/{id}")]
pub async fn delete_bill(
    pool: web::Data<DbPool>,
    path: web::Path<u8>,
    query: web::Query<HashMap<String, String>>
) -> impl Responder {
    let id = path.into_inner() as i32;
    
    // Get group_id from query, default to 1
    let group_id = query.get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);
    
    // Get table names and storage path from data_groups
    let (expenses_table, bills_table, storage_path) = {
        let dg_conn = pool.data_groups_conn.lock().unwrap();
        let (exp_table, bill_table) = match DbPool::get_table_names(&dg_conn, group_id) {
            Ok(t) => t,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        };
        let path = match DbPool::get_bills_storage_path(&dg_conn, group_id) {
            Ok(p) => p,
            Err(_) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        };
        (exp_table, bill_table, path)
    };
    
    // Get the filename before deleting (for file removal)
    let filename_to_delete: Option<String> = {
        let conn = pool.bills_conn.lock().unwrap();
        let sql = format!("SELECT filename FROM {} WHERE no = ?1", bills_table);
        conn.query_row(&sql, params![id], |row| row.get(0)).ok()
    };
    
    // Reset expenses with this bill number to 0 (same data group only)
    {
        let conn = pool.expenses_conn.lock().unwrap();
        let sql = format!("UPDATE {} SET bill = 0 WHERE bill = ?1", expenses_table);
        if let Err(e) = conn.execute(&sql, params![id]) {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to reset expense bill numbers: {}", e)
            }));
        }
    }
    
    // Delete from bills table
    {
        let conn = pool.bills_conn.lock().unwrap();
        let sql = format!("DELETE FROM {} WHERE no = ?1", bills_table);
        match conn.execute(&sql, params![id]) {
            Ok(rows_affected) => {
                if rows_affected == 0 {
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
    
    // Delete the file from filesystem if it exists
    if let Some(filename) = filename_to_delete {
        let file_path = format!("./public/{}/{}", storage_path, filename);
        if Path::new(&file_path).exists() {
            if let Err(e) = std::fs::remove_file(&file_path) {
                // Log but don't fail - bill is already deleted from DB
                log::warn!("Failed to delete file {}: {}", file_path, e);
            }
        }
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Bill {} deleted successfully", id)
    }))
}