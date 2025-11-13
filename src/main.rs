use std::env;
use std::process::exit;

use log::error;

use ping_pong_api::create_app;

use ping_pong_api::database::init_db;

#[tokio::main]
async fn main() {
    env_logger::init();
    let server_port: u16 = env::var("SERVER_PORT")
        .unwrap_or_else(|_| 3000.to_string())
        .parse()
        .expect("SERVER_PORT must be a valid port number");
    let pool = init_db().await;

    match pool {
        Ok(p) => {
            let app = create_app(p).await;
            let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{server_port}"))
                .await
                .unwrap();
            axum::serve(listener, app).await.unwrap();
        }
        Err(e) => {
            error!("Database initialization failed with error: {e}");
            exit(1);
        }
    }
}
