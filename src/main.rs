mod config;
mod http;
mod search;

use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::Config;
use crate::http::build_router;
use crate::http::routes::AppState;
use crate::search::IndexManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::from_env().context("Failed to load configuration")?;

    // Initialize tracing/logging
    init_tracing(&config.log_level)?;

    info!("Starting tax2go-search service");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));
    info!("Bind address: {}", config.bind_addr);
    info!("Data directory: {:?}", config.data_dir);

    // Validate configuration
    config.validate().context("Configuration validation failed")?;

    // Initialize index manager
    let index_manager = Arc::new(IndexManager::new(config.data_dir.clone()));
    info!("Index manager initialized");

    // Build application state
    let state = AppState { index_manager };

    // Build router
    let app = build_router(state);

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .with_context(|| format!("Failed to bind to {}", config.bind_addr))?;

    info!("Server listening on {}", config.bind_addr);
    info!("Health check available at http://{}/health", config.bind_addr);
    info!("API endpoints available at http://{}/v1/*", config.bind_addr);

    // Start server
    axum::serve(listener, app)
        .await
        .context("Server error")?;

    Ok(())
}

/// Initialize tracing subscriber for logging
fn init_tracing(log_level: &str) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize tracing: {}", e))?;

    Ok(())
}
