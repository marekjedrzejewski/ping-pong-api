use sqlx::{Connection, PgConnection, query};
use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};

use crate::common::{get_random_port, start_server_and_wait_for_the_message};

#[tokio::test]
async fn test_invalid_database_url() {
    let api_port = get_random_port();
    let _ = start_server_and_wait_for_the_message(
        "",
        api_port,
        "error with configuration: relative URL without a base",
    )
    .expect("Invalid database url message not found")
    .wait();
}

#[tokio::test]
async fn test_db_migration_fail() {
    let container = postgres::Postgres::default().start().await.unwrap();
    let db_port = container.get_host_port_ipv4(5432).await.unwrap();
    let connection_string = &format!("postgres://postgres:postgres@127.0.0.1:{db_port}/postgres",);
    let api_port = get_random_port();

    let mut connection = PgConnection::connect(connection_string)
        .await
        .expect("Failed to connect to database");
    let _ = query("CREATE TABLE game_state(whatever TEXT)")
        .execute(&mut connection)
        .await
        .expect("Failed to create game_state table");

    let _ = start_server_and_wait_for_the_message(
        connection_string,
        api_port,
        "\"game_state\" already exists",
    )
    .expect("Invalid migration message not found")
    .wait();
}
