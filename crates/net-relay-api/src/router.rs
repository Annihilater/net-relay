//! API router configuration.

use axum::body::Body;
use axum::http::{header, HeaderValue, Request, StatusCode};
use axum::middleware;
use axum::response::Response;
use axum::routing::{delete, get, post, put};
use axum::Router;
use net_relay_core::{ConfigManager, Stats};
use rust_embed::Embed;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::auth::{session_auth_middleware, SessionStore};
use crate::handlers::{self, AppState};

/// Embedded frontend assets - compiled into the binary
#[derive(Embed)]
#[folder = "../../frontend/"]
struct FrontendAssets;

/// Handler for serving embedded static files
async fn serve_embedded(req: Request<Body>) -> Response {
    let path = req.uri().path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match FrontendAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(mime.as_ref()).unwrap(),
                )
                .body(Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => {
            // For SPA: return index.html for unknown paths (client-side routing)
            if !path.contains('.') {
                if let Some(index) = FrontendAssets::get("index.html") {
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, HeaderValue::from_static("text/html"))
                        .body(Body::from(index.data.into_owned()))
                        .unwrap();
                }
            }
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap()
        }
    }
}

/// Create the API router.
pub fn create_router(
    stats: Arc<Stats>,
    config_manager: ConfigManager,
    static_dir: Option<PathBuf>,
) -> Router {
    let session_store = SessionStore::new();

    let state = AppState {
        stats,
        config_manager: config_manager.clone(),
        session_store: session_store.clone(),
    };

    // Auth routes (public, no auth required)
    let auth_routes = Router::new()
        .route("/auth/check", get(handlers::auth_check))
        .route("/auth/login", post(handlers::login))
        .route("/auth/logout", post(handlers::logout))
        .with_state(state.clone());

    // Protected API routes
    let api_routes = Router::new()
        // Health & Stats
        .route("/health", get(handlers::health))
        .route("/stats", get(handlers::get_stats))
        .route("/connections", get(handlers::get_connections))
        .route("/history", get(handlers::get_history))
        .route("/stats/users", get(handlers::get_user_stats))
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
        // Security & Users
        .route("/config/security", get(handlers::get_security))
        .route("/config/security", put(handlers::update_security))
        .route("/config/users", post(handlers::add_user))
        .route("/config/users", put(handlers::update_user))
        .route("/config/users", delete(handlers::remove_user))
        // Server configuration
        .route("/config/server", get(handlers::get_server_config))
        .route("/config/server", put(handlers::update_server_config))
        .with_state(state);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create session auth middleware layer
    let auth_config_manager = config_manager.clone();
    let auth_session_store = session_store.clone();
    let auth_layer = middleware::from_fn(move |req, next| {
        let cm = auth_config_manager.clone();
        let ss = auth_session_store.clone();
        async move { session_auth_middleware(cm, ss, req, next).await }
    });

    let mut app = Router::new()
        .nest("/api", auth_routes.merge(api_routes))
        .layer(auth_layer)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Serve static files: prefer external directory if exists, otherwise use embedded
    if let Some(dir) = static_dir {
        tracing::info!("Serving static files from external directory: {:?}", dir);
        app = app.fallback_service(ServeDir::new(dir));
    } else {
        tracing::info!("Serving embedded static files (frontend built into binary)");
        app = app.fallback(serve_embedded);
    }

    app
}
