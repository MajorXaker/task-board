//! Domain types and database helpers for phrase boxes.

pub mod db;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// A draggable text box living on a board.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct PhraseBox {
    pub id: Uuid,
    pub board_id: i32,
    pub text: String,
    pub color_bg: String,
    pub color_text: String,
    pub pos_x: f64,
    pub pos_y: f64,
    pub z_index: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload for creating a new box.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBoxRequest {
    pub board_id: i32,
    pub text: String,
    pub color_bg: String,
    pub color_text: String,
    /// Optional explicit position; backend uses defaults when absent.
    pub pos_x: Option<f64>,
    pub pos_y: Option<f64>,
}

/// Payload for moving/updating a box.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateBoxRequest {
    pub text: Option<String>,
    pub color_bg: Option<String>,
    pub color_text: Option<String>,
    pub pos_x: Option<f64>,
    pub pos_y: Option<f64>,
    pub z_index: Option<i32>,
    pub board_id: Option<i32>,
}

/// Full board snapshot sent to clients on poll / WS push.
#[derive(Debug, Serialize, ToSchema)]
pub struct BoardSnapshot {
    pub board_id: i32,
    pub boxes: Vec<PhraseBox>,
}
