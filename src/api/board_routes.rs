//! Axum handlers for board endpoints.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::{info, warn};

use crate::api::{AppState, StateChange};
use crate::boards::{db, UpdateBoardRequest};

/// `GET /api/boards` — list all 5 boards.
#[utoipa::path(
    get,
    path = "/api/boards",
    responses((status = 200, description = "All boards", body = Vec<crate::boards::Board>))
)]
pub async fn list_boards(State(state): State<AppState>) -> impl IntoResponse {
    match db::get_all_boards(&state.db_pool).await {
        Ok(boards) => Json(boards).into_response(),
        Err(e) => {
            warn!(error = %e, "Failed to list boards");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// `GET /api/boards/:id` — fetch a single board.
#[utoipa::path(
    get,
    path = "/api/boards/{id}",
    responses(
        (status = 200, description = "Board found", body = crate::boards::Board),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_board(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match db::get_board(&state.db_pool, id).await {
        Ok(Some(board)) => Json(board).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            warn!(error = %e, id, "Failed to get board");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// `PATCH /api/boards/:id` — update board title / color.
#[utoipa::path(
    patch,
    path = "/api/boards/{id}",
    request_body = crate::boards::UpdateBoardRequest,
    responses(
        (status = 200, description = "Updated board", body = crate::boards::Board),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_board(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<UpdateBoardRequest>,
) -> impl IntoResponse {
    match db::update_board(&state.db_pool, id, &body).await {
        Ok(Some(board)) => {
            let _ = state.tx.send(StateChange::BoardUpdated(board.clone()));
            Json(board).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            warn!(error = %e, id, "Failed to update board");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// `DELETE /api/boards/:board_id/boxes/clear` — remove all boxes from a board.
///
/// Boards themselves are never deleted (there are always exactly 5 slots);
/// this endpoint just wipes all boxes belonging to the board.
#[utoipa::path(
    delete,
    path = "/api/boards/{board_id}/boxes/clear",
    responses(
        (status = 204, description = "All boxes on this board deleted"),
        (status = 404, description = "Board not found"),
    )
)]
pub async fn clear_board(
    State(state): State<AppState>,
    Path(board_id): Path<i32>,
) -> impl IntoResponse {
    // Verify the board exists first.
    match db::get_board(&state.db_pool, board_id).await {
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            warn!(error = %e, board_id, "Failed to look up board before clear");
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
        Ok(Some(_)) => {}
    }

    match db::clear_board(&state.db_pool, board_id).await {
        Ok(count) => {
            info!(board_id, deleted = count, "Board cleared");
            let _ = state.tx.send(StateChange::BoardCleared { board_id });
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            warn!(error = %e, board_id, "Failed to clear board");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
