use actix_web::{web, HttpMessage, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

use crate::db::DbPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub username: String,
    pub exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
}

fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 6 {
        return Err("Password must be at least 6 characters".to_string());
    }
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    if !has_uppercase {
        return Err("Password must contain at least one uppercase letter".to_string());
    }
    if !has_digit {
        return Err("Password must contain at least one number".to_string());
    }
    Ok(())
}

pub fn create_token(user_id: i32, username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "fallback-secret".to_string());
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "fallback-secret".to_string());
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

pub async fn register(
    pool: web::Data<DbPool>,
    body: web::Json<AuthRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    if body.username.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Username is required"
        })));
    }

    if let Err(e) = validate_password(&body.password) {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })));
    }

    let password_hash = hash(&body.password, DEFAULT_COST)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to hash password"))?;

    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return Ok(response),
    };

    let result = client
        .query_one(
            "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING id",
            &[&body.username.trim(), &password_hash],
        )
        .await;

    match result {
        Ok(row) => {
            let user_id: i32 = row.get(0);
            let token = create_token(user_id, body.username.trim()).map_err(|_| {
                actix_web::error::ErrorInternalServerError("Failed to create token")
            })?;

            Ok(HttpResponse::Created().json(AuthResponse {
                token,
                user: UserInfo {
                    id: user_id,
                    username: body.username.trim().to_string(),
                },
            }))
        }
        Err(e) => {
            crate::db::log_db_error("register", &e);
            if let Some(db_err) = e.as_db_error() {
                if db_err.code().code() == "23505" {
                    return Ok(HttpResponse::Conflict().json(serde_json::json!({
                        "error": "Username already exists"
                    })));
                }
            }
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to register: {}", e)
            })))
        }
    }
}

pub async fn login(
    pool: web::Data<DbPool>,
    body: web::Json<AuthRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(response) => return Ok(response),
    };

    let result = client
        .query_opt(
            "SELECT id, username, password_hash FROM users WHERE username = $1",
            &[&body.username],
        )
        .await;

    match result {
        Ok(Some(row)) => {
            let user_id: i32 = row.get(0);
            let username: String = row.get(1);
            let password_hash: String = row.get(2);

            match verify(&body.password, &password_hash) {
                Ok(true) => {
                    let token = create_token(user_id, &username).map_err(|_| {
                        actix_web::error::ErrorInternalServerError("Failed to create token")
                    })?;

                    Ok(HttpResponse::Ok().json(AuthResponse {
                        token,
                        user: UserInfo {
                            id: user_id,
                            username,
                        },
                    }))
                }
                _ => Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Invalid username or password"
                }))),
            }
        }
        Ok(None) => Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid username or password"
        }))),
        Err(e) => {
            crate::db::log_db_error("login", &e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Login failed: {}", e)
            })))
        }
    }
}

pub async fn me(req: actix_web::HttpRequest) -> HttpResponse {
    let user_id = match req.extensions().get::<i32>() {
        Some(id) => *id,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }));
        }
    };

    let client = match req
        .app_data::<web::Data<DbPool>>()
        .map(|pool| pool.get_ref())
    {
        Some(pool) => match pool.get_client().await {
            Ok(c) => c,
            Err(response) => return response,
        },
        None => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database pool not available"
            }));
        }
    };

    let result = client
        .query_opt("SELECT username FROM users WHERE id = $1", &[&user_id])
        .await;

    match result {
        Ok(Some(row)) => {
            let username: String = row.get(0);
            HttpResponse::Ok().json(UserInfo {
                id: user_id,
                username,
            })
        }
        _ => HttpResponse::NotFound().json(serde_json::json!({
            "error": "User not found"
        })),
    }
}

pub fn get_user_id(req: &actix_web::HttpRequest) -> Result<i32, HttpResponse> {
    req.extensions().get::<i32>().copied().ok_or_else(|| {
        HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Not authenticated"
        }))
    })
}
