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

Two ways to run it with Docker Compose:

### Option A — prebuilt image from GHCR (no build, no toolchain)

```bash
docker compose -f docker-compose.image.yml up -d
# App available at http://localhost:8080
# Swagger UI at  http://localhost:8080/api/docs
```

The image is published automatically by CI to
[ghcr.io/majorxaker/task-board](https://github.com/MajorXaker/task-board/pkgs/container/task-board)
on every push to `main` and every `vX.Y.Z` tag. Tags: `latest`, `X.Y.Z`,
`X.Y`, and `sha-<commit>`.

> The first release of the package is private by default. To pull without
> logging in, set it public once: Repo → Packages → task-board → Package
> settings → Change visibility → Public.

### Option B — build from source

```bash
docker compose up --build
# App available at http://localhost:8080
# Swagger UI at  http://localhost:8080/api/docs
```

## Android client

A slim Android app (`android/`) that wraps this same web UI in a WebView —
one config item (the server URL), built and signed by CI.

- **Build:** CI builds a signed release APK on every push (artifact) and
  attaches `taskboard-android-X.Y.Z.apk` to GitHub Releases on `vX.Y.Z` tags.
  First launch asks for the server URL (e.g. `http://192.168.1.50:8080`),
  stored on the device.
- **Offline:** if the server is unreachable, the app shows a local error page
  with *Try again* / *Change server URL* actions instead of a raw WebView error.
- **Plain HTTP** is allowed (`usesCleartextTraffic`) since it targets LAN use.
- **Signing:** a PKCS12 keystore is committed (`android/keystore/`) so CI APKs
  can update over each other. Intentional for a LAN app — do not reuse it for
  Play Store distribution.

Local build (needs JDK 17 + Android SDK):

```bash
cd android
./gradlew assembleRelease
# APK: app/build/outputs/apk/release/app-release.apk
```

## Linux client

A lite Linux app (`linux/`) that wraps this same web UI in a WebKitGTK
window — one config item (the server URL), built by CI on Arch Linux x64
(`archlinux:latest` container, so the binary matches a current Arch system).

- **Build:** CI builds a release tarball on every push (artifact) and
  attaches `taskboard-linux-x64-X.Y.Z.tar.gz` to GitHub Releases on `vX.Y.Z`
  tags. First launch asks for the server URL (e.g. `http://192.168.1.50:8080`)
  and remembers it in `~/.config/taskboard/url`.
- **Config priority:** `--url http://host:port` argument, `TASKBOARD_URL`
  environment variable, config file.
- **Runtime requirements:** webkit2gtk-4.1 and gtk3
  (Arch: `sudo pacman -S webkit2gtk-4.1 gtk3`).
- **Plain HTTP** is allowed — it targets LAN use.

Local build (needs Rust + webkit2gtk-4.1 + gtk3 dev packages):

```bash
cargo build --release --manifest-path linux/Cargo.toml
# Binary: linux/target/release/taskboard
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
| `BOX_COLOR_1` … `BOX_COLOR_32` | built-in 8 | Colors offered for new boxes, `"#rrggbb,#rrggbb"` (bg, text). Indices 1–32, unset ones skipped; if none are set the built-in dark palette is used. Existing boxes keep their colors. |
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
- **Configurable box colors** — palette defined by `BOX_COLOR_*` env vars (built-in 8-color dark palette by default); click a box to pick up its color for the next one, legacy colors included
- **Add to current tab** — new boxes appear at the center of the screen and the toolbar Add button always targets the board tab you are on
- **Drag & drop** — touch and mouse, boxes scroll the canvas if moved out of bounds
- **Multi-tap delete** — configurable via `DELETE_TAP_COUNT` env var (default triple-tap)
- **Collaborative** — WebSocket push + 1 s polling fallback
- **Help panel** — instructions accessible via the `?` button in the toolbar
- **Dark theme** (only)
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
