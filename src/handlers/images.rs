use crate::db::DbPool;
use actix_web::{get, web, HttpResponse, Responder};
use std::collections::HashMap;
use std::path::Path;

#[get("/images/{filename}")]
pub async fn get_image(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let filename = path.into_inner();

    let group_id = query
        .get("group_id")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);

    let storage_path = {
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
                "SELECT bills_storage_path FROM data_groups WHERE id = $1",
                &[&group_id],
            )
            .await
        {
            Ok(Some(row)) => row.get::<_, String>(0),
            _ => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", group_id)
                }));
            }
        }
    };

    let file_path = format!("./public/{}/{}", storage_path, filename);

    if Path::new(&file_path).exists() {
        match std::fs::read(&file_path) {
            Ok(data) => {
                let content_type = if filename.ends_with(".png") {
                    "image/png"
                } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
                    "image/jpeg"
                } else {
                    "application/octet-stream"
                };

                HttpResponse::Ok().content_type(content_type).body(data)
            }
            Err(_) => HttpResponse::InternalServerError().body("Failed to read file"),
        }
    } else {
        HttpResponse::NotFound().body("File not found")
    }
}