use axum_test::TestServer;
use ping_pong_api::{create_app, create_initial_state};
use serde_json::json;

#[tokio::test]
async fn play_some_ping_pong() {
    let state = create_initial_state();
    let app = create_app(state);
    let server = TestServer::builder()
        .mock_transport()
        .build(app)
        .expect("Cannot create server");

    let root_response = server.get("/").await;
    root_response.assert_status_ok();
    root_response.assert_json(&json!({
        "currentSide": "ping",
        "overallGameState": {
            "server": "ping",
            "score": {
                "ping": 0,
                "pong": 0
            }
        }
    }));

    // can't pong if it's ping turn
    let pong_response = server.get("/pong").await;
    pong_response.assert_text("MISS");
    server.get("/").await.assert_json_contains(&json!({
        "overallGameState": {
            "server": "pong",
            "score": {
                "ping": 1,
                "pong": 0
            }
        }
    }));

    // pong now serving, it's miss again
    let ping_response = server.get("/ping").await;
    ping_response.assert_text("MISS");
    server.get("/").await.assert_json_contains(&json!({
        "overallGameState": {
            "server": "ping",
            "score": {
                "ping": 1,
                "pong": 1
            }
        }
    }));

    // ok, at last ping can hit some ball
    let ping_response = server.get("/ping").await;
    ping_response.assert_text("pong");

    // same goes for pong
    let pong_response = server.get("/pong").await;
    pong_response.assert_text("ping");

    for _n in 0..50 {
        server.get("/ping").await.assert_text("pong");
        server.get("/").await.assert_json_contains(&json!({
            "currentSide": "pong"
        }));
        server.get("/pong").await.assert_text("ping");
        server.get("/").await.assert_json_contains(&json!({
            "currentSide": "ping"
        }));
    }

    // and another point goes to ping
    let pong_response = server.get("/pong").await;
    pong_response.assert_text("MISS");
    server.get("/").await.assert_json_contains(&json!({
        "overallGameState": {
            "score": {
                "ping": 2,
            }
        }
    }));
}
