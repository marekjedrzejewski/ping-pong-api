//! Clock abstraction allowing mocking time in tests.

#[cfg(test)]
pub use mock_instant::global::{SystemTime, UNIX_EPOCH};

#[cfg(not(test))]
pub use std::time::{SystemTime, UNIX_EPOCH};
