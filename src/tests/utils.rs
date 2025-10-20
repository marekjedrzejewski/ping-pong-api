use axum_test::TestServer;

use crate::{create_app, create_initial_state};

pub fn setup_test_server() -> TestServer {
    let state = create_initial_state();
    let app = create_app(state);
    TestServer::builder()
        .mock_transport()
        .build(app)
        .expect("Cannot create server")
}
