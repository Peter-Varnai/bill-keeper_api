use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestError {
    #[error("Failed to parse database config")]
    ConfigParse,

    #[error("Failed to connect to database: {0}")]
    Connect(#[source] tokio_postgres::Error),

    #[error("Database error: {0}")]
    Query(#[source] tokio_postgres::Error),

    #[error("IO error reading file: {0}")]
    Io(#[source] std::io::Error),
}
