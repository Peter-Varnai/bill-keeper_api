use tokio_postgres::Client;

use super::config::db_config;

const tables: &[&str] = &["expenses", "bills", "application_reports", "data_groups"];

pub async fn drop_tables(client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for table in tables {
        client
            .execute(&format!("DROP TABLE IF EXISTS {}", table), &[])
            .await?;
    }
    Ok(())
}

pub async fn setup() -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
    let config = db_config().parse::<tokio_postgres::Config>()?;
    let (client, connection) = config.connect(tokio_postgres::NoTls).await?;
    tokio::spawn(connection);

    drop_tables(&client).await?;

    let schema = std::fs::read_to_string("schema.sql")?;
    client.simple_query(&schema).await?;

    let seed = std::fs::read_to_string("seed_data.sql")?;
    client.simple_query(&seed).await?;

    Ok(client)
}

pub async fn teardown() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = db_config().parse::<tokio_postgres::Config>()?;
    let (client, connection) = config.connect(tokio_postgres::NoTls).await?;
    tokio::spawn(connection);

    drop_tables(&client).await?;

    Ok(())
}

