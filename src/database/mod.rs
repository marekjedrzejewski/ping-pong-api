use crate::models::{
    application::GameTables,
    game::{GameState, TableState},
};
use log::info;
use sqlx::{PgPool, migrate::MigrateError, postgres::PgPoolOptions};
use std::{
    env::{self, VarError},
    error::Error,
    fmt,
};

/// TableUid has constraints on its format enforced by the database.
/// Specifying different type to be clear that this is not just any string
/// and not duplicate checks.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct TableUid(String);

impl TableUid {
    pub fn new(uid: String) -> Self {
        TableUid(uid)
    }
}

#[derive(Clone)]
pub struct TableDbSyncHandle {
    game_state_id: i64,
    pool: PgPool,
}
impl TableDbSyncHandle {
    pub fn new(game_state_id: i64, pool: &PgPool) -> Self {
        TableDbSyncHandle {
            game_state_id,
            pool: pool.clone(),
        }
    }

    pub async fn update_game_state(&self, game_state: GameState) -> Result<(), DbError> {
        update_game_state(&self.pool, self.game_state_id, game_state).await
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

/// gets and initializes game tables with sync handles
pub async fn get_game_tables(pool: &PgPool) -> Result<GameTables, DbError> {
    sqlx::query!(
        "SELECT uid, game_state_id, data_dump as game_state
     FROM match JOIN game_state ON match.game_state_id = game_state.id"
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        // TODO: One bad record destroys everything. Think if we want that or filter
        let table_state = TableState::new(
            serde_json::from_value(row.game_state)?,
            TableDbSyncHandle::new(row.game_state_id, pool),
        );
        Ok((TableUid(row.uid), table_state))
    })
    .collect()
}

pub async fn update_game_state(
    pool: &PgPool,
    table_id: i64,
    game_state: GameState,
) -> Result<(), DbError> {
    let data_dump =
        serde_json::to_value(game_state).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

    let mut tx = pool.begin().await?;

    let result = sqlx::query!(
        "UPDATE game_state SET data_dump = $2 WHERE id = $1",
        table_id,
        data_dump
    )
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::RowNotFound);
    }

    tx.commit().await?;

    Ok(())
}

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
