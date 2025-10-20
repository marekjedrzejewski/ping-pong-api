use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use tokio::time::interval;

mod clock;
mod models;

#[cfg(test)]
pub mod tests;

use crate::clock::SystemTime;

use crate::models::{AppState, GameState, RallyState, Score, Side};

const BALL_AIR_TIME_SECONDS: u64 = 30;
const GAME_LOOP_INTERVAL_MS: u64 = 1000;

pub fn create_initial_state() -> AppState {
    AppState {
        rally_state: Arc::new(RwLock::new(RallyState {
            side: Side::Ping,
            hit_timeout: None,
        })),
        game_state: Arc::new(RwLock::new(GameState {
            server: Side::Ping,
            score: Score { ping: 0, pong: 0 },
        })),
    }
}

async fn run_game_events(state: AppState) {
    let mut interval = interval(Duration::from_millis(GAME_LOOP_INTERVAL_MS));

    loop {
        interval.tick().await;

        let (side, hit_timeout) = {
            let rally_state = state
                .rally_state
                .read()
                .expect("rally_state read lock was poisoned");

            (rally_state.side.clone(), rally_state.hit_timeout)
        };

        if let Some(t) = hit_timeout
            && t < SystemTime::now()
        {
            lose_point(side, &state);
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

fn lose_point(side: Side, state: &AppState) {
    let mut game_state = state
        .game_state
        .write()
        .expect("game_state write lock was poisoned");
    let mut rally_state = state
        .rally_state
        .write()
        .expect("game_state write lock was poisoned");
    game_state.score.lose_point(side);
    game_state.server = game_state.server.flip();
    rally_state.side = game_state.server.clone();
    rally_state.hit_timeout = None;
}

fn try_hit(side: Side, state: AppState) -> String {
    let state_side = state
        .rally_state
        .read()
        .expect("rally_state read lock was poisoned")
        .side
        .clone();
    if side == state_side {
        let mut rally_state = state
            .rally_state
            .write()
            .expect("rally_state write lock was poisoned");

        rally_state.side = (rally_state.side).flip();
        rally_state.hit_timeout =
            Some(SystemTime::now() + Duration::from_secs(BALL_AIR_TIME_SECONDS));
        (rally_state.side).to_string()
    } else {
        lose_point(side, &state);

        "MISS".to_string()
    }
}

async fn ping(State(state): State<AppState>) -> String {
    try_hit(Side::Ping, state)
}
async fn pong(State(state): State<AppState>) -> String {
    try_hit(Side::Pong, state)
}
