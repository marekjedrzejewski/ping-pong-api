use sqlx::{Connection, PgConnection, query};

use crate::common::{get_random_port, setup_db, start_server_and_wait_for_the_message};

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
    let (connection_string, _db) = setup_db().await;
    let api_port = get_random_port();

    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to database");
    let _ = query("CREATE TABLE game_state(whatever TEXT)")
        .execute(&mut connection)
        .await
        .expect("Failed to create game_state table");

    let _ = start_server_and_wait_for_the_message(
        &connection_string,
        api_port,
        "\"game_state\" already exists",
    )
    .expect("Invalid migration message not found")
    .wait();
}
