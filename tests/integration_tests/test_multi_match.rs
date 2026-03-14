use serde_json::Value;

use crate::common::{
    get_random_port, send_sigterm_and_wait_for_exit, setup_db, start_server_and_wait_until_ready,
};

#[tokio::test]
async fn test_multi_match() {
    let (connection_string, _db) = setup_db().await;
    let api_port = get_random_port();
    let api_endpoint = format!("http://127.0.0.1:{api_port}");
    let get_match_list = || async {
        reqwest::get(&format!("{api_endpoint}/"))
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    };

    let server_process = start_server_and_wait_until_ready(&connection_string, api_port);

    // 1. Check initial state: openMatches is empty
    let match_list: Value = get_match_list().await;
    assert!(match_list["openMatches"].as_array().unwrap().is_empty());

    // 2. Create by access: GET /match/m1 and then verify / contains ["m1"]
    let _ = reqwest::get(&format!("{api_endpoint}/match/m1"))
        .await
        .unwrap();
    let match_list: Value = get_match_list().await;
    let open_matches = match_list["openMatches"].as_array().unwrap();
    assert_eq!(open_matches.len(), 1);
    assert_eq!(open_matches[0], "m1");

    // 3. Add another: GET /match/m2 and verify / contains both
    let _ = reqwest::get(&format!("{api_endpoint}/match/m2"))
        .await
        .unwrap();
    let match_list: Value = get_match_list().await;
    let open_matches = match_list["openMatches"].as_array().unwrap();
    assert_eq!(open_matches.len(), 2);
    let ids: Vec<_> = open_matches.iter().map(|v| v.as_str().unwrap()).collect();
    assert!(ids.contains(&"m1"));
    assert!(ids.contains(&"m2"));

    // 4. No duplicates: GET /match/m1 again
    let _ = reqwest::get(&format!("{api_endpoint}/match/m1"))
        .await
        .unwrap();
    let match_list: Value = get_match_list().await;
    assert_eq!(match_list["openMatches"].as_array().unwrap().len(), 2);

    // 5. Invalid ID check
    let resp = reqwest::get(&format!("{api_endpoint}/match/INVALID_ID"))
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
    let match_list: Value = get_match_list().await;
    // Match list untouched
    assert_eq!(match_list["openMatches"].as_array().unwrap().len(), 2);

    send_sigterm_and_wait_for_exit(server_process).unwrap();
}
