use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::db::DbPool;
use std::fs;

#[derive(Serialize)]
pub struct DataGroup {
    id: i32,
    name: String,
    #[serde(rename = "type")]
    group_type: String,
    created_at: String,
    bills_storage_path: String,
}

#[derive(Deserialize)]
pub struct CreateDataGroupRequest {
    name: String,
    #[serde(rename = "type")]
    group_type: String,
}

#[derive(Serialize)]
pub struct CreateDataGroupResponse {
    id: i32,
    name: String,
    #[serde(rename = "type")]
    group_type: String,
    bills_storage_path: String,
    message: String,
}

#[get("/data_groups")]
pub async fn get_data_groups(pool: web::Data<DbPool>) -> impl Responder {
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
            "SELECT id, name, type, created_at, bills_storage_path 
             FROM data_groups ORDER BY created_at DESC",
            &[],
        )
        .await;

    match result {
        Ok(rows) => {
            let groups: Vec<DataGroup> = rows
                .iter()
                .map(|row| DataGroup {
                    id: row.get(0),
                    name: row.get(1),
                    group_type: row.get(2),
                    created_at: row.get(3),
                    bills_storage_path: row.get(4),
                })
                .collect();
            HttpResponse::Ok().json(groups)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch data groups: {}", e)
        })),
    }
}

#[post("/data_groups")]
pub async fn create_data_group(
    pool: web::Data<DbPool>,
    data: web::Json<CreateDataGroupRequest>,
) -> impl Responder {
    if data.group_type != "project" && data.group_type != "organization" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Type must be either 'project' or 'organization'"
        }));
    }

    let storage_path = format!("pdf_imgs/{}", data.name.to_lowercase().replace(' ', "_"));

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
            "INSERT INTO data_groups (name, type, bills_storage_path) 
             VALUES ($1, $2, $3) RETURNING id",
            &[&data.name, &data.group_type, &storage_path],
        )
        .await;

    match result {
        Ok(rows) => {
            let id: i32 = rows.first().map(|r| r.get(0)).unwrap_or(0);

            if let Err(e) = fs::create_dir_all(&format!("./public/{}", storage_path)) {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to create bills storage folder: {}", e)
                }));
            }

            HttpResponse::Created().json(CreateDataGroupResponse {
                id,
                name: data.name.clone(),
                group_type: data.group_type.clone(),
                bills_storage_path: storage_path,
                message: format!("Data group '{}' created successfully", data.name),
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create data group: {}", e)
        })),
    }
}