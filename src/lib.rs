use std::{
    fmt::{self, Formatter},
    sync::{Arc, RwLock},
};

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;

#[derive(Clone, Serialize)]
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
async fn ping(State(state): State<AppState>) -> String {
    let mut data = state.current_side.write().expect("write lock was poisoned");
    match *data {
        Side::Ping => {
            *data = (*data).flip();
            (*data).to_string()
        }
        Side::Pong => "MISS".to_string(),
    }
}
async fn pong(State(state): State<AppState>) -> String {
    let mut data = state.current_side.write().expect("write lock was poisoned");
    match *data {
        Side::Pong => {
            *data = (*data).flip();
            (*data).to_string()
        }
        Side::Ping => "MISS".to_string(),
    }
}
