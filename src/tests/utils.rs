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
