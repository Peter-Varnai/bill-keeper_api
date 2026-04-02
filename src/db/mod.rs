use deadpool_postgres::{Config, Pool};
use std::env;

pub struct DbPool {
    pub pool: Pool,
}

impl DbPool {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
        let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
        let password = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "".to_string());
        let db = env::var("POSTGRES_DB").unwrap_or_else(|_| "postgres".to_string());

        let config = Config {
            host: Some(host),
            port: Some(port.parse().unwrap_or(5432)),
            user: Some(user),
            password: Some(password),
            dbname: Some(db),
            ..Default::default()
        };

        let pool = config.create_pool(Some(deadpool_postgres::Runtime::Tokio1), tokio_postgres::NoTls)?;

        Ok(DbPool { pool })
    }

    pub async fn get_data_group(
        &self,
        group_id: i32,
    ) -> Result<DataGroup, Box<dyn std::error::Error>> {
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                "SELECT id, name, type, created_at, bills_storage_path 
                 FROM data_groups WHERE id = $1",
                &[&group_id],
            )
            .await?;
        
        Ok(DataGroup {
            id: row.get(0),
            name: row.get(1),
            group_type: row.get(2),
            created_at: row.get(3),
            bills_storage_path: row.get(4),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DataGroup {
    pub id: i32,
    pub name: String,
    pub group_type: String,
    pub created_at: String,
    pub bills_storage_path: String,
}