use std::process::{Child, Command, Stdio};

use super::config::{BINARY_PATH, TEST_PORT};
use super::error::TestError;

pub fn start() -> Result<Child, TestError> {
    let child = Command::new(BINARY_PATH)
        .env_clear()
        .env("TESTING", "true")
        .env("PORT", TEST_PORT.to_string())
        .env("POSTGRES_HOST", std::env::var("POSTGRES_HOST").unwrap())
        .env("POSTGRES_PORT", std::env::var("POSTGRES_PORT").unwrap())
        .env("POSTGRES_DB", std::env::var("POSTGRES_DB").unwrap())
        .env("POSTGRES_USER", std::env::var("POSTGRES_USER").unwrap())
        .env(
            "POSTGRES_PASSWORD",
            std::env::var("POSTGRES_PASSWORD").unwrap(),
        )
        .current_dir("/home/peter/projects/bill_keeper/api-service")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(TestError::Io)?;

    Ok(child)
}
