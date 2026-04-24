//! WebSocket upgrade handler.
//!
//! Clients connect to `GET /api/ws`.  The server sends `StateChange` JSON
//! messages whenever a box or board is mutated.  Clients are read-only on
//! this channel; mutations go through the REST endpoints.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use tracing::{info, warn};

use crate::api::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();
    let (mut sender, mut receiver) = socket.split();

    info!("WebSocket client connected");

    // Forward broadcast events to the client.
    let send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(change) => {
                    let json = match serde_json::to_string(&change) {
                        Ok(j) => j,
                        Err(e) => {
                            warn!(error = %e, "Failed to serialize StateChange");
                            continue;
                        }
                    };
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break; // client disconnected
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    warn!(skipped = n, "WS broadcast lagged — client missed events");
                }
                Err(_) => break,
            }
        }
    });

    // Drain incoming frames (we ignore them but keep the connection alive).
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(_)) = receiver.next().await {}
    });

    // When either task ends, abort the other.
    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }

    info!("WebSocket client disconnected");
}