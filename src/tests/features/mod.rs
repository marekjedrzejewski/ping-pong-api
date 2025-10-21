//! Tests in this module act as integration tests but require time mocking from `#[cfg(test)]` in `clock.rs`, so they must remain in `src/`.

//! Otherwise, clock.rs would always use std::time even in tests, because files outside
//! of src/ (such as those in the tests/ directory) are compiled as separate crates without #[cfg(test)],
//! so conditional compilation for tests does not apply to them.

mod basic_game;
mod time_dependent;
