use axum_test::TestServer;

use crate::{AppState, create_app};

pub fn setup_test_server() -> TestServer {
    let state = AppState::default();
    let app = create_app(state);
    TestServer::builder()
        .mock_transport()
        .build(app)
        .expect("Cannot create server")
}
