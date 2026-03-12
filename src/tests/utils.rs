use std::sync::{Arc, RwLock};

use axum_test::TestServer;
use sqlx::PgPool;

use crate::{
    AppState, create_app_from_state,
    database::{TableDbSyncHandle, TableUid},
    models::game::{GameState, TableState},
};

pub const MATCH_ID: &str = "test";
pub const MATCH_ENDPOINT: &str = "/match/test";
pub const PING_ENDPOINT: &str = "/match/test/ping";
pub const PONG_ENDPOINT: &str = "/match/test/pong";

pub fn setup_test_server() -> TestServer {
    setup_test_server_with_matches(&[MATCH_ID])
}

pub fn setup_test_server_with_matches(ids: &[&str]) -> TestServer {
    let state = init_test_state_with_matches(ids);
    let app = create_app_from_state(state);
    TestServer::builder()
        .mock_transport()
        .build(app)
        .expect("Cannot create server")
}

pub mod mock_clock {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use jiff::Timestamp;

    // A thread-local key to store the mock clock for the current thread/test.
    // This prevents concurrent tests from interfering with each other's time.
    thread_local! {
        pub static TEST_CLOCK: MockClock = MockClock::new(Timestamp::now());
    }

    #[derive(Clone)]
    pub struct MockClock {
        time: Arc<Mutex<Timestamp>>,
    }

    impl MockClock {
        pub fn new(start_time: Timestamp) -> Self {
            Self {
                time: Arc::new(Mutex::new(start_time)),
            }
        }

        pub fn current(&self) -> Timestamp {
            *self.time.lock().unwrap()
        }

        pub fn advance(&self, duration: Duration) {
            let mut time = self.time.lock().unwrap();
            *time = time
                .checked_add(duration)
                .expect("Time overflowed or underflowed while advancing mock clock");
        }

        pub fn set_time(&self, new_time: Timestamp) {
            *self.time.lock().unwrap() = new_time;
        }
    }

    pub fn advance(duration: Duration) {
        TEST_CLOCK.with(|clock| clock.advance(duration));
    }

    pub fn set_time(time: Timestamp) {
        TEST_CLOCK.with(|clock| clock.set_time(time));
    }
}

fn init_test_state_with_matches(ids: &[&str]) -> AppState {
    let dummy_pool =
        PgPool::connect_lazy("postgres://localhost/unused").expect("Failed to connect to database");
    let tables = ids
        .iter()
        .enumerate()
        .map(|(i, &id)| {
            (
                TableUid::parse(id).unwrap(),
                TableState::new(
                    GameState::default(),
                    TableDbSyncHandle::new(i as i64, &dummy_pool),
                ),
            )
        })
        .collect();

    AppState {
        game_tables: Arc::new(RwLock::new(tables)),
        db_pool: dummy_pool,
    }
}
