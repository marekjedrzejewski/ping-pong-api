use std::{env::VarError, error::Error, fmt};

use sqlx::migrate::MigrateError;

#[derive(Debug)]
pub enum DbError {
    EnvVar(VarError),
    Connection(sqlx::Error),
    Migration(MigrateError),
    Decoding(serde_json::Error),
    RowNotFound,
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

impl From<serde_json::Error> for DbError {
    fn from(value: serde_json::Error) -> Self {
        DbError::Decoding(value)
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::EnvVar(e) => write!(f, "Database environment variable error: {}", e),
            DbError::Connection(e) => write!(f, "Database connection error: {}", e),
            DbError::Migration(e) => write!(f, "Database migration failed: {}", e),
            DbError::Decoding(e) => write!(f, "Value decoding failed: {}", e),
            DbError::RowNotFound => write!(f, "Row not found"),
        }
    }
}

impl Error for DbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DbError::EnvVar(e) => Some(e),
            DbError::Connection(e) => Some(e),
            DbError::Migration(e) => Some(e),
            DbError::Decoding(e) => Some(e),
            DbError::RowNotFound => None,
        }
    }
}
