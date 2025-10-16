use axum::Router;
use axum::extract::State;
use axum::routing::get;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
enum Side {
    Ping,
    Pong,
}
#[derive(Clone)]
struct AppState {
    current_side: Arc<RwLock<Side>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        current_side: Arc::new(RwLock::new(Side::Ping)),
    };

    let app = Router::new()
        .route("/", get(spectate))
        .route("/ping", get(ping))
        .route("/pong", get(pong))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
