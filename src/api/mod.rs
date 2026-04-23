//! API layer: router setup, shared state, WebSocket broadcaster.

pub mod board_routes;
pub mod box_routes;
pub mod config_routes;
pub mod ws;

use std::sync::Arc;
use tokio::sync::broadcast;

use axum::{
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::boxes::PhraseBox;
use crate::config::AppSettings;
use crate::boards::Board;
use crate::boxes::{BoardSnapshot, CreateBoxRequest, UpdateBoxRequest};
use crate::boards::UpdateBoardRequest;
use crate::api::config_routes::ClientConfig;

/// Shared state available to every handler.
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub app_settings: AppSettings,
    /// Broadcast channel: whenever a box is created/moved/deleted we push a
    /// `StateChange` to all connected WebSocket clients.
    pub tx: Arc<broadcast::Sender<StateChange>>,
}

/// Lightweight event broadcast over WebSocket.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "event", content = "data")]
pub enum StateChange {
    BoxCreated(PhraseBox),
    BoxUpdated(PhraseBox),
    BoxDeleted { id: uuid::Uuid, board_id: i32 },
    BoardUpdated(Board),
}

/// `GET /api/health` — public liveness probe.
#[utoipa::path(
    get,
    path = "/api/health",
    responses((status = 200, description = "Service is alive"))
)]
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        config_routes::get_config,
        board_routes::list_boards,
        board_routes::get_board,
        board_routes::update_board,
        box_routes::list_boxes,
        box_routes::list_all_boxes,
        box_routes::create_box,
        box_routes::update_box,
        box_routes::delete_box,
    ),
    components(schemas(
        Board,
        UpdateBoardRequest,
        PhraseBox,
        BoardSnapshot,
        CreateBoxRequest,
        UpdateBoxRequest,
        ClientConfig,
    )),
    info(title = "TaskBoard API", version = "0.1.0",
         description = "Collaborative drag-and-drop task board"),
)]
pub struct ApiDoc;

/// Build and return the full axum [`Router`].
pub fn create_router(state: AppState, enable_docs: bool, static_dir: &str) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .route("/config", get(config_routes::get_config))
        // boards
        .route("/boards", get(board_routes::list_boards))
        .route("/boards/{id}", get(board_routes::get_board))
        .route("/boards/{id}", patch(board_routes::update_board))
        // boxes
        .route("/boards/{board_id}/boxes", get(box_routes::list_boxes))
        .route("/boards/{board_id}/boxes", post(box_routes::create_box))
        .route("/boxes", get(box_routes::list_all_boxes))
        .route("/boxes/{id}", patch(box_routes::update_box))
        .route("/boxes/{id}", delete(box_routes::delete_box))
        // websocket
        .route("/ws", get(ws::ws_handler))
        .with_state(state.clone());

    let api = if enable_docs {
        api.merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", ApiDoc::openapi()))
    } else {
        api
    };

    // Serve the frontend SPA from /static, with fallback to index.html
    use tower_http::services::{ServeDir, ServeFile};
    Router::new()
        .nest("/api", api)
        .nest_service(
            "/",
            ServeDir::new(static_dir).not_found_service(ServeFile::new(
                format!("{}/index.html", static_dir),
            )),
        )
}
