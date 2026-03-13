use std::{
    process::exit,
    sync::{Arc, RwLock},
    time::Duration,
};

use axum::Router;
use log::error;
use sqlx::PgPool;
use tokio::time::interval;
use tower_http::cors::{Any, CorsLayer};

pub mod clock;
pub mod database;
mod game_table;
pub mod models;

#[cfg(test)]
pub mod tests;

use crate::{
    database::get_game_tables,
    game_table::match_routes,
    models::{application::AppState, game::TableState},
};

pub const BALL_AIR_TIME_SECONDS: u64 = 30;
const GAME_LOOP_INTERVAL_MS: u64 = 1000;

// TODO: this was good enough for starting, but not sure how well it will scale
async fn run_game_events(state: TableState) {
    let mut interval = interval(Duration::from_millis(GAME_LOOP_INTERVAL_MS));

    loop {
        interval.tick().await;

        let (side, hit_timeout) = {
            let rally_state = state
                .rally_state
                .read()
                .expect("rally_state read lock was poisoned");

            (rally_state.side, rally_state.hit_timeout)
        };

        if let Some(t) = hit_timeout
            && t < clock::now()
        {
            state.lose_point(side).await;
        }
    }
}

async fn init_state(pool: &PgPool) -> Result<AppState, database::DbError> {
    let game_tables = get_game_tables(pool).await?;

    Ok(AppState {
        game_tables: Arc::new(RwLock::new(game_tables)),
        db_pool: pool.clone(),
    })
}

pub async fn create_app(pool: PgPool) -> Router {
    match init_state(&pool).await {
        Ok(state) => create_app_from_state(state),
        Err(e) => {
            error!("Failed to initialize app state from database: {e}");
            exit(1)
        }
    }
}

pub fn create_app_from_state(state: AppState) -> Router {
    for (_, table) in state
        .game_tables
        .read()
        .expect("game_tables read lock was poisoned")
        .iter()
    {
        tokio::spawn(run_game_events(table.clone()));
    }

    // TODO: Consider restricting CORS in the future
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/match/{id}", match_routes(state.clone()))
        .with_state(state)
        .layer(cors)
}
