pub const TEST_PORT: u16 = 8090;
pub const BINARY_PATH: &str = "./target/debug/bills_to_db";

pub fn load_env() {
    dotenv::from_filename(".env.test").ok();
}

pub fn db_config() -> String {
    format!(
        "host={} port={} user={} password={} dbname={}",
        std::env::var("POSTGRES_HOST").unwrap(),
        std::env::var("POSTGRES_PORT").unwrap(),
        std::env::var("POSTGRES_USER").unwrap(),
        std::env::var("POSTGRES_PASSWORD").unwrap(),
        std::env::var("POSTGRES_DB").unwrap()
    )
}
