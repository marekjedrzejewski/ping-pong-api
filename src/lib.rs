use std::{
    process::exit,
    sync::{Arc, RwLock},
    time::Duration,
};

use axum::{
    Extension, Json, Router,
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use log::error;
use sqlx::PgPool;
use tokio::time::interval;
use tower_http::cors::{Any, CorsLayer};

pub mod clock;
pub mod database;
pub mod models;

#[cfg(test)]
pub mod tests;

use crate::{
    database::{TableUid, get_game_tables},
    models::{
        application::AppState,
        game::{Side, TableState},
    },
};

pub const BALL_AIR_TIME_SECONDS: u64 = 30;
const GAME_LOOP_INTERVAL_MS: u64 = 1000;

// TODO: this was good enough for starting, but not sure how well it will scale
async fn run_game_events(state: TableState) {
    let mut interval = interval(Duration::from_millis(GAME_LOOP_INTERVAL_MS));

    loop {
        interval.tick().await;

        let (side, hit_timeout) = {
            let rally_state = state
                .rally_state
                .read()
                .expect("rally_state read lock was poisoned");

            (rally_state.side, rally_state.hit_timeout)
        };

        if let Some(t) = hit_timeout
            && t < clock::now()
        {
            state.lose_point(side).await;
        }
    }
}

async fn init_state(pool: &PgPool) -> Result<AppState, database::DbError> {
    let game_tables = get_game_tables(pool).await?;

    Ok(AppState {
        game_tables: Arc::new(RwLock::new(game_tables)),
        db_pool: pool.clone(),
    })
}

pub async fn create_app(pool: PgPool) -> Router {
    match init_state(&pool).await {
        Ok(state) => create_app_from_state(state),
        Err(e) => {
            error!("Failed to initialize app state from database: {e}");
            exit(1)
        }
    }
}

pub fn create_app_from_state(state: AppState) -> Router {
    for (_, table) in state
        .game_tables
        .read()
        .expect("game_tables read lock was poisoned")
        .iter()
    {
        tokio::spawn(run_game_events(table.clone()));
    }

    // TODO: Consider restricting CORS in the future
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let match_routes = Router::new()
        .route("/", get(get_state))
        .route("/ping", get(ping))
        .route("/pong", get(pong))
        .route_layer(middleware::from_fn_with_state(state.clone(), get_match));

    Router::new()
        .nest("/match/{id}", match_routes)
        .with_state(state)
        .layer(cors)
}

async fn get_match(
    State(state): State<AppState>,
    Path(uid): Path<String>,
    mut request: Request,
    next: Next,
) -> Response {
    let uid = match TableUid::parse(&uid) {
        Ok(uid) => uid,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("Invalid match id format: {uid}").into())
                .unwrap();
        }
    };

    let table_state = state
        .game_tables
        .read()
        .expect("game_tables read lock was poisoned")
        .get(&uid)
        .cloned();

    let table_state = match table_state {
        Some(table_state) => table_state,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(format!("No match with id {uid}").into())
                .unwrap();
            // TODO: create new match instead of 404
            // IF THE CONFIG IS SET TO ALLOW IT. And yeah, btw add config
            // config should also have `DEBUG` which would be used to add
            // info about having to allow that in config to the response.
        }
    };
    request.extensions_mut().insert(table_state.clone());

    Response::from(next.run(request).await)
}

async fn get_state(
    Extension(table_state): Extension<TableState>,
) -> (StatusCode, Json<TableState>) {
    (StatusCode::OK, Json(table_state))
}

async fn try_hit(side: Side, state: TableState) -> bool {
    let state_side = state
        .rally_state
        .read()
        .expect("rally_state read lock was poisoned")
        .side;

    if side == state_side {
        let mut rally_state = state
            .rally_state
            .write()
            .expect("rally_state write lock was poisoned");

        rally_state.side = (rally_state.side).flip();
        rally_state.hit_count += 1;
        rally_state.hit_timeout = Some(clock::now() + Duration::from_secs(BALL_AIR_TIME_SECONDS));
        rally_state.first_hit_at.get_or_insert_with(clock::now);

        true
    } else {
        state.lose_point(side).await;

        false
    }
}

async fn get_hit_response(side: Side, state: TableState) -> (StatusCode, String) {
    match try_hit(side, state).await {
        true => (StatusCode::OK, side.flip().to_string()),
        false => (StatusCode::CONFLICT, "MISS".to_string()),
    }
}

async fn ping(Extension(table_state): Extension<TableState>) -> (StatusCode, String) {
    get_hit_response(Side::Ping, table_state).await
}
async fn pong(Extension(table_state): Extension<TableState>) -> (StatusCode, String) {
    get_hit_response(Side::Pong, table_state).await
}
