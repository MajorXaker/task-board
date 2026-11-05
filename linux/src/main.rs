//! Lite TaskBoard client for Linux: the server's web UI inside a WebKitGTK
//! window.
//!
//! Single configuration item — the server URL:
//!   * `--url http://192.168.1.50:8080` argument, or
//!   * `TASKBOARD_URL` environment variable, or
//!   * `~/.config/taskboard/url` (written automatically when you connect
//!     through the first-run window).
//!
//! On first run (no URL configured) a small dark setup window asks for the
//! URL, remembers it, and navigates there. Plain HTTP is allowed — this is
//! a LAN client.

use std::path::PathBuf;

use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    platform::unix::WindowExtUnix,
    window::WindowBuilder,
};
use wry::{WebViewBuilder, WebViewBuilderExtUnix};

/// Event sent from the webview IPC handler back to the event loop.
enum UserEvent {
    Connect(String),
}

fn main() {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("TaskBoard")
        .with_inner_size(LogicalSize::new(1280.0, 800.0))
        .build(&event_loop)
        .expect("failed to open window");

    // First-run setup page posts the chosen URL via window.ipc.postMessage().
    let ipc_proxy = proxy.clone();
    let ipc_handler = move |req: wry::http::Request<String>| {
        let url = req.body().trim().to_string();
        if url.starts_with("http://") || url.starts_with("https://") {
            let _ = ipc_proxy.send_event(UserEvent::Connect(url));
        }
    };

    // Embed the webview in the window's GTK container instead of the raw
    // window handle — the raw-handle path only works on X11, this works
    // on Wayland and X11 alike.
    let vbox = window
        .default_vbox()
        .expect("window has no default gtk box");

    let webview = match configured_url() {
        Some(url) => WebViewBuilder::new()
            .with_url(&url)
            .with_ipc_handler(ipc_handler)
            .build_gtk(vbox)
            .expect("failed to create webview"),
        None => WebViewBuilder::new()
            .with_html(CONFIG_PAGE)
            .with_ipc_handler(ipc_handler)
            .build_gtk(vbox)
            .expect("failed to create webview"),
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::Connect(url)) => {
                if save_url(&url).is_ok() {
                    let _ = webview.load_url(&url);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

// ── Configuration (single item: server URL) ─────────────────────────────────

/// URL priority: `--url` argument, `TASKBOARD_URL` env var, config file.
fn configured_url() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    for (i, a) in args.iter().enumerate() {
        if a == "--url" {
            if let Some(v) = args.get(i + 1) {
                return Some(v.clone());
            }
        }
    }
    if let Ok(u) = std::env::var("TASKBOARD_URL") {
        let u = u.trim().to_string();
        if u.starts_with("http") {
            return Some(u);
        }
    }
    let path = config_path()?;
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| s.starts_with("http://") || s.starts_with("https://"))
}

fn config_path() -> Option<PathBuf> {
    let base = match std::env::var("XDG_CONFIG_HOME") {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => {
            let home = std::env::var("HOME").ok()?;
            if home.is_empty() {
                return None;
            }
            PathBuf::from(home).join(".config")
        }
    };
    Some(base.join("taskboard").join("url"))
}

fn save_url(url: &str) -> std::io::Result<()> {
    let path = config_path()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "no home directory"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, url)
}

// ── First-run setup page (shown until a URL is configured) ─────────────────

const CONFIG_PAGE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>TaskBoard — setup</title>
<style>
  :root { color-scheme: dark; }
  * { box-sizing: border-box; }
  html, body { height: 100%; margin: 0; }
  body {
    display: flex; align-items: center; justify-content: center;
    background: #101418; color: #e6e9ef;
    font-family: system-ui, sans-serif;
  }
  .card { max-width: 420px; text-align: center; padding: 32px; }
  h1 { font-size: 24px; margin: 0 0 8px; }
  p { color: #9aa3b2; font-size: 15px; line-height: 1.5; margin: 0 0 24px; }
  input {
    width: 100%; padding: 12px 14px; font-size: 16px;
    border: 1px solid #393836; border-radius: 12px;
    background: #201f1d; color: #e6e9ef; outline: none;
  }
  input:focus { border-color: #4f98a3; }
  button {
    width: 100%; margin-top: 16px; padding: 12px;
    font-size: 16px; border: none; border-radius: 12px;
    background: #3b82f6; color: #fff; cursor: pointer;
  }
</style>
</head>
<body>
  <div class="card">
    <h1>TaskBoard</h1>
    <p>Enter the URL of your TaskBoard server.<br>It looks like http://192.168.1.50:8080</p>
    <input id="url" type="text" placeholder="http://192.168.1.50:8080" autofocus>
    <button onclick="connect()">Connect</button>
  </div>
<script>
  function connect() {
    let u = document.getElementById('url').value.trim();
    if (!u) return;
    if (!/^https?:\/\//.test(u)) u = 'http://' + u;
    window.ipc.postMessage(u);
  }
  document.getElementById('url').addEventListener('keydown', e => {
    if (e.key === 'Enter') connect();
  });
</script>
</body>
</html>
"#;
