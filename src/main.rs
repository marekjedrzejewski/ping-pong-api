use ping_pong_api::{create_app, models::AppState};

#[tokio::main]
async fn main() {
    let state = AppState::default();

    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
