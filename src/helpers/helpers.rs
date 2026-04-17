use std::collections::HashMap;

use actix_web::{web, HttpResponse};
use chrono::NaiveDate;

// pub fn parse_date_or_panic(date_opt: Option<String>) -> Option<NaiveDate> {
//     date_opt.map(|d| {
//         NaiveDate::parse_from_str(&d, "%Y-%m-%d")
//             .expect("Failed to parse date: expected format YYYY-MM-DD")
//     })
// }

pub fn get_data_group_req(data_group: Option<i32>) -> Result<i32, HttpResponse> {
    match data_group {
        Some(c) => Ok(c),
        None => Err(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "No data group found in request!"
        }))),
    }
}

pub fn get_data_group_url(
    query: &web::Query<HashMap<String, String>>,
) -> Result<i32, HttpResponse> {
    query
        .get("data_group")
        .and_then(|v| v.parse::<i32>().ok())
        .ok_or_else(|| {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "No data group found in request!"
            }))
        })
}

pub fn sanitize_filename(filename: &str) -> String {
    let filename = filename.split(&['/', '\\'][..]).last().unwrap_or(filename);
    filename
        .replace("..", "_")
        .replace(' ', "_")
        .replace('\n', "")
        .replace('\r', "")
}
