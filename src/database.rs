use log::info;
use sqlx::{PgPool, migrate::MigrateError, postgres::PgPoolOptions};
use std::{
    env::{self, VarError},
    error::Error,
    fmt,
};

pub async fn init_db() -> Result<PgPool, DbError> {
    info!("Starting database initialization");
    info!("Getting DATABASE_URL env variable");
    let db_url = env::var("DATABASE_URL")?;

    info!("Connecting to the database");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    info!("Running migrations");
    sqlx::migrate!().run(&pool).await?;

    info!("Database initialized");
    Ok(pool)
}

#[derive(Debug)]
pub enum DbError {
    EnvVar(VarError),
    Connection(sqlx::Error),
    Migration(MigrateError),
}

impl From<VarError> for DbError {
    fn from(value: VarError) -> Self {
        DbError::EnvVar(value)
    }
}

impl From<sqlx::Error> for DbError {
    fn from(value: sqlx::Error) -> Self {
        DbError::Connection(value)
    }
}

impl From<MigrateError> for DbError {
    fn from(value: MigrateError) -> Self {
        DbError::Migration(value)
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::EnvVar(e) => write!(f, "Database environment variable error: {}", e),
            DbError::Connection(e) => write!(f, "Database connection error: {}", e),
            DbError::Migration(e) => write!(f, "Database migration failed: {}", e),
        }
    }
}

impl Error for DbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DbError::EnvVar(e) => Some(e),
            DbError::Connection(e) => Some(e),
            DbError::Migration(e) => Some(e),
        }
    }
}
