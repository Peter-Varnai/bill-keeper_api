use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
use log::info;
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::env;
use std::fs::OpenOptions;

mod auth;
mod db;
mod handlers;
mod helpers;
mod middleware;
mod models;
mod routes;
mod services;

use db::DbPool;
use middleware::auth::AuthMiddleware;
use middleware::logging::RequestLogger;

async fn ensure_default_admin(pool: &DbPool) {
    let client = match pool.get_client().await {
        Ok(c) => c,
        Err(_) => return,
    };

    let count = client
        .query_one("SELECT COUNT(*) FROM users", &[])
        .await
        .map(|r| r.get::<_, i64>(0))
        .unwrap_or(0);

    if count == 0 {
        let password_hash = bcrypt::hash("admin", bcrypt::DEFAULT_COST).unwrap_or_default();
        let result = client
            .execute(
                "INSERT INTO users (id, username, password_hash) VALUES (1, 'admin', $1)",
                &[&password_hash],
            )
            .await;

        match result {
            Ok(_) => info!("Default admin user created (username: admin, password: admin)"),
            Err(e) => log::error!("Failed to create default admin user: {}", e),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let testing = env::var("TESTING") == Ok("true".to_string());
    #[cfg(debug_assertions)]
    if !testing {
        dotenv::from_filename(".env").ok();
    }

    std::fs::create_dir_all("logs").unwrap_or_else(|e| {
        eprintln!("Failed to create logs directory: {}", e);
    });

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/app.log")
        .expect("Failed to open log file");

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        ),
        WriteLogger::new(LevelFilter::Debug, Config::default(), log_file),
    ])
    .expect("Failed to initialize logging");

    info!("Logging initialized. Starting server...");

    let db_pool = match DbPool::new().await {
        Ok(pool) => {
            info!("Successfully connected to PostgreSQL database");
            ensure_default_admin(&pool).await;
            web::Data::new(pool)
        }
        Err(e) => {
            eprintln!("Failed to connect to PostgreSQL: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Database connection failed: {}", e),
            ));
        }
    };

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS", "PATCH"])
            .allowed_headers(vec![
                http::header::CONTENT_TYPE,
                http::header::ACCEPT,
                http::header::AUTHORIZATION,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(RequestLogger)
            .wrap(AuthMiddleware)
            .wrap(cors)
            .app_data(db_pool.clone())
            .configure(routes::config)
    })
    .bind((
        "127.0.0.1",
        env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap(),
    ))?
    .run()
    .await
}
