use crate::models::game::GameState;
use log::{error, info};
use sqlx::{PgPool, migrate::MigrateError, postgres::PgPoolOptions};
use std::{
    env::{self, VarError},
    error::Error,
    fmt,
};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct TableDbSyncHandle {
    table_id: i64,
    tx: mpsc::Sender<(i64, GameState)>,
}
impl TableDbSyncHandle {
    pub fn new(table_id: i64, tx: mpsc::Sender<(i64, GameState)>) -> Self {
        TableDbSyncHandle { table_id, tx }
    }

    pub async fn upsert_game_state(
        &self,
        game_state: GameState,
    ) -> Result<(), mpsc::error::SendError<(i64, GameState)>> {
        self.tx.send((self.table_id, game_state)).await
    }
}

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

pub async fn get_table_state(table_id: i64, pool: &PgPool) -> Result<Option<GameState>, DbError> {
    let game_state_row = sqlx::query!("SELECT data_dump FROM game_state WHERE id = $1", table_id)
        .fetch_optional(pool)
        .await?;

    match game_state_row {
        Some(row) => Ok(Some(serde_json::from_value(row.data_dump)?)),
        None => Ok(None),
    }
}

// TODO: This only handles single table now, extend it.
pub async fn upsert_game_state(pool: PgPool, game_state: GameState) -> Result<(), DbError> {
    let data_dump =
        serde_json::to_value(game_state).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

    let mut tx = pool.begin().await?;

    // We currently only care about single game state, hence clearing everything
    sqlx::query!("DELETE FROM game_state")
        .execute(&mut *tx)
        .await?;

    sqlx::query!("INSERT INTO game_state (data_dump) VALUES ($1)", data_dump)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn db_worker(pool: PgPool, mut rx: mpsc::Receiver<(i64, GameState)>) {
    while let Some((table_id, game_state)) = rx.recv().await {
        if let Err(e) = upsert_game_state(pool.clone(), game_state).await {
            error!(
                "Error while upserting game state for table {}: {e}",
                table_id
            );
        }
    }
}

#[derive(Debug)]
pub enum DbError {
    EnvVar(VarError),
    Connection(sqlx::Error),
    Migration(MigrateError),
    Decoding(serde_json::Error),
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
        }
    }
}
