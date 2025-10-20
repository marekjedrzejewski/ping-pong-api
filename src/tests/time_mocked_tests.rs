//! Time mocked tests need to be within src/ to rely on #[cfg(test)] in clock.rs
//! Otherwise, clock.rs would always use std::time even in tests as whatever lies outside
//! of src/ is linked against the crate compiled without the test flag.

use mock_instant::global::MockClock;
use serde_json::json;
use std::time::Duration;

use super::utils::setup_test_server;

#[tokio::test]
async fn score_changes_on_timeout() {
    let server = setup_test_server();
    MockClock::set_system_time(std::time::Duration::ZERO);
    let ping_response = server.get("/ping").await;
    ping_response.assert_text("pong");

    tokio::time::pause();
    MockClock::advance_system_time(Duration::from_secs(11));
    tokio::time::advance(Duration::from_secs(11)).await;
    tokio::time::resume();
    tokio::time::sleep(Duration::from_millis(1)).await;
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

    tokio::time::pause();
    MockClock::advance_system_time(Duration::from_secs(32));
    tokio::time::advance(Duration::from_secs(32)).await;
    tokio::time::resume();
    tokio::time::sleep(Duration::from_millis(1)).await;

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
