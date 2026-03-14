use serde_json::json;

use crate::tests::utils::{setup_test_server, setup_test_server_with_matches};

pub const MATCH_A: &str = "/match/a";
pub const MATCH_B: &str = "/match/b";
pub const MATCH_IDS: &[&str] = &["a", "b"];

#[tokio::test]
async fn matches_are_isolated() {
    let server = setup_test_server_with_matches(MATCH_IDS);

    // pong misses on ping's serve in match A — ping scores, pong now serves
    server.get(&format!("{MATCH_A}/pong")).await;

    // match A score changed, rally reset with pong serving
    server.get(MATCH_A).await.assert_json_contains(&json!({
        "rallyState": { "side": "pong", "hitCount": 0 },
        "gameState": { "score": { "ping": 1, "pong": 0 } }
    }));

    // match B is completely untouched
    server.get(MATCH_B).await.assert_json(&json!({
        "rallyState": {
            "side": "ping",
            "hitTimeoutTimestamp": null,
            "serveTimestamp": null,
            "hitCount": 0
        },
        "gameState": {
            "score": { "ping": 0, "pong": 0 },
            "longestRally": null,
            "server": "ping"
        }
    }));
}

#[tokio::test]
async fn invalid_match_id_returns_bad_request() {
    let server = setup_test_server();

    let response = server.get("/match/INVALID").await;
    response.assert_status_bad_request();
    response.assert_text_contains(
        "at most 6 characters long and contain only lowercase letters and digits",
    );
}
