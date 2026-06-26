use std::process::Child;

use tokio::sync::OnceCell;

use super::config::SERVER_PORT;
use super::db;
use super::error::TestError;
use super::server;

struct AutoKillChild(Option<Child>);

impl Drop for AutoKillChild {
    fn drop(&mut self) {
        if let Some(mut c) = self.0.take() {
            c.kill().ok();
            c.wait().ok();
        }
    }
}

struct SuiteState {
    base_url: String,
    auth_token: String,
}

static SUITE: OnceCell<SuiteState> = OnceCell::const_new();

async fn get_suite() -> Result<&'static SuiteState, TestError> {
    SUITE
        .get_or_try_init(|| async move {
            db::suite_setup().await?;

            let child = server::start()?;
            std::thread::spawn(move || {
                let _guard = AutoKillChild(Some(child));
                std::thread::park();
            });

            let base_url = format!("http://localhost:{}", SERVER_PORT);
            server::wait_for_ready(&base_url).await?;

            let login_resp = reqwest::Client::new()
                .post(format!("{}/api/auth/login", base_url))
                .json(&serde_json::json!({"username": "test", "password": "test"}))
                .send()
                .await
                .map_err(|e| TestError::Io(std::io::Error::other(e.to_string())))?;

            let auth: serde_json::Value = login_resp
                .json()
                .await
                .map_err(|e| TestError::Io(std::io::Error::other(e.to_string())))?;

            let token = auth["token"]
                .as_str()
                .ok_or_else(|| {
                    TestError::Io(std::io::Error::other("no token in login response"))
                })?
                .to_string();

            Ok(SuiteState {
                base_url,
                auth_token: token,
            })
        })
        .await
}

static UNIQUE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn unique_suffix() -> u64 {
    UNIQUE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

fn make_client(token: &str) -> Result<reqwest::Client, TestError> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
            .map_err(|e| TestError::Io(std::io::Error::other(e.to_string())))?,
    );
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| TestError::Io(std::io::Error::other(e.to_string())))
}

pub async fn new_test_context() -> Result<(reqwest::Client, String, String, i32), TestError> {
    let suite = get_suite().await?;
    let client = make_client(&suite.auth_token)?;

    let suffix = unique_suffix();
    let group_name = format!("test_{:016x}", suffix);

    let create_resp = client
        .post(format!("{}/api/data_groups", suite.base_url))
        .json(&serde_json::json!({"name": group_name, "type": "project"}))
        .send()
        .await
        .map_err(|e| TestError::Io(std::io::Error::other(e.to_string())))?;

    let status = create_resp.status().as_u16();
    if status != 201 {
        let body = create_resp.text().await.unwrap_or_default();
        return Err(TestError::Io(std::io::Error::other(format!(
            "failed to create test data group (status {}): {}",
            status, body
        ))));
    }

    let result: serde_json::Value = create_resp
        .json()
        .await
        .map_err(|e| TestError::Io(std::io::Error::other(e.to_string())))?;

    let dg_id = result["id"]
        .as_i64()
        .ok_or_else(|| TestError::Io(std::io::Error::other("no id in create response")))?
        as i32;

    Ok((
        client,
        suite.base_url.clone(),
        group_name,
        dg_id,
    ))
}

pub async fn cleanup_test_context(client: &reqwest::Client, base_url: &str, dg_id: i32) -> Result<(), TestError> {
    let resp = client
        .delete(format!("{}/api/data_groups/{}", base_url, dg_id))
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => Ok(()),
        Ok(r) => {
            let status = r.status();
            let body = r.text().await.unwrap_or_default();
            eprintln!("cleanup warning (status {}): {}", status, body);
            Ok(())
        }
        Err(_) => Ok(()),
    }
}
