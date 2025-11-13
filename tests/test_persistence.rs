use std::{
    io::{BufRead, BufReader},
    net::TcpListener,
    process::{Child, Command, Stdio},
};

use ping_pong_api::models::GameState;
use serde_json::Value;
use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};

#[tokio::test]
async fn test_persistence() {
    let container = postgres::Postgres::default().start().await.unwrap();
    let db_port = container.get_host_port_ipv4(5432).await.unwrap();
    let connection_string = &format!("postgres://postgres:postgres@127.0.0.1:{db_port}/postgres",);
    let api_port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let api_endpoint = format!("http://127.0.0.1:{api_port}");

    let mut server_process = start_server_and_wait_until_ready(connection_string, api_port);

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
    server_process.kill().unwrap();
    server_process.wait().unwrap();
    let mut server_process = start_server_and_wait_until_ready(connection_string, api_port);

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

    server_process.kill().unwrap();
    server_process.wait().unwrap();
}

fn start_server_and_wait_until_ready(db_url: &str, api_port: u16) -> Child {
    const SUCCESS_MESSAGE: &str = "Database initialized";
    let mut server_process = Command::new("cargo")
        .args(["run"])
        .env("DATABASE_URL", db_url)
        .env("RUST_LOG", "info")
        .env("SERVER_PORT", api_port.to_string())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to run application");

    // env_logger logs to stderr by default, so even when looking for 'info' it will be there
    let stderr = server_process
        .stderr
        .take()
        .expect("Child process did not have a handle to stdout");
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();

    // Read the output until we see the success message
    loop {
        match reader.read_line(&mut line) {
            Ok(0) => {
                panic!("Application terminated unexpectedly before logging",);
            }
            Ok(_) => {
                if line.contains(SUCCESS_MESSAGE) {
                    break;
                }
                line.clear(); // Clear the buffer for the next line
            }
            Err(_) => panic!(),
        }
    }

    server_process
}
