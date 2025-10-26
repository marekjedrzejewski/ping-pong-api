use serde_json::json;
use std::time::Duration;

use crate::BALL_AIR_TIME_SECONDS;
use crate::tests::utils::mock_clock;
use crate::tests::utils::setup_test_server;

async fn advance_time(duration: Duration) {
    tokio::time::pause();
    mock_clock::advance(duration);
    tokio::time::advance(duration).await;
    tokio::time::resume();
    // give some time for the game loop to process the time advancement
    tokio::time::sleep(Duration::from_millis(1)).await;
}

#[tokio::test]
async fn score_changes_on_timeout() {
    let server = setup_test_server();
    let ping_response = server.get("/ping").await;
    ping_response.assert_text("pong");

    advance_time(Duration::from_secs(BALL_AIR_TIME_SECONDS / 2)).await;
    let root_response: serde_json::Value = server.get("/").await.json();
    let initial_timeout_timestamp =
        root_response.as_object().unwrap()["rallyState"]["hitTimeoutTimestamp"].clone();

    // hit again to refresh timeout
    server.get("/pong").await.assert_text("ping");
    let root_response: serde_json::Value = server.get("/").await.json();

    // the timeout timestamp should have changed
    assert_ne!(
        initial_timeout_timestamp,
        root_response.as_object().unwrap()["rallyState"]["hitTimeoutTimestamp"]
    );

    // now advance time beyond the timeout to cause a miss
    advance_time(Duration::from_secs(BALL_AIR_TIME_SECONDS + 1)).await;

    server.get("/").await.assert_json_contains(&json!({
        "gameState": {
            "score": {
                "pong": 1,
                "ping": 0,
            }
        },
        "rallyState": {
            "hitTimeoutTimestamp": null
        }
    }));
}
