use std::{
    process::exit,
    sync::{Arc, RwLock},
    time::Duration,
};

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use log::error;
use sqlx::PgPool;
use tokio::time::interval;

pub mod clock;
pub mod database;
pub mod models;

#[cfg(test)]
pub mod tests;

use crate::models::{AppState, Side};

pub const BALL_AIR_TIME_SECONDS: u64 = 30;
const GAME_LOOP_INTERVAL_MS: u64 = 1000;

async fn run_game_events(state: AppState) {
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
            state.lose_point(side);
        }
    }
}

async fn init_state(pool: &PgPool) -> Result<AppState, database::DbError> {
    let game_state = database::get_game_state(pool).await?;

    let game_state = game_state.unwrap_or_default();

    Ok(AppState {
        game_state: Arc::new(RwLock::new(game_state)),
        db_pool: Some((*pool).clone()),
        ..Default::default()
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
    tokio::spawn(run_game_events(state.clone()));
    Router::new()
        .route("/", get(get_state))
        .route("/ping", get(ping))
        .route("/pong", get(pong))
        .with_state(state)
}

async fn get_state(State(state): State<AppState>) -> (StatusCode, Json<AppState>) {
    (StatusCode::OK, Json(state))
}

fn try_hit(side: Side, state: AppState) -> bool {
    let state_side = state
        .rally_state
        .read()
        .expect("rally_state read lock was poisoned")
        .side;

    if side == state_side {
        let mut rally_state = state
            .rally_state
            .write()
            .expect("rally_state write lock was poisoned");

        rally_state.side = (rally_state.side).flip();
        rally_state.hit_count += 1;
        rally_state.hit_timeout = Some(clock::now() + Duration::from_secs(BALL_AIR_TIME_SECONDS));
        rally_state.first_hit_at.get_or_insert_with(clock::now);

        true
    } else {
        state.lose_point(side);

        false
    }
}

fn get_hit_response(side: Side, state: AppState) -> (StatusCode, String) {
    match try_hit(side, state) {
        true => (StatusCode::OK, side.flip().to_string()),
        false => (StatusCode::CONFLICT, "MISS".to_string()),
    }
}

async fn ping(State(state): State<AppState>) -> (StatusCode, String) {
    get_hit_response(Side::Ping, state)
}
async fn pong(State(state): State<AppState>) -> (StatusCode, String) {
    get_hit_response(Side::Pong, state)
}
