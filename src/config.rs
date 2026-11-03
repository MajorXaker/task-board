//! Configuration structs and environment-based loading for task-board.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use utoipa::ToSchema;

/// One selectable box color: a background hex value plus a readable text hex value.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BoxColor {
    /// Card background, `#rrggbb`.
    pub bg: String,
    /// Card text color, `#rrggbb`.
    pub text: String,
    /// Human-readable label shown as the swatch tooltip.
    pub label: String,
}

/// Top-level application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
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
    /// When true, the "Add / Manage" controls move to a dedicated tab
    /// instead of living in the top toolbar.
    pub separate_add_tab: bool,
    /// Colors offered for new boxes (`BOX_COLOR_*` env vars; built-in
    /// dark palette when none are set). Boxes already created keep their
    /// stored colors regardless of this list.
    pub box_colors: Vec<BoxColor>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_canvas_width: 1280.0,
            default_canvas_height: 800.0,
            delete_tap_count: 3,
            separate_add_tab: false,
            box_colors: default_box_colors(),
        }
    }
}

/// Built-in dark-theme palette (8 colors).
fn default_box_colors() -> Vec<BoxColor> {
    vec![
        BoxColor { bg: "#422006".into(), text: "#fde68a".into(), label: "Yellow".into()  },
        BoxColor { bg: "#052e16".into(), text: "#86efac".into(), label: "Green".into()   },
        BoxColor { bg: "#0f1e40".into(), text: "#93c5fd".into(), label: "Blue".into()    },
        BoxColor { bg: "#3d0a22".into(), text: "#f9a8d4".into(), label: "Pink".into()    },
        BoxColor { bg: "#3d1604".into(), text: "#fdba74".into(), label: "Orange".into()  },
        BoxColor { bg: "#200d40".into(), text: "#c4b5fd".into(), label: "Purple".into()  },
        BoxColor { bg: "#042420".into(), text: "#5eead4".into(), label: "Teal".into()    },
        BoxColor { bg: "#1e2530".into(), text: "#cbd5e1".into(), label: "Slate".into()   },
    ]
}

/// Read `BOX_COLOR_1` … `BOX_COLOR_32` from the environment.
///
/// Each entry is `"#rrggbb,#rrggbb"` (background, text). Indices with a
/// missing variable are skipped; invalid entries are skipped with a warning.
/// Returns `None` when no `BOX_COLOR_*` variables are set at all (use defaults).
fn load_box_colors_from_env() -> Option<Vec<BoxColor>> {
    let mut colors = Vec::new();
    for i in 1..=32 {
        let raw = match env::var(format!("BOX_COLOR_{i}")) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let parts: Vec<&str> = raw.split(',').map(|p| p.trim()).collect();
        if parts.len() != 2 || !is_hex_color(parts[0]) || !is_hex_color(parts[1]) {
            tracing::warn!(key = i, value = %raw, "Invalid BOX_COLOR_ entry, expected '#rrggbb,#rrggbb' — skipping");
            continue;
        }
        colors.push(BoxColor {
            bg: parts[0].to_string(),
            text: parts[1].to_string(),
            label: format!("Color {i}"),
        });
    }
    if colors.is_empty() { None } else { Some(colors) }
}

/// `#rrggbb` check.
fn is_hex_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
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
        default_canvas_width: env_or("DEFAULT_CANVAS_WIDTH", "1280")
            .parse::<f64>()
            .context("DEFAULT_CANVAS_WIDTH must be a number")?,
        default_canvas_height: env_or("DEFAULT_CANVAS_HEIGHT", "800")
            .parse::<f64>()
            .context("DEFAULT_CANVAS_HEIGHT must be a number")?,
        delete_tap_count: env_or("DELETE_TAP_COUNT", "3")
            .parse::<u8>()
            .context("DELETE_TAP_COUNT must be 1-255")?,
        separate_add_tab: env_or("SEPARATE_ADD_TAB", "false").to_lowercase() == "true",
        box_colors: load_box_colors_from_env().unwrap_or_else(default_box_colors),
    };

    Ok(AppConfig { database: db, server, app })
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
