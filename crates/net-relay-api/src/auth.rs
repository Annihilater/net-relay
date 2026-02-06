//! Session-based authentication for the dashboard.

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use net_relay_core::ConfigManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Session store for managing authentication tokens.
#[derive(Clone, Default)]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
}

/// Session data associated with a token.
#[derive(Clone)]
pub struct SessionData {
    pub username: String,
    pub created_at: std::time::Instant,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session and return the token.
    pub async fn create_session(&self, username: String) -> String {
        let token = generate_token();
        let session = SessionData {
            username,
            created_at: std::time::Instant::now(),
        };
        self.sessions.write().await.insert(token.clone(), session);
        token
    }

    /// Validate a session token.
    pub async fn validate(&self, token: &str) -> Option<String> {
        let sessions = self.sessions.read().await;
        sessions.get(token).map(|s| s.username.clone())
    }

    /// Remove a session.
    pub async fn remove(&self, token: &str) {
        self.sessions.write().await.remove(token);
    }
}

/// Generate a secure random token.
fn generate_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let random: u64 = rand_simple();
    format!("{:x}{:016x}", timestamp, random)
}

/// Simple pseudo-random number generator (no external dependency).
fn rand_simple() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    // xorshift64
    let mut x = seed;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

/// Session auth middleware that checks for valid session cookie.
pub async fn session_auth_middleware(
    config_manager: ConfigManager,
    session_store: SessionStore,
    request: Request,
    next: Next,
) -> Response {
    // Check if authentication is enabled
    if !config_manager.is_dashboard_auth_enabled().await {
        return next.run(request).await;
    }

    let path = request.uri().path();

    // Allow public endpoints without auth
    if is_public_path(path) {
        return next.run(request).await;
    }

    // Check for session cookie
    let cookie_header = request
        .headers()
        .get(header::COOKIE)
        .and_then(|h| h.to_str().ok());

    if let Some(cookies) = cookie_header {
        if let Some(token) = extract_session_token(cookies) {
            if session_store.validate(&token).await.is_some() {
                return next.run(request).await;
            }
        }
    }

    unauthorized_response()
}

/// Check if a path is public (doesn't require auth).
fn is_public_path(path: &str) -> bool {
    // Auth endpoints are public
    path == "/api/auth/login"
        || path == "/api/auth/check"
        || path == "/api/auth/logout"
        // Static files are public (login page needs to load)
        || path == "/"
        || path == "/index.html"
        || path.starts_with("/src/")
        || path.ends_with(".css")
        || path.ends_with(".js")
        || path.ends_with(".ico")
        || path.ends_with(".png")
        || path.ends_with(".svg")
}

/// Extract session token from cookie header.
fn extract_session_token(cookies: &str) -> Option<String> {
    for cookie in cookies.split(';') {
        let cookie = cookie.trim();
        if let Some(value) = cookie.strip_prefix("net_relay_session=") {
            return Some(value.to_string());
        }
    }
    None
}

/// Generate a 401 Unauthorized response.
fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(header::CONTENT_TYPE, "application/json")],
        r#"{"success":false,"error":"Authentication required"}"#,
    )
        .into_response()
}
