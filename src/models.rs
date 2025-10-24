use std::{
    fmt::{self, Formatter},
    sync::{Arc, RwLock},
    time::Duration,
};

use serde::{Serialize, Serializer};

use crate::clock::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Side {
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

#[derive(Clone, Serialize)]
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

#[derive(Clone, Serialize)]
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
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub server: Side,
    pub score: Score,
    pub longest_rally: Option<Duration>,
}

#[derive(Clone, Serialize)]
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
        rally_state.side = game_state.server.clone();

        if let Some(start) = rally_state.first_hit_at {
            let current_rally_time_duration_result = SystemTime::now().duration_since(start);

            match current_rally_time_duration_result {
                Ok(current_rally_time) => {
                    let longest_rally = game_state.longest_rally.unwrap_or(Duration::ZERO);
                    if current_rally_time > longest_rally {
                        game_state.longest_rally = Some(current_rally_time);
                    }
                }
                Err(error) => {
                    eprintln!("Error calculating rally duration: {}", error);
                }
            }
        }

        rally_state.hit_timeout = None;
        rally_state.first_hit_at = None;
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
