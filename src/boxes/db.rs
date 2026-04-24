//! PostgreSQL persistence for phrase boxes.

use anyhow::{Context, Result};
use sqlx::PgPool;
use uuid::Uuid;

use super::{CreateBoxRequest, PhraseBox, UpdateBoxRequest};

/// Return all boxes for a given board, ordered by z_index then created_at.
pub async fn get_boxes_for_board(pool: &PgPool, board_id: i32) -> Result<Vec<PhraseBox>> {
    sqlx::query_as::<_, PhraseBox>(
        r#"
        SELECT id, board_id, text, color_bg, color_text, pos_x, pos_y, z_index, created_at, updated_at
        FROM boxes
        WHERE board_id = $1
        ORDER BY z_index ASC, created_at ASC
        "#,
    )
    .bind(board_id)
    .fetch_all(pool)
    .await
    .context("Failed to fetch boxes")
}

/// Return all boxes across all boards (for collaborative sync).
pub async fn get_all_boxes(pool: &PgPool) -> Result<Vec<PhraseBox>> {
    sqlx::query_as::<_, PhraseBox>(
        r#"
        SELECT id, board_id, text, color_bg, color_text, pos_x, pos_y, z_index, created_at, updated_at
        FROM boxes
        ORDER BY board_id, z_index ASC, created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch all boxes")
}

/// Insert a new box. Returns the created record.
pub async fn create_box(pool: &PgPool, req: &CreateBoxRequest, default_x: f64, default_y: f64, board_id: i32) -> Result<PhraseBox> {
    let x = req.pos_x.unwrap_or(default_x);
    let y = req.pos_y.unwrap_or(default_y);

    sqlx::query_as::<_, PhraseBox>(
        r#"
        INSERT INTO boxes (board_id, text, color_bg, color_text, pos_x, pos_y)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, board_id, text, color_bg, color_text, pos_x, pos_y, z_index, created_at, updated_at
        "#,
    )
    .bind(board_id)
    .bind(&req.text)
    .bind(&req.color_bg)
    .bind(&req.color_text)
    .bind(x)
    .bind(y)
    .fetch_one(pool)
    .await
    .context("Failed to create box")
}

/// Update a box's fields selectively. Returns the updated record or None if not found.
pub async fn update_box(pool: &PgPool, id: Uuid, req: &UpdateBoxRequest) -> Result<Option<PhraseBox>> {
    sqlx::query_as::<_, PhraseBox>(
        r#"
        UPDATE boxes
        SET text       = COALESCE($1, text),
            color_bg   = COALESCE($2, color_bg),
            color_text = COALESCE($3, color_text),
            pos_x      = COALESCE($4, pos_x),
            pos_y      = COALESCE($5, pos_y),
            z_index    = COALESCE($6, z_index),
            board_id   = COALESCE($7, board_id),
            updated_at = NOW()
        WHERE id = $8
        RETURNING id, board_id, text, color_bg, color_text, pos_x, pos_y, z_index, created_at, updated_at
        "#,
    )
    .bind(req.text.as_deref())
    .bind(req.color_bg.as_deref())
    .bind(req.color_text.as_deref())
    .bind(req.pos_x)
    .bind(req.pos_y)
    .bind(req.z_index)
    .bind(req.board_id)
    .bind(id)
    .fetch_optional(pool)
    .await
    .context("Failed to update box")
}

/// Delete a box by id. Returns true if a row was deleted.
pub async fn delete_box(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM boxes WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .context("Failed to delete box")?;
    Ok(res.rows_affected() == 1)
}
