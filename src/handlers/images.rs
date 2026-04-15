use crate::db::DbPool;
use crate::helpers::get_data_group_url;
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
            Ok(Some(row)) => row.get::<_, String>(0),
            _ => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Data group {} not found", data_group)
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

