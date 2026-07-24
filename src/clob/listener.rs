// src/clob/listener.rs

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::clob::parser::parse_clob_text;
use crate::clob::subscription::ClobSubscription;
use crate::config::ClobConfig;
use crate::types::{Book, TokenId};

pub async fn run_clob_listener_loop<F>(
    config: ClobConfig,
    subscription: Arc<RwLock<ClobSubscription>>,
    mut on_books: F,
) -> Result<()>
where
    F: FnMut(Vec<Book>) + Send,
{
    loop {
        let result =
            run_clob_listener_once(config.clone(), subscription.clone(), &mut on_books).await;

        if let Err(error) = result {
            eprintln!("CLOB listener disconnected/error: {error:#}");
        }

        sleep(Duration::from_millis(config.reconnect_backoff_ms)).await;
    }
}

pub async fn run_clob_listener_once<F>(
    config: ClobConfig,
    subscription: Arc<RwLock<ClobSubscription>>,
    on_books: &mut F,
) -> Result<()>
where
    F: FnMut(Vec<Book>) + Send,
{
    let (mut socket, _) = connect_async(&config.websocket_url)
        .await
        .with_context(|| {
            format!(
                "failed to connect to CLOB websocket {}",
                config.websocket_url
            )
        })?;

    let token_ids = {
        let guard = subscription.read().await;
        guard.token_ids()
    };

    let subscribe_message = build_subscribe_message(&token_ids);

    socket
        .send(Message::Text(subscribe_message.into()))
        .await
        .context("failed to send CLOB subscription message")?;

    while let Some(message) = socket.next().await {
        let message = message.context("failed to read CLOB websocket message")?;

        match message {
            Message::Text(text) => {
                let books = handle_clob_text(text.as_str(), now_ms())?;

                if !books.is_empty() {
                    on_books(books);
                }
            }
            Message::Binary(bytes) => {
                let text = String::from_utf8_lossy(bytes.as_ref());
                let books = handle_clob_text(&text, now_ms())?;

                if !books.is_empty() {
                    on_books(books);
                }
            }
            Message::Ping(payload) => {
                socket.send(Message::Pong(payload)).await?;
            }
            Message::Close(_) => {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

pub fn handle_clob_text(text: &str, received_at_ms: i64) -> Result<Vec<Book>> {
    match parse_clob_text(text, received_at_ms) {
        Ok(books) => Ok(books),
        Err(_) => Ok(Vec::new()),
    }
}

fn build_subscribe_message(token_ids: &[TokenId]) -> String {
    let assets = token_ids
        .iter()
        .map(|token_id| token_id.as_str())
        .collect::<Vec<_>>();

    serde_json::json!({
        "type": "subscribe",
        "assets_ids": assets
    })
    .to_string()
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_millis() as i64
}
