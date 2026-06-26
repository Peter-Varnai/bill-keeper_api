use std::process::{Child, Command, Stdio};

use super::config::BINARY_PATH;
use super::error::TestError;

pub fn start(port: u16) -> Result<Child, TestError> {
    let child = Command::new(BINARY_PATH)
        .env_clear()
        .env("TESTING", "true")
        .env("PORT", port.to_string())
        .env("POSTGRES_HOST", std::env::var("POSTGRES_HOST").unwrap())
        .env("POSTGRES_PORT", std::env::var("POSTGRES_PORT").unwrap())
        .env("POSTGRES_DB", std::env::var("POSTGRES_DB").unwrap())
        .env("POSTGRES_USER", std::env::var("POSTGRES_USER").unwrap())
        .env(
            "POSTGRES_PASSWORD",
            std::env::var("POSTGRES_PASSWORD").unwrap(),
        )
        .env(
            "JWT_SECRET",
            std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret".to_string()),
        )
        .current_dir("/home/peter/projects/bill_keeper/api-service")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(TestError::Io)?;

    Ok(child)
}
