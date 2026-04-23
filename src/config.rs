//! Configuration structs and environment-based loading for task-board.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

/// Top-level application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// PostgreSQL connection settings.
    pub database: DatabaseConfig,
    /// HTTP server settings.
    pub server: ServerConfig,
    /// Application behaviour settings.
    pub app: AppSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub database: String,
}

impl DatabaseConfig {
    pub fn connection_string(&self, redact: bool) -> String {
        let (u, p) = if redact {
            ("***".to_string(), "***".to_string())
        } else {
            (self.user.clone(), self.password.clone())
        };
        format!("postgres://{}:{}@{}:{}/{}", u, p, self.host, self.port, self.database)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    /// Whether to expose the Swagger UI at /api/docs.
    pub enable_docs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Default board canvas width used when placing new boxes (pixels).
    pub default_canvas_width: f64,
    /// Default board canvas height used when placing new boxes (pixels).
    pub default_canvas_height: f64,
    /// How many taps/clicks are required to delete a box.
    pub delete_tap_count: u8,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_canvas_width: 1280.0,
            default_canvas_height: 800.0,
            delete_tap_count: 3,
        }
    }
}

/// Load configuration from environment variables with sensible defaults.
pub fn load_config() -> Result<AppConfig> {
    let db = DatabaseConfig {
        user: env_or("DB_USER", "postgres"),
        password: env_or("DB_PASSWORD", "postgres"),
        host: env_or("DB_HOST", "localhost"),
        port: env_or("DB_PORT", "5432").parse::<u16>().context("DB_PORT must be a valid port")?,
        database: env_or("DB_NAME", "taskboard"),
    };

    let server = ServerConfig {
        host: env_or("SERVER_HOST", "0.0.0.0"),
        port: env_or("SERVER_PORT", "8080").parse::<u16>().context("SERVER_PORT must be a valid port")?,
        enable_docs: env_or("ENABLE_DOCS", "true").to_lowercase() != "false",
    };

    let app = AppSettings {
        default_canvas_width: env_or("DEFAULT_CANVAS_WIDTH", "1280").parse::<f64>().context("DEFAULT_CANVAS_WIDTH must be a number")?,
        default_canvas_height: env_or("DEFAULT_CANVAS_HEIGHT", "800").parse::<f64>().context("DEFAULT_CANVAS_HEIGHT must be a number")?,
        delete_tap_count: env_or("DELETE_TAP_COUNT", "3").parse::<u8>().context("DELETE_TAP_COUNT must be 1-255")?,
    };

    Ok(AppConfig { database: db, server, app })
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
