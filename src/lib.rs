use std::{
    process::exit,
    sync::{Arc, RwLock},
};

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use log::error;
use serde::Serialize;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};

pub mod clock;
pub mod database;
mod game_table;
pub mod models;

#[cfg(test)]
pub mod tests;

use crate::{database::get_game_tables, game_table::match_routes, models::application::AppState};

pub const BALL_AIR_TIME_SECONDS: u64 = 30;

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
    // TODO: Consider restricting CORS in the future
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/matches", get(open_matches))
        .nest("/matches/{id}", match_routes(state.clone()))
        .with_state(state)
        .layer(cors)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchList {
    open_matches: Vec<String>,
}

async fn open_matches(State(state): State<AppState>) -> (StatusCode, Json<MatchList>) {
    let game_tables = state
        .game_tables
        .read()
        .expect("game_tables read lock was poisoned");

    (
        StatusCode::OK,
        Json(MatchList {
            open_matches: game_tables.keys().map(|uid| uid.to_string()).collect(),
        }),
    )
}
