use std::{
    fmt::{self, Formatter},
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Serialize, Serializer};

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
}

#[derive(Clone, Serialize)]
pub struct GameState {
    pub server: Side,
    pub score: Score,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub rally_state: Arc<RwLock<RallyState>>,
    pub game_state: Arc<RwLock<GameState>>,
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
