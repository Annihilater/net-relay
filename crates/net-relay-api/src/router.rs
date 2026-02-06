//! API router configuration.

use axum::routing::get;
use axum::Router;
use net_relay_core::Stats;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::handlers;

/// Create the API router.
pub fn create_router(stats: Arc<Stats>, static_dir: Option<PathBuf>) -> Router {
    let api_routes = Router::new()
        .route("/health", get(handlers::health))
        .route("/stats", get(handlers::get_stats))
        .route("/connections", get(handlers::get_connections))
        .route("/history", get(handlers::get_history))
        .with_state(stats);

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
