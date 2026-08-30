//! Domain types and database helpers for boards.

pub mod db;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A named, coloured lane that owns a set of boxes.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Board {
    pub id: i32,
    /// Slot number (order position, 1-based).
    pub slot: i16,
    pub title: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload for creating a new board.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBoardRequest {
    pub title: String,
    pub color: Option<String>,
}

/// Payload accepted when updating a board's metadata.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateBoardRequest {
    pub title: Option<String>,
    pub color: Option<String>,
}
