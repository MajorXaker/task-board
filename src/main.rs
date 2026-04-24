//! task-board — Collaborative drag-and-drop task tracker
//!
//! Entry point: loads config, connects to Postgres, runs migrations,
//! sets up the WebSocket broadcast channel, then starts the axum server.

mod api;
mod boards;
mod boxes;
mod config;

use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

use api::{AppState, create_router, StateChange};

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    // 2. Load configuration from environment variables.
    dotenvy::dotenv().ok(); // silently ignores missing .env
    let cfg = config::load_config().unwrap_or_else(|e| {
        error!(error = %e, "Configuration error — aborting");
        std::process::exit(1);
    });

    // 1. Structured logging.
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(cfg.server.log_level.clone())),
        )
        .init();

    info!("task-board starting up");

    // 2. Load configuration from environment variables.
    dotenvy::dotenv().ok(); // silently ignores missing .env
    let cfg = config::load_config().unwrap_or_else(|e| {
        error!(error = %e, "Configuration error — aborting");
        std::process::exit(1);
    });

    info!(
        url = cfg.database.connection_string(true),
        "Connecting to PostgreSQL"
    );

    // 3. Connect to PostgreSQL.
    let pool = sqlx::PgPool::connect(&cfg.database.connection_string(false))
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "Failed to connect to PostgreSQL — aborting");
            std::process::exit(1);
        });

    // 4. Run migrations (idempotent).
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "Failed to run DB migrations — aborting");
            std::process::exit(1);
        });
    info!("Database migrations applied");

    // 5. Build the shared broadcast channel for WebSocket push.
    let (tx, _) = broadcast::channel::<StateChange>(256);
    let tx = Arc::new(tx);

    // 6. Build shared app state.
    let state = AppState {
        db_pool: pool,
        app_settings: cfg.app.clone(),
        tx,
    };

    // 7. Determine the static files directory.
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "./static".to_string());

    // 8. Build the axum router.
    let router = create_router(state, cfg.server.enable_docs, &static_dir);

    // 9. Start the HTTP server.
    let bind_addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    info!(address = %format!("http://{}", bind_addr), "HTTP server listening");

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, address = %bind_addr, "Failed to bind TCP listener — aborting");
            std::process::exit(1);
        });

    axum::serve(listener, router).await.unwrap_or_else(|e| {
        error!(error = %e, "HTTP server error");
        std::process::exit(1);
    });

    Ok(())
}
