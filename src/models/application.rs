use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use sqlx::PgPool;

use super::game::TableState;

#[derive(Default)]
pub struct AppState {
    // TODO: consider DashMap if contention becomes an issue
    pub game_tables: Arc<RwLock<HashMap<i64, TableState>>>,
    pub db_pool: Option<PgPool>,
}
