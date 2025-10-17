use std::{
    fmt::{self, Formatter},
    sync::{Arc, RwLock},
};

use axum::{Router, extract::State, routing::get};

#[derive(Clone)]
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
            Side::Ping => write!(f, "Ping"),
            Side::Pong => write!(f, "Pong"),
        }
    }
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
    let data = state.current_side.read().expect("read lock was poisoned");
    (*data).to_string()
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
