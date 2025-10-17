use std::sync::{Arc, RwLock};

use axum::{Router, extract::State, routing::get};

#[derive(Clone)]
pub enum Side {
    Ping,
    Pong,
}

#[derive(Clone)]
pub struct AppState {
    pub current_side: Arc<RwLock<Side>>,
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(spectate))
        .route("/ping", get(ping))
        .route("/pong", get(pong))
        .with_state(state)
}

async fn spectate(State(state): State<AppState>) -> String {
    let data = state.current_side.read().expect("mutex was poisoned");
    match *data {
        Side::Ping => "Ping".to_string(),
        Side::Pong => "Pong".to_string(),
    }
}
async fn ping(State(state): State<AppState>) -> String {
    let mut data = state.current_side.write().expect("mutex was poisoned");
    match *data {
        Side::Ping => {
            *data = Side::Pong;
            "Pong".to_string()
        }
        Side::Pong => "MISS".to_string(),
    }
}
async fn pong(State(state): State<AppState>) -> String {
    let mut data = state.current_side.write().expect("mutex was poisoned");
    match *data {
        Side::Pong => {
            *data = Side::Ping;
            "Ping".to_string()
        }
        Side::Ping => "MISS".to_string(),
    }
}
