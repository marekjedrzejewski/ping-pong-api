use std::{
    fmt::{self, Formatter},
    sync::{Arc, RwLock},
    time::Duration,
};

use serde::{Serialize, Serializer};

use crate::clock::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Serialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    #[default]
    Ping,
    Pong,
}

impl Side {
    pub fn flip(&self) -> Self {
        match self {
            Side::Ping => Side::Pong,
            Side::Pong => Side::Ping,
        }
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Side::Ping => write!(f, "ping"),
            Side::Pong => write!(f, "pong"),
        }
    }
}

#[derive(Clone, Serialize, Default)]
pub struct Score {
    pub ping: usize,
    pub pong: usize,
}

impl Score {
    pub fn lose_point(&mut self, side: Side) {
        match side {
            Side::Ping => self.pong += 1,
            Side::Pong => self.ping += 1,
        }
    }
}

#[derive(Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RallyState {
    pub side: Side,
    #[serde(
        serialize_with = "unix_millis_serializer",
        rename = "hitTimeoutTimestamp"
    )]
    pub hit_timeout: Option<SystemTime>,
    #[serde(serialize_with = "unix_millis_serializer", rename = "serveTimestamp")]
    pub first_hit_at: Option<SystemTime>,
    pub hit_count: usize,
}

#[derive(Clone, Serialize)]
pub struct RallyStatistics {
    hit_count: usize,
    duration: Duration,
}

#[derive(Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub server: Side,
    pub score: Score,
    pub longest_rally: Option<RallyStatistics>,
}

#[derive(Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub rally_state: Arc<RwLock<RallyState>>,
    pub game_state: Arc<RwLock<GameState>>,
}

impl AppState {
    pub fn lose_point(self: &AppState, side: Side) {
        let mut game_state = self
            .game_state
            .write()
            .expect("game_state write lock was poisoned");
        let mut rally_state = self
            .rally_state
            .write()
            .expect("rally_state write lock was poisoned");
        game_state.score.lose_point(side);
        game_state.server = game_state.server.flip();
        rally_state.side = game_state.server;

        update_statistics(game_state, &rally_state);

        rally_state.hit_timeout = None;
        rally_state.first_hit_at = None;
        rally_state.hit_count = 0;
    }
}

/// Updates longest rally - hit count based.
///
/// Duration only saved as a bonus - you can have more hits with shorter duration
/// and it will overwrite previous, longer one.
fn update_statistics(
    mut game_state: std::sync::RwLockWriteGuard<'_, GameState>,
    rally_state: &std::sync::RwLockWriteGuard<'_, RallyState>,
) {
    if let Some(start) = rally_state.first_hit_at
        && let Some(current_rally_time) = SystemTime::now()
            .duration_since(start)
            .inspect_err(|error| eprintln!("Error calculating rally duration: {}", error))
            .ok()
    {
        match &mut game_state.longest_rally {
            None => {
                game_state.longest_rally = Some(RallyStatistics {
                    hit_count: rally_state.hit_count,
                    duration: current_rally_time,
                })
            }
            Some(longest_rally) => {
                if longest_rally.hit_count < rally_state.hit_count {
                    longest_rally.duration = current_rally_time;
                    longest_rally.hit_count = rally_state.hit_count
                }
            }
        }
    }
}

fn unix_millis_serializer<S>(time: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match time {
        Some(t) => serializer.serialize_some(&t.duration_since(UNIX_EPOCH).unwrap().as_millis()),
        None => serializer.serialize_none(),
    }
}
