use std::{
    fmt::{self, Formatter},
    sync::{Arc, RwLock},
};

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;

#[derive(Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Ping,
    Pong,
}

impl Side {
    fn flip(&self) -> Self {
        match self {
            Side::Ping => Side::Pong,
            Side::Pong => Side::Ping,
        }
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Side::Ping => write!(f, "ping"),
            Side::Pong => write!(f, "pong"),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct Score {
    pub ping: usize,
    pub pong: usize,
}

impl Score {
    fn lose_point(&mut self, side: Side) {
        match side {
            Side::Ping => self.pong += 1,
            Side::Pong => self.ping += 1,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct GameState {
    pub server: Side,
    pub score: Score,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub current_side: Arc<RwLock<Side>>,
    pub overall_game_state: Arc<RwLock<GameState>>,
}

pub fn create_initial_state() -> AppState {
    AppState {
        current_side: Arc::new(RwLock::new(Side::Ping)),
        overall_game_state: Arc::new(RwLock::new(GameState {
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
    let mut current_side = state
        .current_side
        .write()
        .expect("current_side write lock was poisoned");
    if side == *current_side {
        *current_side = (*current_side).flip();

        (*current_side).to_string()
    } else {
        let mut game_state = state
            .overall_game_state
            .write()
            .expect("overall_game_state lock was poisoned");
        game_state.score.lose_point(side);
        game_state.server = game_state.server.flip();
        *current_side = game_state.server.clone();

        "MISS".to_string()
    }
}

async fn ping(State(state): State<AppState>) -> String {
    try_hit(Side::Ping, state)
}
async fn pong(State(state): State<AppState>) -> String {
    try_hit(Side::Pong, state)
}
