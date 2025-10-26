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

#[tokio::test]
async fn longest_rally_updates_on_hit_count() {
    let server = setup_test_server();

    // --- Rally 1: 3 hits ---
    server.get("/ping").await;
    advance_time(Duration::from_secs(1)).await;
    server.get("/pong").await;
    advance_time(Duration::from_secs(1)).await;
    server.get("/ping").await;
    advance_time(Duration::from_secs(BALL_AIR_TIME_SECONDS + 1)).await;

    // Check state: longest rally should be 3
    let state: serde_json::Value = server.get("/").await.json();
    assert_eq!(state["gameState"]["longestRally"]["hitCount"], 3);

    // --- Rally 2: 2 hits (shorter) ---
    // Server is "pong"
    server.get("/pong").await;
    server.get("/ping").await;
    advance_time(Duration::from_secs(BALL_AIR_TIME_SECONDS + 1)).await;

    // Check state: longest rally should STILL be 3
    let state: serde_json::Value = server.get("/").await.json();
    assert_eq!(state["gameState"]["longestRally"]["hitCount"], 3);

    // --- Rally 3: 4 hits (longer) ---
    // Server is "ping"
    server.get("/ping").await;
    server.get("/pong").await;
    server.get("/ping").await;
    server.get("/pong").await;
    advance_time(Duration::from_secs(BALL_AIR_TIME_SECONDS + 1)).await;

    // Check state: longest rally should NOW be 4
    let state: serde_json::Value = server.get("/").await.json();
    assert_eq!(state["gameState"]["longestRally"]["hitCount"], 4);
}

#[tokio::test]
async fn longest_rally_updates_on_duration_tie_break() {
    let server = setup_test_server();

    // --- Rally 1: 2 hits, 5-second duration ---
    // First hit at T=0
    server.get("/ping").await;
    // Second hit at T=5
    advance_time(Duration::from_secs(5)).await;
    server.get("/pong").await;
    // Timeout at T=5 + 31 = T=36
    advance_time(Duration::from_secs(BALL_AIR_TIME_SECONDS + 1)).await;
    // Rally duration = T=36 - T=0 = 36 seconds

    let state: serde_json::Value = server.get("/").await.json();
    assert_eq!(state["gameState"]["longestRally"]["hitCount"], 2);
    // jiff duration serializes to ISO 8601 string. 5s + 31s = 36s
    assert_eq!(state["gameState"]["longestRally"]["duration"], "PT36S");
    let duration1 = state["gameState"]["longestRally"]["duration"].clone();

    // --- Rally 2: 2 hits, 3-second duration (shorter duration) ---
    // Server is "pong"
    // First hit at T=36
    server.get("/pong").await;
    // Second hit at T=36 + 3 = T=39
    advance_time(Duration::from_secs(3)).await;
    server.get("/ping").await;
    // Timeout at T=39 + 31 = T=70
    advance_time(Duration::from_secs(BALL_AIR_TIME_SECONDS + 1)).await;
    // Rally duration = T=70 - T=36 = 34 seconds

    let state: serde_json::Value = server.get("/").await.json();
    // Record should NOT update
    assert_eq!(state["gameState"]["longestRally"]["hitCount"], 2);
    assert_eq!(state["gameState"]["longestRally"]["duration"], duration1); // Still PT36S

    // --- Rally 3: 2 hits, 10-second duration (longer duration) ---
    // Server is "ping"
    // First hit at T=70
    server.get("/ping").await;
    // Second hit at T=70 + 10 = T=80
    advance_time(Duration::from_secs(10)).await;
    server.get("/pong").await;
    // Timeout at T=80 + 31 = T=111
    advance_time(Duration::from_secs(BALL_AIR_TIME_SECONDS + 1)).await;
    // Rally duration = T=111 - T=70 = 41 seconds

    let state: serde_json::Value = server.get("/").await.json();
    // Record SHOULD update
    assert_eq!(state["gameState"]["longestRally"]["hitCount"], 2);
    // 10s + 31s = 41s
    assert_eq!(state["gameState"]["longestRally"]["duration"], "PT41S");
    assert_ne!(state["gameState"]["longestRally"]["duration"], duration1);
}
