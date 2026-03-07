mod db_error;
mod table_uid;
use crate::models::{
    application::GameTables,
    game::{GameState, TableState},
};
pub use db_error::DbError;
use log::info;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
pub use table_uid::TableUid;

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
        Ok((
            // If database has invalid UIDs we want to fail fast and fix
            TableUid::parse(&row.uid)
                .unwrap_or_else(|_| panic!("Invalid Table UID in database: {}", &row.uid)),
            table_state,
        ))
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
