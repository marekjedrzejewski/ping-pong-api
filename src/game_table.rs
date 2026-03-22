use std::time::Duration;

use axum::{
    Extension, Json, Router,
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use log::error;
use tokio::time::sleep;

use crate::{
    BALL_AIR_TIME_SECONDS, clock,
    database::{TableUid, create_new_match},
    models::{
        application::AppState,
        game::{Side, TableState},
    },
};

pub fn match_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_state))
        .route("/ping", get(ping))
        .route("/pong", get(pong))
        .route_layer(middleware::from_fn_with_state(state, get_or_create_match))
}

async fn get_or_create_match(
    State(state): State<AppState>,
    Path(uid): Path<String>,
    mut request: Request,
    next: Next,
) -> Response {
    let uid = match TableUid::parse(&uid) {
        Ok(uid) => uid,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
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
            // TODO: create new match IF THE CONFIG IS SET TO ALLOW IT.
            // And yeah, btw add config.
            // config should also have `DEBUG` which would be used to add
            // info about having to allow that in config to the response.
            match create_new_match(&state.db_pool, &uid).await {
                Ok(table_state) => {
                    state
                        .game_tables
                        .write()
                        .expect("game_tables write lock was poisoned")
                        .insert(uid, table_state.clone());

                    table_state
                }
                Err(e) => {
                    error!("Failed to create new match with id {uid}: {e}");
                    return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
                }
            }
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
    let did_hit = {
        let mut rally_state = state
            .rally_state
            .write()
            .expect("rally_state write lock was poisoned");

        if let Some(hit_timeout_task) = rally_state.hit_timeout_task.take() {
            hit_timeout_task.abort();
        }

        if side == rally_state.side {
            rally_state.side = side.flip();
            rally_state.hit_count += 1;
            rally_state.hit_timeout =
                Some(clock::now() + Duration::from_secs(BALL_AIR_TIME_SECONDS));
            rally_state.first_hit_at.get_or_insert_with(clock::now);

            let state_clone = state.clone();
            rally_state.hit_timeout_task = Some(tokio::spawn(async move {
                sleep(Duration::from_secs(BALL_AIR_TIME_SECONDS)).await;
                state_clone.lose_point(side.flip()).await;
            }));

            true
        } else {
            false
        }
    };
    if !did_hit {
        state.lose_point(side).await;
    };

    did_hit
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
