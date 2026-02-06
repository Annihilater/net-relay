//! # Net-Relay Server
//!
//! Main entry point for the net-relay proxy server.

use anyhow::{Context, Result};
use net_relay_api::create_router;
use net_relay_core::proxy::{HttpProxy, Socks5Proxy};
use net_relay_core::{Config, ConfigManager, Stats};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let (config, config_path) = load_config()?;

    // Initialize logging
    init_logging(&config.logging.level);

    info!(
        "Starting net-relay proxy server v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Create config manager for runtime configuration
    let config_manager = ConfigManager::new(config.clone(), config_path);

    // Create shared stats
    let stats = Arc::new(Stats::new(1000));

    // Prepare authentication
    let auth = if config.security.auth_enabled {
        match (&config.security.username, &config.security.password) {
            (Some(u), Some(p)) => Some((u.clone(), p.clone())),
            _ => {
                error!("Authentication enabled but username/password not configured");
                return Err(anyhow::anyhow!("Invalid authentication configuration"));
            }
        }
    } else {
        None
    };

    // Start SOCKS5 proxy
    let socks_addr: SocketAddr = format!("{}:{}", config.server.host, config.server.socks_port)
        .parse()
        .context("Invalid SOCKS5 bind address")?;
    let socks_proxy = Socks5Proxy::new(
        socks_addr,
        auth.clone(),
        Arc::clone(&stats),
        config_manager.clone(),
    );

    let socks_handle = tokio::spawn(async move {
        if let Err(e) = socks_proxy.run().await {
            error!("SOCKS5 proxy error: {}", e);
        }
    });

    // Start HTTP CONNECT proxy
    let http_addr: SocketAddr = format!("{}:{}", config.server.host, config.server.http_port)
        .parse()
        .context("Invalid HTTP bind address")?;
    let http_proxy = HttpProxy::new(http_addr, auth, Arc::clone(&stats), config_manager.clone());

    let http_handle = tokio::spawn(async move {
        if let Err(e) = http_proxy.run().await {
            error!("HTTP proxy error: {}", e);
        }
    });

    // Start API server
    let api_addr: SocketAddr = format!("{}:{}", config.server.host, config.server.api_port)
        .parse()
        .context("Invalid API bind address")?;

    let static_dir = find_static_dir();
    let router = create_router(Arc::clone(&stats), config_manager, static_dir);

    let api_handle = tokio::spawn(async move {
        info!("API server listening on http://{}", api_addr);
        let listener = tokio::net::TcpListener::bind(api_addr).await.unwrap();
        if let Err(e) = axum::serve(listener, router).await {
            error!("API server error: {}", e);
        }
    });

    info!("Net-relay is running:");
    info!("  SOCKS5 proxy: {}", socks_addr);
    info!("  HTTP proxy:   {}", http_addr);
    info!("  Dashboard:    http://{}", api_addr);

    // Wait for all services
    tokio::select! {
        _ = socks_handle => error!("SOCKS5 proxy stopped"),
        _ = http_handle => error!("HTTP proxy stopped"),
        _ = api_handle => error!("API server stopped"),
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
    }

    info!("Net-relay shutting down");
    Ok(())
}

/// Load configuration from file or use defaults.
/// Returns (Config, Option<config_path>)
fn load_config() -> Result<(Config, Option<String>)> {
    let config_paths = ["config.toml", "/etc/net-relay/config.toml"];

    for path in config_paths {
        if std::path::Path::new(path).exists() {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read config file: {}", path))?;
            let config: Config = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", path))?;
            info!("Loaded configuration from {}", path);
            return Ok((config, Some(path.to_string())));
        }
    }

    info!("No config file found, using defaults");
    Ok((Config::default(), None))
}

/// Initialize logging with the specified level.
fn init_logging(level: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .init();
}

/// Find the static files directory for the frontend.
fn find_static_dir() -> Option<PathBuf> {
    let paths = [
        "frontend",
        "../frontend",
        "../../frontend",
        "/usr/share/net-relay/frontend",
    ];

    for path in paths {
        let p = PathBuf::from(path);
        if p.exists() && p.is_dir() {
            info!("Serving static files from {:?}", p);
            return Some(p);
        }
    }

    None
}
