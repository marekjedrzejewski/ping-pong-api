use ping_pong_api::models::GameState;
use serde_json::Value;
use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};

use crate::common::{
    get_random_port, send_sigterm_and_wait_for_exit, start_server_and_wait_until_ready,
};

#[tokio::test]
async fn test_persistence() {
    let container = postgres::Postgres::default().start().await.unwrap();
    let db_port = container.get_host_port_ipv4(5432).await.unwrap();
    let connection_string = &format!("postgres://postgres:postgres@127.0.0.1:{db_port}/postgres",);
    let api_port = get_random_port();
    let api_endpoint = format!("http://127.0.0.1:{api_port}");

    let server_process = start_server_and_wait_until_ready(connection_string, api_port);

    // Server should start with clean db
    let app_state: Value = reqwest::get(&api_endpoint)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let initial_game_state: GameState =
        serde_json::from_value(app_state["gameState"].clone()).unwrap();
    assert_eq!(initial_game_state, GameState::default());

    // Play a set that ends with pong missing
    for _ in 0..10 {
        let _ = reqwest::get(format!("{}/ping", &api_endpoint)).await;
        let _ = reqwest::get(format!("{}/pong", &api_endpoint)).await;
    }
    let _ = reqwest::get(format!("{}/pong", &api_endpoint)).await;

    // Get game state for comparison after restarting server
    let app_state: Value = reqwest::get(&api_endpoint)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let game_state_before_restart: GameState =
        serde_json::from_value(app_state["gameState"].clone()).unwrap();

    // Restart server
    let _ = send_sigterm_and_wait_for_exit(server_process);
    let server_process = start_server_and_wait_until_ready(connection_string, api_port);

    // ...and compare values with ones from the last run
    let app_state: Value = reqwest::get(&api_endpoint)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let game_state_after_restart: GameState =
        serde_json::from_value(app_state["gameState"].clone()).unwrap();
    assert_ne!(initial_game_state, game_state_before_restart);
    assert_eq!(game_state_before_restart, game_state_after_restart);

    let _ = send_sigterm_and_wait_for_exit(server_process);
}
