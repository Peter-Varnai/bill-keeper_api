use std::process::{Child, Command, Stdio};
use std::time::Duration;

use super::config::{BINARY_PATH, SERVER_PORT};
use super::error::TestError;

pub fn start() -> Result<Child, TestError> {
    let child = Command::new(BINARY_PATH)
        .env_clear()
        .env("TESTING", "true")
        .env("PORT", SERVER_PORT.to_string())
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

pub async fn wait_for_ready(_base_url: &str) -> Result<(), TestError> {
    let addr = format!("127.0.0.1:{}", SERVER_PORT);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);

    loop {
        if tokio::time::Instant::now() > deadline {
            return Err(TestError::Io(std::io::Error::other(
                "server did not become ready within 10s",
            )));
        }

        if tokio::net::TcpStream::connect(&addr).await.is_ok() {
            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
