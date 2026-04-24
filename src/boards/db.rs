//! PostgreSQL persistence for boards.

use anyhow::{Context, Result};
use sqlx::PgPool;

use super::{Board, CreateBoardRequest, UpdateBoardRequest};

/// Return all boards ordered by slot.
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

/// Return the current maximum slot number (0 if no boards exist).
pub async fn max_slot(pool: &PgPool) -> Result<i16> {
    let n: Option<i16> = sqlx::query_scalar("SELECT MAX(slot) FROM boards")
        .fetch_one(pool)
        .await
        .context("Failed to query max slot")?;
    Ok(n.unwrap_or(0))
}

/// Return the current board count.
pub async fn board_count(pool: &PgPool) -> Result<i64> {
    sqlx::query_scalar("SELECT COUNT(*) FROM boards")
        .fetch_one(pool)
        .await
        .context("Failed to count boards")
}

/// Insert a new board. Assigns slot = max_slot + 1.
pub async fn create_board(pool: &PgPool, req: &CreateBoardRequest) -> Result<Board> {
    let next_slot = max_slot(pool).await? + 1;
    let color = req.color.as_deref().unwrap_or("#4f98a3");
    sqlx::query_as::<_, Board>(
        r#"
        INSERT INTO boards (slot, title, color)
        VALUES ($1, $2, $3)
        RETURNING id, slot, title, color, created_at, updated_at
        "#,
    )
    .bind(next_slot)
    .bind(&req.title)
    .bind(color)
    .fetch_one(pool)
    .await
    .context("Failed to create board")
}

/// Update a board's title and/or color. Returns the updated board.
pub async fn update_board(pool: &PgPool, id: i32, req: &UpdateBoardRequest) -> Result<Option<Board>> {
    sqlx::query_as::<_, Board>(
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
    .context("Failed to update board")
}

/// Delete a board and all its boxes (cascade). Re-sequences remaining slots.
pub async fn delete_board(pool: &PgPool, id: i32) -> Result<bool> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let res = sqlx::query("DELETE FROM boards WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .context("Failed to delete board")?;

    if res.rows_affected() == 0 {
        return Ok(false);
    }

    // Re-sequence slots to keep them contiguous (1, 2, 3, …)
    sqlx::query(
        r#"
        UPDATE boards b
        SET slot = seq.new_slot, updated_at = NOW()
        FROM (
            SELECT id, ROW_NUMBER() OVER (ORDER BY slot) AS new_slot
            FROM boards
        ) seq
        WHERE b.id = seq.id
        "#,
    )
    .execute(&mut *tx)
    .await
    .context("Failed to re-sequence slots")?;

    tx.commit().await.context("Failed to commit delete transaction")?;
    Ok(true)
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
