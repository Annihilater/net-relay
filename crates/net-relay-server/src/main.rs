//! # Net-Relay Server
//!
//! Main entry point for the net-relay proxy server.

use anyhow::{Context, Result};
use net_relay_api::create_router;
use net_relay_core::proxy::{HttpProxy, Socks5Proxy};
use net_relay_core::{Config, ConfigManager, LoggingConfig, Stats};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let (config, config_path) = load_config()?;

    // Initialize logging (must be before any log calls)
    let _guard = init_logging(&config.logging);

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

/// Initialize logging with the specified config.
/// Returns a guard that must be kept alive for the duration of the program
/// when using file logging (to ensure logs are flushed).
fn init_logging(
    logging_config: &LoggingConfig,
) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&logging_config.level));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false);

    // If log file is configured, set up dual output (console + file)
    if let Some(ref log_file) = logging_config.file {
        // Parse the file path to get directory and filename
        let log_path = PathBuf::from(log_file);
        let log_dir = log_path.parent().unwrap_or(std::path::Path::new("."));
        let log_filename = log_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("net-relay.log");

        // Create log directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(log_dir) {
            eprintln!(
                "Warning: Failed to create log directory {:?}: {}",
                log_dir, e
            );
        }

        // Create rolling file appender (daily rotation)
        let file_appender = tracing_appender::rolling::daily(log_dir, log_filename);
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        // File layer (no ANSI colors)
        let file_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(false)
            .with_file(false)
            .with_ansi(false)
            .with_writer(non_blocking);

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(file_layer)
            .init();

        eprintln!("Logging to console and file: {}", log_file);
        Some(guard)
    } else {
        // Console only
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();

        None
    }
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
