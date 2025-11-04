use ping_pong_api::{create_app, models::AppState};
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL env variable is required.");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Can't connect to database");

    let row: (String,) = sqlx::query_as("SELECT 'it works'")
        .fetch_one(&pool)
        .await
        .expect("YOU BROKE THE QUERY");
    dbg!(row);

    let state = AppState::default();

    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
