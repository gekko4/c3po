// src/runtime/shutdown.rs

use anyhow::Result;

pub async fn wait_for_shutdown_signal() -> Result<()> {
    tokio::signal::ctrl_c().await?;
    Ok(())
}
