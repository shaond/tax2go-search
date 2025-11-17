use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::path::PathBuf;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    /// Address to bind the HTTP server to
    pub bind_addr: SocketAddr,

    /// Base directory for storing per-user indexes
    pub data_dir: PathBuf,

    /// Log level for tracing
    pub log_level: String,

    /// Enable web UI for testing (binds on localhost only)
    pub web_ui_enabled: bool,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// Expected environment variables:
    /// - `BIND_ADDR`: Socket address (default: "127.0.0.1:8080")
    /// - `DATA_DIR`: Base directory for indexes (required)
    /// - `LOG_LEVEL`: Logging level (default: "info")
    /// - `WEB_UI_ENABLED`: Enable web UI (default: "false")
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists (development only)
        let _ = dotenvy::dotenv();

        let bind_addr = std::env::var("BIND_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
            .parse()
            .context("Failed to parse BIND_ADDR as a valid socket address")?;

        let data_dir = std::env::var("DATA_DIR")
            .context("DATA_DIR environment variable is required")?
            .into();

        let log_level = std::env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());

        let web_ui_enabled = std::env::var("WEB_UI_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";

        Ok(Config {
            bind_addr,
            data_dir,
            log_level,
            web_ui_enabled,
        })
    }

    /// Validate configuration and create necessary directories
    pub fn validate(&self) -> Result<()> {
        // Create data directory if it doesn't exist
        std::fs::create_dir_all(&self.data_dir)
            .with_context(|| format!("Failed to create data directory: {:?}", self.data_dir))?;

        // Verify we can write to the data directory
        let test_file = self.data_dir.join(".write_test");
        std::fs::write(&test_file, b"test")
            .with_context(|| format!("Data directory is not writable: {:?}", self.data_dir))?;
        std::fs::remove_file(&test_file)
            .context("Failed to clean up write test file")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = Config {
            bind_addr: "127.0.0.1:8080".parse().unwrap(),
            data_dir: temp_dir.path().to_path_buf(),
            log_level: "info".to_string(),
            web_ui_enabled: false,
        };

        assert!(config.validate().is_ok());
    }
}
