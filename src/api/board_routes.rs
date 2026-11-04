//! Axum handlers for board endpoints.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::warn;

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

/// `DELETE /api/boards/:id` — delete a board.
///
/// Rules: the board must exist, must not contain any boxes, and must not be
/// the last remaining board.
#[utoipa::path(
    delete,
    path = "/api/boards/{id}",
    responses(
        (status = 204, description = "Board deleted"),
        (status = 400, description = "Cannot delete the last remaining board"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Board still has boxes"),
    )
)]
pub async fn delete_board(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    // Board must exist.
    match db::get_board(&state.db_pool, id).await {
        Ok(Some(_)) => {}
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            warn!(error = %e, id, "Failed to get board");
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    }

    // Boards with boxes cannot be deleted.
    match db::count_boxes_for_board(&state.db_pool, id).await {
        Ok(n) if n > 0 => {
            return (
                StatusCode::CONFLICT,
                "Board still has boxes — move or delete them first".to_string(),
            )
                .into_response();
        }
        Ok(_) => {}
        Err(e) => {
            warn!(error = %e, id, "Failed to count boxes for board");
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    }

    // At least one board must remain.
    match db::count_boards(&state.db_pool).await {
        Ok(n) if n <= 1 => {
            return (
                StatusCode::BAD_REQUEST,
                "Cannot delete the last remaining board".to_string(),
            )
                .into_response();
        }
        Ok(_) => {}
        Err(e) => {
            warn!(error = %e, "Failed to count boards");
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    }

    match db::delete_board(&state.db_pool, id).await {
        Ok(Some(_)) => {
            let _ = state.tx.send(StateChange::BoardDeleted { id });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            warn!(error = %e, id, "Failed to delete board");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
