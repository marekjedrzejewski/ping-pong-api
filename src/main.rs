use std::process::exit;

use log::error;

use ping_pong_api::create_app;

mod database;
use database::init_db;

#[tokio::main]
async fn main() {
    env_logger::init();
    let pool = init_db().await;

    match pool {
        Ok(p) => {
            let app = create_app(p).await;
            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }
        Err(e) => {
            error!("Database initialization failed with error: {e}");
            exit(1);
        }
    }
}
