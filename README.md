# TaskBoard

Collaborative drag-and-drop day-task tracker. Run it on a touch tablet for quick task tracking, and add phrases from a desktop with a keyboard — changes appear in real time on all connected devices.

## Stack

| Layer | Tech |
|-------|------|
| Backend | Rust · Axum 0.7 · sqlx 0.7 |
| Database | PostgreSQL 16 |
| Frontend | Vanilla HTML / CSS / JS (single file) |
| Transport | REST + WebSocket push + 1 s polling fallback |
| Docs | utoipa + Swagger UI at `/api/docs` |

---

## Quick start (Docker Compose)

```bash
docker compose up --build
# App available at http://localhost:8080
# Swagger UI at  http://localhost:8080/api/docs
```

## Local development

### Prerequisites

- Rust (stable, >= 1.75)
- PostgreSQL running locally

```bash
# 1. Copy and edit env
cp .env.example .env

# 2. Create the database
createdb taskboard

# 3. Run
cargo run
```

Migrations run automatically on startup.

---

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `DB_HOST` | `localhost` | Postgres host |
| `DB_PORT` | `5432` | Postgres port |
| `DB_USER` | `postgres` | Postgres user |
| `DB_PASSWORD` | `postgres` | Postgres password |
| `DB_NAME` | `taskboard` | Database name |
| `SERVER_HOST` | `0.0.0.0` | Bind address |
| `SERVER_PORT` | `8080` | HTTP port |
| `STATIC_DIR` | `./static` | Path to frontend static files |
| `ENABLE_DOCS` | `true` | Expose Swagger UI at `/api/docs` |
| `DELETE_TAP_COUNT` | `3` | Taps/clicks to delete a box |
| `SEPARATE_ADD_TAB` | `false` | Move Add/Manage controls to a dedicated tab (ideal for touch-only screens) |
| `MAX_BOARDS` | `5` | Maximum number of boards the user can create |
| `DEFAULT_CANVAS_WIDTH` | `1280` | Canvas width for new-box placement |
| `DEFAULT_CANVAS_HEIGHT` | `800` | Canvas height for new-box placement |
| `RUST_LOG` | `info` | Log level |

---

## API overview

All REST endpoints are under `/api`. WebSocket endpoint is `GET /api/ws`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Liveness probe |
| GET | `/api/config` | Client configuration (tap count etc.) |
| GET | `/api/boards` | List all 5 boards |
| GET | `/api/boards/:id` | Get a single board |
| PATCH | `/api/boards/:id` | Update board title / color |
| GET | `/api/boards/:id/boxes` | List boxes on a board |
| POST | `/api/boards/:id/boxes` | Create a box |
| GET | `/api/boxes` | List all boxes (all boards) |
| PATCH | `/api/boxes/:id` | Move / edit a box |
| DELETE | `/api/boxes/:id` | Delete a box |
| GET | `/api/ws` | WebSocket event stream |

WebSocket messages are JSON with `{ "event": "BoxCreated"|"BoxUpdated"|"BoxDeleted"|"BoardUpdated", "data": ... }`.

---

## Frontend features

- **5 boards** — switchable via top tab bar, each with configurable title and accent color
- **8 color palettes** for boxes — light & dark variants
- **Drag & drop** — touch and mouse, boxes scroll the canvas if moved out of bounds
- **Multi-tap delete** — configurable via `DELETE_TAP_COUNT` env var (default triple-tap)
- **Collaborative** — WebSocket push + 1 s polling fallback
- **Help panel** — instructions accessible via the `?` button in the toolbar
- **Dark / light theme** toggle
- **Responsive** — works on small touch screens and wide desktop monitors

---

## Project structure

```
task-board/
├── src/
│   ├── main.rs              # Entry point
│   ├── config.rs            # Env-based configuration
│   ├── api/
│   │   ├── mod.rs           # Router, AppState, broadcast channel
│   │   ├── board_routes.rs  # CRUD for boards
│   │   ├── box_routes.rs    # CRUD for boxes
│   │   ├── config_routes.rs # Client config endpoint
│   │   └── ws.rs            # WebSocket handler
│   ├── boards/
│   │   ├── mod.rs           # Board domain types
│   │   └── db.rs            # Board DB queries
│   └── boxes/
│       ├── mod.rs           # Box domain types
│       └── db.rs            # Box DB queries
├── migrations/
│   ├── 0001_create_boards.up.sql
│   └── 0001_create_boards.down.sql
├── static/
│   └── index.html           # Full SPA frontend
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
└── .env.example
```
