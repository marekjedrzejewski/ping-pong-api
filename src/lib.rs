use std::{
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};

mod models;

use crate::models::{AppState, GameState, RallyState, Score, Side};

const BALL_AIR_TIME_SECONDS: u64 = 30;

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

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(get_state))
        .route("/ping", get(ping))
        .route("/pong", get(pong))
        .with_state(state)
}

async fn get_state(State(state): State<AppState>) -> (StatusCode, Json<AppState>) {
    (StatusCode::OK, Json(state))
}

fn try_hit(side: Side, state: AppState) -> String {
    let mut rally_state = state
        .rally_state
        .write()
        .expect("current_side write lock was poisoned");
    if side == rally_state.side {
        rally_state.side = (rally_state.side).flip();
        rally_state.hit_timeout =
            Some(SystemTime::now() + Duration::from_secs(BALL_AIR_TIME_SECONDS));
        (rally_state.side).to_string()
    } else {
        let mut game_state = state
            .game_state
            .write()
            .expect("overall_game_state lock was poisoned");
        game_state.score.lose_point(side);
        game_state.server = game_state.server.flip();
        rally_state.side = game_state.server.clone();
        rally_state.hit_timeout = None;

        "MISS".to_string()
    }
}

async fn ping(State(state): State<AppState>) -> String {
    try_hit(Side::Ping, state)
}
async fn pong(State(state): State<AppState>) -> String {
    try_hit(Side::Pong, state)
}
