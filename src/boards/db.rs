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

/// Delete all boxes belonging to a board. Returns the number of rows deleted.
pub async fn clear_board(pool: &PgPool, board_id: i32) -> Result<u64> {
    let res = sqlx::query("DELETE FROM boxes WHERE board_id = $1")
        .bind(board_id)
        .execute(pool)
        .await
        .context("Failed to clear board")?;
    Ok(res.rows_affected())
}
