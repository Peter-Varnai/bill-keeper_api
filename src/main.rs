use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
use dotenv::dotenv;
use log::info;
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::fs::OpenOptions;

mod db;
mod handlers;
mod helpers;
mod middleware;
mod models;
mod routes;
mod services;

use db::DbPool;
use middleware::logging::RequestLogger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

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
            .allowed_headers(vec![http::header::CONTENT_TYPE, http::header::ACCEPT])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(RequestLogger)
            .wrap(cors)
            .app_data(db_pool.clone())
            .configure(routes::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

