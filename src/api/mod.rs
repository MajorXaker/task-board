//! API layer: router setup, shared state, WebSocket broadcaster.

pub mod board_routes;
pub mod box_routes;
pub mod config_routes;

use std::sync::Arc;
use tokio::sync::broadcast;

use axum::{
    extract::Request,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
    Json, Router,
};
use sqlx::PgPool;
use tower_http::services::ServeDir;
use tracing::debug;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::config_routes::ClientConfig;
use crate::boards::{Board, UpdateBoardRequest};
use crate::boxes::{BoardSnapshot, CreateBoxRequest, PhraseBox, UpdateBoxRequest};
use crate::config::AppSettings;

/// Shared state available to every handler.
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub app_settings: AppSettings,
    pub tx: Arc<broadcast::Sender<StateChange>>,
}

/// Lightweight event broadcast over WebSocket.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "event", content = "data")]
pub enum StateChange {
    BoxCreated(PhraseBox),
    BoxUpdated(PhraseBox),
    BoxDeleted { id: uuid::Uuid, board_id: i32 },
    BoardCreated(Board),
    BoardUpdated(Board),
    BoardDeleted { board_id: i32 },
    BoardCleared { board_id: i32 },
}

/// `GET /api/health`
#[utoipa::path(
    get,
    path = "/api/__heartbeat__",
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
        board_routes::create_board,
        board_routes::get_board,
        board_routes::update_board,
        board_routes::delete_board,
        board_routes::clear_board,
        box_routes::list_boxes,
        box_routes::list_all_boxes,
        box_routes::create_box,
        box_routes::update_box,
        box_routes::delete_box,
    ),
    components(schemas(
        Board,
        UpdateBoardRequest,
        crate::boards::CreateBoardRequest,
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

/// Build the full axum [`Router`].
///
/// ## Routing rules
/// - All API endpoints live under `/api/…`
/// - Static files are served from `static_dir` for paths that match actual files
/// - Everything else (unknown paths, SPA client routes) falls back to `index.html`
/// - The `/api` prefix is matched first; `ServeDir` never sees `/api/…` paths
pub fn create_router(state: AppState, enable_docs: bool, static_dir: &str) -> Router {
    let mut api = Router::new()
        .route("/__heartbeat__", get(health))
        .route("/config", get(config_routes::get_config))
        // boards collection
        .route(
            "/boards",
            get(board_routes::list_boards)
                .post(board_routes::create_board),
        )
        // single board
        .route(
            "/boards/{id}",
            get(board_routes::get_board)
                .patch(board_routes::update_board)
                .delete(board_routes::delete_board),
        )
        // boxes on a board
        .route(
            "/boards/{board_id}/boxes",
            post(box_routes::create_box).get(box_routes::list_boxes),
        )
        // clear all boxes on a board
        .route(
            "/boards/{board_id}/boxes/clear",
            delete(board_routes::clear_board),
        )
        // all boxes across all boards
        .route("/boxes", get(box_routes::list_all_boxes))
        // single box
        .route(
            "/boxes/{id}",
            patch(box_routes::update_box).delete(box_routes::delete_box),
        )
        .with_state(state.clone());

    // let api = if enable_docs {
    //     debug!("Enabling docs endpoint");
    //     api.merge(SwaggerUi::new("/docs").url("/api/docs/openapi.json", ApiDoc::openapi()))
    // } else {
    //     api
    // };


    // ── Static file serving + SPA fallback ─────────────────────────────────
    //
    // Strategy: mount the API router under /api first (highest priority),
    // then serve static files with ServeDir.  ServeDir only handles paths
    // that do NOT start with /api — enforced by the middleware below.
    // Paths that don't match any static file are rewritten to /index.html
    // so client-side routing works.

    // let static_dir = static_dir.to_string();
    // let spa_fallback = axum::routing::get(move || {
    //     let path = format!("{}/index.html", static_dir);
    //     async move {
    //         match tokio::fs::read(path).await {
    //             Ok(bytes) => axum::response::Response::builder()
    //                 .header("Content-Type", "text/html; charset=utf-8")
    //                 .body(axum::body::Body::from(bytes))
    //                 .unwrap(),
    //             Err(_) => axum::response::Response::builder()
    //                 .status(404)
    //                 .body(axum::body::Body::from("index.html not found"))
    //                 .unwrap(),
    //         }
    //     }
    // });

    let mut root = Router::new()
        .nest("/api", api);
        // .nest_service("/static", ServeDir::new("static"))
        // .fallback(spa_fallback.clone());

    if enable_docs {
        debug!("Enabling docs endpoint");
        root = root.merge(
            SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", ApiDoc::openapi()),
        );
    }

    root
}
