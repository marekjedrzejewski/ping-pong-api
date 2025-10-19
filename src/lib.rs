use std::{
    fmt::{self, Formatter},
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::{Serialize, Serializer};

const BALL_AIR_TIME_SECONDS: u64 = 30;

#[derive(Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Ping,
    Pong,
}

impl Side {
    fn flip(&self) -> Self {
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
    fn lose_point(&mut self, side: Side) {
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

pub fn create_initial_state() -> AppState {
    AppState {
        rally_state: Arc::new(RwLock::new(RallyState {
            side: Side::Ping,
            hit_timeout: None,
        })),
        game_state: Arc::new(RwLock::new(GameState {
            server: Side::Ping,
            score: Score { ping: 0, pong: 0 },
        })),
    }
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(get_state))
        .route("/ping", get(ping))
        .route("/pong", get(pong))
        .with_state(state)
}

async fn get_state(State(state): State<AppState>) -> (StatusCode, Json<AppState>) {
    (StatusCode::OK, Json(state))
}

fn try_hit(side: Side, state: AppState) -> String {
    let mut rally_state = state
        .rally_state
        .write()
        .expect("current_side write lock was poisoned");
    if side == rally_state.side {
        rally_state.side = (rally_state.side).flip();
        rally_state.hit_timeout =
            Some(SystemTime::now() + Duration::from_secs(BALL_AIR_TIME_SECONDS));
        (rally_state.side).to_string()
    } else {
        let mut game_state = state
            .game_state
            .write()
            .expect("overall_game_state lock was poisoned");
        game_state.score.lose_point(side);
        game_state.server = game_state.server.flip();
        rally_state.side = game_state.server.clone();
        rally_state.hit_timeout = None;

        "MISS".to_string()
    }
}

async fn ping(State(state): State<AppState>) -> String {
    try_hit(Side::Ping, state)
}
async fn pong(State(state): State<AppState>) -> String {
    try_hit(Side::Pong, state)
}
