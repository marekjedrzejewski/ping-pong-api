use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use sqlx::PgPool;

use crate::database::TableUid;

use super::game::TableState;

#[derive(Default)]
pub struct AppState {
    // TODO: consider DashMap if contention becomes an issue
    pub game_tables: Arc<RwLock<HashMap<TableUid, TableState>>>,
    pub db_pool: Option<PgPool>,
}
