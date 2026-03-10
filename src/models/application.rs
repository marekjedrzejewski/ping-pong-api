use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use sqlx::PgPool;

use crate::database::TableUid;

use super::game::TableState;

// TODO: consider DashMap if contention becomes an issue
pub type GameTables = HashMap<TableUid, TableState>;

#[derive(Clone)]
pub struct AppState {
    pub game_tables: Arc<RwLock<GameTables>>,
    pub db_pool: PgPool,
}
