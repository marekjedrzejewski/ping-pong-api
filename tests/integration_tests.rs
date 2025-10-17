use std::sync::{Arc, RwLock};

use axum_test::TestServer;
use ping_pong_api::{AppState, Side, create_app};

#[tokio::test]
async fn play_some_ping_pong() {
    let state = AppState {
        current_side: Arc::new(RwLock::new(Side::Ping)),
    };
    let app = create_app(state);
    let server = TestServer::builder()
        .mock_transport()
        .build(app)
        .expect("Cannot create server");

    let root_response = server.get("/").await;
    root_response.assert_status_ok();
    root_response.assert_text("Ping");

    // can't pong if it's ping turn
    let pong_response = server.get("/pong").await;
    pong_response.assert_text("MISS");

    let ping_response = server.get("/ping").await;
    ping_response.assert_text("Pong");

    // NOT SO FAST, WAIT FOR YOUR TURN!
    let ping_response = server.get("/ping").await;
    ping_response.assert_text("MISS");

    let pong_response = server.get("/pong").await;
    pong_response.assert_text("Ping");

    for _n in 0..50 {
        server.get("/ping").await.assert_text("Pong");
        server.get("/").await.assert_text("Pong");
        server.get("/pong").await.assert_text("Ping");
        server.get("/").await.assert_text("Ping");
    }
}
