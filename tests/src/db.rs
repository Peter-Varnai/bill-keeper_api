use tokio_postgres::Client;

use super::config::db_config;
use super::error::TestError;

const TABLES_IN_ORDER: &[&str] = &["expenses", "bills", "application_reports", "data_groups"];

pub async fn drop_tables(client: &Client) -> Result<(), TestError> {
    for table in TABLES_IN_ORDER {
        client
            .execute(&format!("DROP TABLE IF EXISTS {}", table), &[])
            .await
            .map_err(TestError::Query)?;
    }
    Ok(())
}

pub async fn setup() -> Result<Client, TestError> {
    let config = db_config()
        .parse::<tokio_postgres::Config>()
        .map_err(|_| TestError::ConfigParse)?;

    let (client, connection) = config
        .connect(tokio_postgres::NoTls)
        .await
        .map_err(TestError::Connect)?;
    tokio::spawn(connection);

    drop_tables(&client).await?;

    let schema = std::fs::read_to_string("schema.sql").map_err(TestError::Io)?;
    client
        .simple_query(&schema)
        .await
        .map_err(TestError::Query)?;

    let seed = std::fs::read_to_string("seed_data.sql").map_err(TestError::Io)?;
    client.simple_query(&seed).await.map_err(TestError::Query)?;

    Ok(client)
}

pub async fn teardown() -> Result<(), TestError> {
    let config = db_config()
        .parse::<tokio_postgres::Config>()
        .map_err(|_| TestError::ConfigParse)?;

    let (client, connection) = config
        .connect(tokio_postgres::NoTls)
        .await
        .map_err(TestError::Connect)?;
    tokio::spawn(connection);

    drop_tables(&client).await?;

    Ok(())
}
