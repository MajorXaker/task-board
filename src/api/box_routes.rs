//! Axum handlers for phrase-box endpoints.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::api::{AppState, StateChange};
use crate::boxes::{db, CreateBoxRequest, UpdateBoxRequest};

/// `GET /api/boards/:board_id/boxes` — list all boxes on a board.
#[utoipa::path(
    get,
    path = "/api/boards/{board_id}/boxes",
    responses((status = 200, description = "Boxes for board", body = Vec<crate::boxes::PhraseBox>))
)]
pub async fn list_boxes(
    State(state): State<AppState>,
    Path(board_id): Path<i32>,
) -> impl IntoResponse {
    match db::get_boxes_for_board(&state.db_pool, board_id).await {
        Ok(boxes) => Json(boxes).into_response(),
        Err(e) => {
            warn!(error = %e, board_id, "Failed to list boxes");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// `GET /api/boxes` — list every box across all boards.
#[utoipa::path(
    get,
    path = "/api/boxes",
    responses((status = 200, description = "All boxes", body = Vec<crate::boxes::PhraseBox>))
)]
pub async fn list_all_boxes(State(state): State<AppState>) -> impl IntoResponse {
    match db::get_all_boxes(&state.db_pool).await {
        Ok(boxes) => Json(boxes).into_response(),
        Err(e) => {
            warn!(error = %e, "Failed to list all boxes");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// `POST /api/boards/:board_id/boxes` — create a new box.
#[utoipa::path(
    post,
    path = "/api/boards/{board_id}/boxes",
    request_body = crate::boxes::CreateBoxRequest,
    responses((status = 201, description = "Created box", body = crate::boxes::PhraseBox))
)]
pub async fn create_box(
    State(state): State<AppState>,
    Path(board_id): Path<i32>,
    Json(mut body): Json<CreateBoxRequest>,
) -> impl IntoResponse {
    info!(
        board_id,
        text = %body.text,
        color_bg = %body.color_bg,
        color_text = %body.color_text,
        pos_x = ?body.pos_x,
        pos_y = ?body.pos_y,
        "create_box: request received"
    );

    // Scatter new boxes within the configured default canvas area
    if body.pos_x.is_none() {
        body.pos_x = Some(40.0 + rand_offset(state.app_settings.default_canvas_width - 340.0));
    }
    if body.pos_y.is_none() {
        body.pos_y = Some(40.0 + rand_offset(state.app_settings.default_canvas_height - 120.0));
    }

    debug!(
        board_id,
        pos_x = body.pos_x.unwrap(),
        pos_y = body.pos_y.unwrap(),
        "create_box: resolved position"
    );

    match db::create_box(&state.db_pool, &body, body.pos_x.unwrap(), body.pos_y.unwrap(), board_id).await {
        Ok(b) => {
            info!(
                board_id,
                box_id = %b.id,
                text = %b.text,
                "create_box: box inserted successfully"
            );
            let broadcast_result = state.tx.send(StateChange::BoxCreated(b.clone()));
            debug!(
                box_id = %b.id,
                listeners = broadcast_result.as_ref().map(|n| *n).unwrap_or(0),
                "create_box: broadcast sent"
            );
            (StatusCode::CREATED, Json(b)).into_response()
        }
        Err(e) => {
            warn!(
                error = %e,
                board_id,
                text = %body.text,
                "create_box: DB insert FAILED"
            );
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// `PATCH /api/boxes/:id` — update position, text, color, or board assignment.
#[utoipa::path(
    patch,
    path = "/api/boxes/{id}",
    request_body = crate::boxes::UpdateBoxRequest,
    responses(
        (status = 200, description = "Updated box", body = crate::boxes::PhraseBox),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_box(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateBoxRequest>,
) -> impl IntoResponse {
    match db::update_box(&state.db_pool, id, &body).await {
        Ok(Some(b)) => {
            let _ = state.tx.send(StateChange::BoxUpdated(b.clone()));
            Json(b).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            warn!(error = %e, %id, "Failed to update box");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// `DELETE /api/boxes/:id` — remove a box.
#[utoipa::path(
    delete,
    path = "/api/boxes/{id}",
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_box(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let board_id = match sqlx::query_scalar::<_, i32>("SELECT board_id FROM boxes WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db_pool)
        .await
    {
        Ok(Some(bid)) => bid,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            warn!(error = %e, %id, "Failed to look up box before delete");
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    match db::delete_box(&state.db_pool, id).await {
        Ok(true) => {
            let _ = state.tx.send(StateChange::BoxDeleted { id, board_id });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            warn!(error = %e, %id, "Failed to delete box");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Deterministic-ish scatter offset using system time nanoseconds as cheap randomness.
fn rand_offset(max: f64) -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0) as f64;
    (nanos % (max.max(1.0) as f64)).abs()
}
