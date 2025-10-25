use std::time::Duration;

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use tokio::time::interval;

mod clock;
pub mod models;

#[cfg(test)]
pub mod tests;

use crate::clock::SystemTime;

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
            && t < SystemTime::now()
        {
            state.lose_point(side);
        }
    }
}

pub fn create_app(state: AppState) -> Router {
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
        rally_state.hit_timeout =
            Some(SystemTime::now() + Duration::from_secs(BALL_AIR_TIME_SECONDS));
        rally_state.first_hit_at.get_or_insert_with(SystemTime::now);

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
