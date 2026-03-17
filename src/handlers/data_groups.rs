use actix_web::{get, post, web, HttpResponse, Responder};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use crate::db::DbPool;
use std::fs;

#[derive(Serialize)]
pub struct DataGroup {
    id: i32,
    name: String,
    group_type: String,
    created_at: String,
    expenses_table_name: String,
    bills_table_name: String,
    bills_storage_path: String,
}

#[derive(Deserialize)]
pub struct CreateDataGroupRequest {
    name: String,
    group_type: String, // "project" or "organization"
}

#[derive(Serialize)]
pub struct CreateDataGroupResponse {
    id: i32,
    name: String,
    group_type: String,
    expenses_table_name: String,
    bills_table_name: String,
    bills_storage_path: String,
    message: String,
}

/// Sanitize name for table naming (remove special chars, spaces to underscores)
fn sanitize_table_name(name: &str) -> String {
    name.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '_', "")
        .replace(' ', "_")
        .replace("__", "_")
}

/// Create expenses table with given name
fn create_expenses_table(conn: &rusqlite::Connection, table_name: &str) -> Result<(), rusqlite::Error> {
    let sql = format!(
        "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT,
            partner TEXT NOT NULL,
            amount TEXT NOT NULL,
            expense_type INTEGER NOT NULL,
            bill INTEGER NOT NULL,
            application INTEGER,
            cash BOOLEAN
        )",
        table_name
    );
    conn.execute(&sql, [])?;
    Ok(())
}

/// Create bills table with given name
fn create_bills_table(conn: &rusqlite::Connection, table_name: &str) -> Result<(), rusqlite::Error> {
    let sql = format!(
        "CREATE TABLE IF NOT EXISTS {} (
            no INTEGER PRIMARY KEY,
            filename TEXT NOT NULL,
            amount REAL,
            date TEXT,
            cash BOOLEAN
        )",
        table_name
    );
    conn.execute(&sql, [])?;
    Ok(())
}

#[get("/data_groups")]
pub async fn get_data_groups(pool: web::Data<DbPool>) -> impl Responder {
    let conn = pool.data_groups_conn.lock().unwrap();
    
    let mut stmt = conn.prepare(
        "SELECT id, name, type, created_at, expenses_table_name, bills_table_name, bills_storage_path 
         FROM data_groups ORDER BY created_at DESC"
    ).unwrap();
    
    let groups: Vec<DataGroup> = stmt.query_map([], |row| {
        Ok(DataGroup {
            id: row.get(0)?,
            name: row.get(1)?,
            group_type: row.get(2)?,
            created_at: row.get(3)?,
            expenses_table_name: row.get(4)?,
            bills_table_name: row.get(5)?,
            bills_storage_path: row.get(6)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    
    HttpResponse::Ok().json(groups)
}

#[post("/data_groups")]
pub async fn create_data_group(
    pool: web::Data<DbPool>,
    data: web::Json<CreateDataGroupRequest>,
) -> impl Responder {
    // Validate type
    if data.group_type != "project" && data.group_type != "organization" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Type must be either 'project' or 'organization'"
        }));
    }
    
    // Generate table names based on sanitized name
    let sanitized = sanitize_table_name(&data.name);
    let expenses_table = format!("expenses_{}", sanitized);
    let bills_table = format!("bills_{}", sanitized);
    
    // Generate storage path for bills (will be updated with actual ID after insert)
    let storage_path_prefix = format!("scanned_bills/TEMP_{}", sanitized);
    
    // Create tables in respective databases
    {
        let expenses_conn = pool.expenses_conn.lock().unwrap();
        if let Err(e) = create_expenses_table(&expenses_conn, &expenses_table) {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create expenses table: {}", e)
            }));
        }
    }
    
    {
        let bills_conn = pool.bills_conn.lock().unwrap();
        if let Err(e) = create_bills_table(&bills_conn, &bills_table) {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create bills table: {}", e)
            }));
        }
    }
    
    // Insert into data_groups
    let group_id: i64;
    {
        let conn = pool.data_groups_conn.lock().unwrap();
        match conn.execute(
            "INSERT INTO data_groups (name, type, expenses_table_name, bills_table_name, bills_storage_path) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![&data.name, &data.group_type, &expenses_table, &bills_table, &storage_path_prefix],
        ) {
            Ok(_) => {
                group_id = conn.last_insert_rowid();
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to insert data group: {}", e)
                }));
            }
        }
    }
    
    // Create the storage folder with the actual ID
    let final_storage_path = format!("scanned_bills/{}_{}", group_id, sanitized);
    if let Err(e) = fs::create_dir_all(&format!("./public/{}", final_storage_path)) {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create bills storage folder: {}", e)
        }));
    }
    
    // Update the storage path with the actual ID
    {
        let conn = pool.data_groups_conn.lock().unwrap();
        if let Err(e) = conn.execute(
            "UPDATE data_groups SET bills_storage_path = ?1 WHERE id = ?2",
            params![&final_storage_path, group_id],
        ) {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to update storage path: {}", e)
            }));
        }
    }
    
    HttpResponse::Created().json(CreateDataGroupResponse {
        id: group_id as i32,
        name: data.name.clone(),
        group_type: data.group_type.clone(),
        expenses_table_name: expenses_table,
        bills_table_name: bills_table,
        bills_storage_path: final_storage_path,
        message: format!("Data group '{}' created successfully", data.name),
    })
}