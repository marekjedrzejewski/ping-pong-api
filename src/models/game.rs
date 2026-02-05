use std::fmt::{self, Formatter};
use std::sync::{Arc, RwLock};

use jiff::{SignedDuration, Timestamp};
use log::error;
use serde::{Deserialize, Serialize};

use crate::clock;
use crate::database::TableDbSyncHandle;

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Default, Debug)]
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

#[derive(Clone, Serialize, Deserialize, Default, PartialEq, Debug)]
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
    #[serde(rename = "hitTimeoutTimestamp")]
    pub hit_timeout: Option<Timestamp>,
    #[serde(rename = "serveTimestamp")]
    pub first_hit_at: Option<Timestamp>,
    pub hit_count: usize,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RallyStatistics {
    hit_count: usize,
    duration: SignedDuration,
}

#[derive(Clone, Serialize, Deserialize, Default, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub server: Side,
    pub score: Score,
    pub longest_rally: Option<RallyStatistics>,
}

/// Updates longest rally - hit count based.
///
/// Duration only saved as a bonus - you can have more hits with shorter duration
/// and it will overwrite previous, longer one.
fn update_statistics(
    game_state: &mut std::sync::RwLockWriteGuard<'_, GameState>,
    rally_state: &std::sync::RwLockWriteGuard<'_, RallyState>,
) {
    if let Some(start) = rally_state.first_hit_at {
        let current_rally_time = clock::now().duration_since(start);
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
                } else if longest_rally.hit_count == rally_state.hit_count
                    && longest_rally.duration < current_rally_time
                {
                    longest_rally.duration = current_rally_time
                }
            }
        }
    }
}

#[derive(Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TableState {
    pub rally_state: Arc<RwLock<RallyState>>,
    pub game_state: Arc<RwLock<GameState>>,
    #[serde(skip)]
    db_handle: Option<TableDbSyncHandle>,
}

impl TableState {
    pub fn new(game_state: GameState) -> Self {
        Self {
            game_state: Arc::new(RwLock::new(game_state)),
            rally_state: Arc::default(),
            db_handle: None,
        }
    }

    pub fn with_db_handle(mut self, db_handle: TableDbSyncHandle) -> Self {
        self.db_handle = Some(db_handle);
        self
    }

    pub async fn lose_point(&self, side: Side) {
        let game_state = {
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

            update_statistics(&mut game_state, &rally_state);

            rally_state.hit_timeout = None;
            rally_state.first_hit_at = None;
            rally_state.hit_count = 0;

            game_state.clone()
        };

        if let Some(db_handle) = &self.db_handle
            && let Err(e) = db_handle.upsert_game_state(game_state).await
        {
            error!("Error while updating game state in database: {e}")
        }
    }
}
