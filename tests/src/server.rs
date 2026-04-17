use std::process::{Child, Command, Stdio};

use super::config::{BINARY_PATH, TEST_PORT};

pub fn start() -> Result<Child, Box<dyn std::error::Error>> {
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
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(child)
}
