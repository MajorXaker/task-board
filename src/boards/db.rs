//! PostgreSQL persistence for boards.

use anyhow::{Context, Result};
use sqlx::PgPool;

use super::{Board, UpdateBoardRequest};

/// Return all 5 boards ordered by slot.
pub async fn get_all_boards(pool: &PgPool) -> Result<Vec<Board>> {
    sqlx::query_as::<_, Board>(
        "SELECT id, slot, title, color, created_at, updated_at FROM boards ORDER BY slot",
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch boards")
}

/// Return a single board by id.
pub async fn get_board(pool: &PgPool, id: i32) -> Result<Option<Board>> {
    sqlx::query_as::<_, Board>(
        "SELECT id, slot, title, color, created_at, updated_at FROM boards WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .context("Failed to fetch board")
}

/// Update a board's title and/or color. Returns the updated board.
pub async fn update_board(pool: &PgPool, id: i32, req: &UpdateBoardRequest) -> Result<Option<Board>> {
    let row = sqlx::query_as::<_, Board>(
        r#"
        UPDATE boards
        SET title      = COALESCE($1, title),
            color      = COALESCE($2, color),
            updated_at = NOW()
        WHERE id = $3
        RETURNING id, slot, title, color, created_at, updated_at
        "#,
    )
    .bind(req.title.as_deref())
    .bind(req.color.as_deref())
    .bind(id)
    .fetch_optional(pool)
    .await
    .context("Failed to update board")?;

    Ok(row)
}

/// Number of boards (used to enforce the minimum of one board).
pub async fn count_boards(pool: &PgPool) -> Result<i64> {
    sqlx::query_scalar("SELECT COUNT(*) FROM boards")
        .fetch_one(pool)
        .await
        .context("Failed to count boards")
}

/// Number of boxes on a board (boards with boxes cannot be deleted).
pub async fn count_boxes_for_board(pool: &PgPool, board_id: i32) -> Result<i64> {
    sqlx::query_scalar("SELECT COUNT(*) FROM boxes WHERE board_id = $1")
        .bind(board_id)
        .fetch_one(pool)
        .await
        .context("Failed to count boxes for board")
}

/// Delete a board. Returns the deleted board, or `None` if it did not exist.
pub async fn delete_board(pool: &PgPool, id: i32) -> Result<Option<Board>> {
    let row = sqlx::query_as::<_, Board>(
        r#"
        DELETE FROM boards
        WHERE id = $1
        RETURNING id, slot, title, color, created_at, updated_at
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .context("Failed to delete board")?;

    Ok(row)
}
