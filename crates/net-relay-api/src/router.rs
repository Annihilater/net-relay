//! API router configuration.

use axum::routing::{delete, get, post};
use axum::Router;
use net_relay_core::{ConfigManager, Stats};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::handlers::{self, AppState};

/// Create the API router.
pub fn create_router(
    stats: Arc<Stats>,
    config_manager: ConfigManager,
    static_dir: Option<PathBuf>,
) -> Router {
    let state = AppState {
        stats,
        config_manager,
    };

    let api_routes = Router::new()
        // Health & Stats
        .route("/health", get(handlers::health))
        .route("/stats", get(handlers::get_stats))
        .route("/connections", get(handlers::get_connections))
        .route("/history", get(handlers::get_history))
        // Configuration
        .route("/config", get(handlers::get_config))
        .route("/config/access-control", get(handlers::get_access_control))
        .route(
            "/config/access-control",
            post(handlers::update_access_control),
        )
        // IP lists
        .route("/config/ip/blacklist", post(handlers::add_ip_blacklist))
        .route(
            "/config/ip/blacklist",
            delete(handlers::remove_ip_blacklist),
        )
        .route("/config/ip/whitelist", post(handlers::add_ip_whitelist))
        .route(
            "/config/ip/whitelist",
            delete(handlers::remove_ip_whitelist),
        )
        // Access rules
        .route("/config/rules", post(handlers::add_rule))
        .route("/config/rules", delete(handlers::remove_rule))
        .with_state(state);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut app = Router::new()
        .nest("/api", api_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Serve static files if directory is provided
    if let Some(dir) = static_dir {
        app = app.fallback_service(ServeDir::new(dir));
    }

    app
}
