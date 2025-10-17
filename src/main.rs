use ping_pong_api::{AppState, Side, create_app};
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() {
    let state = AppState {
        current_side: Arc::new(RwLock::new(Side::Ping)),
    };

    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
