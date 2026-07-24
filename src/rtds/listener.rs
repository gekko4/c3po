// src/rtds/listener.rs

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::config::RtdsConfig;
use crate::rtds::message::RtdsMessage;
use crate::rtds::normalizer::normalize_rtds_payload;
use crate::rtds::symbols::is_supported_rtds_symbol;
use crate::tick_store::TickStore;

pub async fn run_rtds_listener_loop(
    config: RtdsConfig,
    tick_store: Arc<RwLock<TickStore>>,
) -> Result<()> {
    loop {
        let result = run_rtds_listener_once(config.clone(), tick_store.clone()).await;

        if let Err(error) = result {
            eprintln!("RTDS listener disconnected/error: {error:#}");
        }

        sleep(Duration::from_millis(config.reconnect_backoff_ms)).await;
    }
}

pub async fn run_rtds_listener_once(
    config: RtdsConfig,
    tick_store: Arc<RwLock<TickStore>>,
) -> Result<()> {
    let (mut socket, _) = connect_async(&config.websocket_url)
        .await
        .with_context(|| {
            format!(
                "failed to connect to RTDS websocket {}",
                config.websocket_url
            )
        })?;

    let subscribe_message = build_subscribe_message(&config.topic);

    socket
        .send(Message::Text(subscribe_message.into()))
        .await
        .with_context(|| format!("failed to subscribe to RTDS topic {}", config.topic))?;

    while let Some(message) = socket.next().await {
        let message = message.context("failed to read RTDS websocket message")?;

        match message {
            Message::Text(text) => {
                handle_rtds_text(text.as_str(), &config.topic, &tick_store).await?;
            }
            Message::Binary(bytes) => {
                let text = String::from_utf8_lossy(bytes.as_ref());
                handle_rtds_text(&text, &config.topic, &tick_store).await?;
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

pub async fn handle_rtds_text(
    text: &str,
    expected_topic: &str,
    tick_store: &Arc<RwLock<TickStore>>,
) -> Result<Option<usize>> {
    let message: RtdsMessage = match serde_json::from_str(text) {
        Ok(message) => message,
        Err(_) => return Ok(None),
    };

    if !message.is_update() || !message.has_topic(expected_topic) {
        return Ok(None);
    }

    let normalized = normalize_rtds_payload(&message.payload)?;

    if !is_supported_rtds_symbol(&normalized.symbol) {
        return Ok(None);
    }

    let tick = normalized.into_tick(now_ms());

    let mut store = tick_store.write().await;
    store.insert_tick(tick);

    Ok(Some(store.len()))
}

fn build_subscribe_message(topic: &str) -> String {
    serde_json::json!({
        "type": "subscribe",
        "topic": topic
    })
    .to_string()
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_millis() as i64
}
