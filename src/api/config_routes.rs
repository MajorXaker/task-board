//! Exposes non-sensitive runtime configuration to the frontend.

use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::api::AppState;

#[derive(Serialize, ToSchema)]
pub struct ClientConfig {
    /// Number of taps/clicks required to delete a box.
    pub delete_tap_count: u8,
    /// Default canvas width (informational).
    pub default_canvas_width: f64,
    /// Default canvas height (informational).
    pub default_canvas_height: f64,
    /// When true, the frontend should show a dedicated "Manage" tab for
    /// adding boxes and editing boards instead of the toolbar controls.
    pub separate_add_tab: bool,
}

/// `GET /api/config` — return client-relevant runtime settings.
#[utoipa::path(
    get,
    path = "/api/config",
    responses((status = 200, description = "Client configuration", body = ClientConfig))
)]
pub async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    Json(ClientConfig {
        delete_tap_count: state.app_settings.delete_tap_count,
        default_canvas_width: state.app_settings.default_canvas_width,
        default_canvas_height: state.app_settings.default_canvas_height,
        separate_add_tab: state.app_settings.separate_add_tab,
    })
}
