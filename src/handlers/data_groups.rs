use crate::db::DbPool;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
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
        Err(e) => {
            crate::db::log_db_error("get_data_groups", &e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch data groups: {}", e)
            }))
        }
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
        Err(e) => {
            crate::db::log_db_error("create_data_group", &e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create data group: {}", e)
            }))
        }
    }
}

#[derive(Serialize)]
pub struct DeleteDataGroupResponse {
    pub success: bool,
    pub message: String,
    pub deleted_counts: DeletedCounts,
}

#[derive(Serialize)]
pub struct DeletedCounts {
    pub expenses: usize,
    pub bills: usize,
    pub application_reports: usize,
    pub utility_data: usize,
    pub data_group: bool,
    pub storage_folder: bool,
}

#[delete("/data_groups/{id}")]
pub async fn delete_data_group(pool: web::Data<DbPool>, path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();

    let client = match pool.pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }));
        }
    };

    let storage_path_result = client
        .query_opt(
            "SELECT bills_storage_path FROM data_groups WHERE id = $1",
            &[&id],
        )
        .await;

    let storage_path = match storage_path_result {
        Ok(Some(row)) => Some(row.get::<_, String>(0)),
        _ => None,
    };

    let expenses_result = client
        .execute("DELETE FROM expenses WHERE data_group = $1", &[&id])
        .await;
    let expenses_count = match expenses_result {
        Ok(count) => count as usize,
        Err(e) => {
            crate::db::log_db_error("delete_data_group_expenses", &e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to delete expenses: {}", e)
            }));
        }
    };

    let bills_result = client
        .execute("DELETE FROM bills WHERE data_group = $1", &[&id])
        .await;
    let bills_count = match bills_result {
        Ok(count) => count as usize,
        Err(e) => {
            crate::db::log_db_error("delete_data_group_bills", &e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to delete bills: {}", e)
            }));
        }
    };

    let app_reports_result = client
        .execute(
            "DELETE FROM application_reports WHERE data_group = $1",
            &[&id],
        )
        .await;
    let app_reports_count = match app_reports_result {
        Ok(count) => count as usize,
        Err(e) => {
            crate::db::log_db_error("delete_data_group_app_reports", &e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to delete application reports: {}", e)
            }));
        }
    };

    let utility_result = client
        .execute("DELETE FROM utility_data WHERE data_group = $1", &[&id])
        .await;
    let utility_count = match utility_result {
        Ok(count) => count as usize,
        Err(e) => {
            crate::db::log_db_error("delete_data_group_utility", &e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to delete utility data: {}", e)
            }));
        }
    };

    let data_group_result = client
        .execute("DELETE FROM data_groups WHERE id = $1", &[&id])
        .await;
    let data_group_deleted = match data_group_result {
        Ok(count) => count > 0,
        Err(e) => {
            crate::db::log_db_error("delete_data_group", &e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to delete data group: {}", e)
            }));
        }
    };

    let mut storage_folder_deleted = false;
    if let Some(path) = storage_path {
        let full_path = format!("./public/{}", path);
        if fs::remove_dir_all(&full_path).is_ok() {
            storage_folder_deleted = true;
        }
    }

    HttpResponse::Ok().json(DeleteDataGroupResponse {
        success: true,
        message: format!("Data group {} deleted successfully", id),
        deleted_counts: DeletedCounts {
            expenses: expenses_count,
            bills: bills_count,
            application_reports: app_reports_count,
            utility_data: utility_count,
            data_group: data_group_deleted,
            storage_folder: storage_folder_deleted,
        },
    })
}
