//! Axum handlers for board endpoints.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::{info, warn};

use crate::api::{AppState, StateChange};
use crate::boards::{db, CreateBoardRequest, UpdateBoardRequest};

/// `GET /api/boards`
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

/// `POST /api/boards`
#[utoipa::path(
    post,
    path = "/api/boards",
    request_body = crate::boards::CreateBoardRequest,
    responses(
        (status = 201, description = "Created board", body = crate::boards::Board),
        (status = 409, description = "Board limit reached"),
    )
)]
pub async fn create_board(
    State(state): State<AppState>,
    Json(body): Json<CreateBoardRequest>,
) -> impl IntoResponse {
    let max = state.app_settings.max_boards as i64;
    match db::board_count(&state.db_pool).await {
        Ok(count) if count >= max => {
            return (
                StatusCode::CONFLICT,
                format!("Board limit of {} reached", max),
            )
                .into_response();
        }
        Err(e) => {
            warn!(error = %e, "Failed to count boards");
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
        _ => {}
    }

    match db::create_board(&state.db_pool, &body).await {
        Ok(board) => {
            info!(board_id = board.id, title = %board.title, "Board created");
            let _ = state.tx.send(StateChange::BoardCreated(board.clone()));
            (StatusCode::CREATED, Json(board)).into_response()
        }
        Err(e) => {
            warn!(error = %e, "Failed to create board");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// `GET /api/boards/:id`
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

/// `PATCH /api/boards/:id`
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

/// `DELETE /api/boards/:id` — delete a board and all its boxes.
#[utoipa::path(
    delete,
    path = "/api/boards/{id}",
    responses(
        (status = 204, description = "Board deleted"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_board(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match db::delete_board(&state.db_pool, id).await {
        Ok(true) => {
            info!(board_id = id, "Board deleted");
            let _ = state.tx.send(StateChange::BoardDeleted { board_id: id });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            warn!(error = %e, id, "Failed to delete board");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// `DELETE /api/boards/:board_id/boxes/clear` — remove all boxes from a board.
#[utoipa::path(
    delete,
    path = "/api/boards/{board_id}/boxes/clear",
    responses(
        (status = 204, description = "All boxes deleted"),
        (status = 404, description = "Board not found"),
    )
)]
pub async fn clear_board(
    State(state): State<AppState>,
    Path(board_id): Path<i32>,
) -> impl IntoResponse {
    match db::get_board(&state.db_pool, board_id).await {
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
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
