//! API route handlers.

use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::HeaderMap;
use axum::Json;
use net_relay_core::stats::{AggregatedStats, ConnectionStats, Stats, UserStats};
use net_relay_core::{
    AccessControlConfig, AccessRule, Config, ConfigManager, ConnectionInfo, ServerConfig, User,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::auth::SessionStore;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub stats: Arc<Stats>,
    pub config_manager: ConfigManager,
    pub session_store: SessionStore,
}

/// API response wrapper.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            data,
            message: None,
        })
    }
}

/// Error response helper.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>) -> Json<Self> {
        Json(Self {
            success: false,
            error: error.into(),
        })
    }
}

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Stats response.
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub aggregated: AggregatedStats,
    pub active_connections: Vec<ConnectionInfo>,
}

/// History query parameters.
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<usize>,
}

/// Health check endpoint.
pub async fn health() -> Json<ApiResponse<HealthResponse>> {
    ApiResponse::ok(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Get server statistics.
pub async fn get_stats(State(state): State<AppState>) -> Json<ApiResponse<StatsResponse>> {
    let aggregated = state.stats.get_aggregated().await;
    let active_connections = state.stats.get_active().await;

    ApiResponse::ok(StatsResponse {
        aggregated,
        active_connections,
    })
}

/// Get active connections.
pub async fn get_connections(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<ConnectionInfo>>> {
    let connections = state.stats.get_active().await;
    ApiResponse::ok(connections)
}

/// Get connection history.
pub async fn get_history(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<HistoryQuery>,
) -> Json<ApiResponse<Vec<ConnectionStats>>> {
    let history = state.stats.get_history(query.limit).await;
    ApiResponse::ok(history)
}

// ==================== Configuration API ====================

/// Get current configuration.
pub async fn get_config(State(state): State<AppState>) -> Json<ApiResponse<Config>> {
    let config = state.config_manager.get().await;
    ApiResponse::ok(config)
}

/// Get access control configuration only.
pub async fn get_access_control(
    State(state): State<AppState>,
) -> Json<ApiResponse<AccessControlConfig>> {
    let config = state.config_manager.get().await;
    ApiResponse::ok(config.access_control)
}

/// Update access control configuration.
pub async fn update_access_control(
    State(state): State<AppState>,
    Json(access_control): Json<AccessControlConfig>,
) -> Json<ApiResponse<AccessControlConfig>> {
    match state
        .config_manager
        .update_access_control(access_control.clone())
        .await
    {
        Ok(_) => ApiResponse::ok(access_control),
        Err(e) => Json(ApiResponse {
            success: false,
            data: access_control,
            message: Some(format!("Failed to save: {}", e)),
        }),
    }
}

/// Add IP to blacklist.
#[derive(Debug, Deserialize)]
pub struct IpListRequest {
    pub ip: String,
}

pub async fn add_ip_blacklist(
    State(state): State<AppState>,
    Json(req): Json<IpListRequest>,
) -> Json<ApiResponse<AccessControlConfig>> {
    let mut config = state.config_manager.get().await;
    if !config.access_control.ip_blacklist.contains(&req.ip) {
        config.access_control.ip_blacklist.push(req.ip);
    }
    let _ = state
        .config_manager
        .update_access_control(config.access_control.clone())
        .await;
    ApiResponse::ok(config.access_control)
}

pub async fn remove_ip_blacklist(
    State(state): State<AppState>,
    Json(req): Json<IpListRequest>,
) -> Json<ApiResponse<AccessControlConfig>> {
    let mut config = state.config_manager.get().await;
    config
        .access_control
        .ip_blacklist
        .retain(|ip| ip != &req.ip);
    let _ = state
        .config_manager
        .update_access_control(config.access_control.clone())
        .await;
    ApiResponse::ok(config.access_control)
}

pub async fn add_ip_whitelist(
    State(state): State<AppState>,
    Json(req): Json<IpListRequest>,
) -> Json<ApiResponse<AccessControlConfig>> {
    let mut config = state.config_manager.get().await;
    if !config.access_control.ip_whitelist.contains(&req.ip) {
        config.access_control.ip_whitelist.push(req.ip);
    }
    let _ = state
        .config_manager
        .update_access_control(config.access_control.clone())
        .await;
    ApiResponse::ok(config.access_control)
}

pub async fn remove_ip_whitelist(
    State(state): State<AppState>,
    Json(req): Json<IpListRequest>,
) -> Json<ApiResponse<AccessControlConfig>> {
    let mut config = state.config_manager.get().await;
    config
        .access_control
        .ip_whitelist
        .retain(|ip| ip != &req.ip);
    let _ = state
        .config_manager
        .update_access_control(config.access_control.clone())
        .await;
    ApiResponse::ok(config.access_control)
}

/// Add access rule.
pub async fn add_rule(
    State(state): State<AppState>,
    Json(rule): Json<AccessRule>,
) -> Json<ApiResponse<AccessControlConfig>> {
    let mut config = state.config_manager.get().await;
    config.access_control.rules.push(rule);
    let _ = state
        .config_manager
        .update_access_control(config.access_control.clone())
        .await;
    ApiResponse::ok(config.access_control)
}

/// Remove access rule by index.
#[derive(Debug, Deserialize)]
pub struct RemoveRuleRequest {
    pub index: usize,
}

pub async fn remove_rule(
    State(state): State<AppState>,
    Json(req): Json<RemoveRuleRequest>,
) -> Json<ApiResponse<AccessControlConfig>> {
    let mut config = state.config_manager.get().await;
    if req.index < config.access_control.rules.len() {
        config.access_control.rules.remove(req.index);
    }
    let _ = state
        .config_manager
        .update_access_control(config.access_control.clone())
        .await;
    ApiResponse::ok(config.access_control)
}

// ==================== Security & User Management API ====================

/// Security configuration response (without exposing passwords).
#[derive(Debug, Serialize)]
pub struct SecurityResponse {
    pub auth_enabled: bool,
    pub users: Vec<UserInfo>,
    pub user_count: usize,
}

/// User info without password.
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub username: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub bandwidth_limit: u64,
    pub connection_limit: u32,
}

impl From<&User> for UserInfo {
    fn from(user: &User) -> Self {
        Self {
            username: user.username.clone(),
            enabled: user.enabled,
            description: user.description.clone(),
            bandwidth_limit: user.bandwidth_limit,
            connection_limit: user.connection_limit,
        }
    }
}

/// Get security configuration (without passwords).
pub async fn get_security(State(state): State<AppState>) -> Json<ApiResponse<SecurityResponse>> {
    let security = state.config_manager.get_security().await;
    let users: Vec<UserInfo> = security.users.iter().map(UserInfo::from).collect();
    ApiResponse::ok(SecurityResponse {
        auth_enabled: security.auth_enabled,
        user_count: users.len(),
        users,
    })
}

/// Update security settings (enable/disable auth).
#[derive(Debug, Deserialize)]
pub struct UpdateSecurityRequest {
    pub auth_enabled: Option<bool>,
}

pub async fn update_security(
    State(state): State<AppState>,
    Json(req): Json<UpdateSecurityRequest>,
) -> Json<ApiResponse<SecurityResponse>> {
    let mut security = state.config_manager.get_security().await;

    if let Some(enabled) = req.auth_enabled {
        security.auth_enabled = enabled;
    }

    let _ = state.config_manager.update_security(security.clone()).await;

    let users: Vec<UserInfo> = security.users.iter().map(UserInfo::from).collect();
    ApiResponse::ok(SecurityResponse {
        auth_enabled: security.auth_enabled,
        user_count: users.len(),
        users,
    })
}

/// Add user request.
#[derive(Debug, Deserialize)]
pub struct AddUserRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

/// Add a new user.
pub async fn add_user(
    State(state): State<AppState>,
    Json(req): Json<AddUserRequest>,
) -> Json<ApiResponse<SecurityResponse>> {
    let mut security = state.config_manager.get_security().await;

    let user = User {
        username: req.username,
        password: req.password,
        enabled: req.enabled.unwrap_or(true),
        description: req.description,
        bandwidth_limit: 0,
        connection_limit: 0,
    };

    if !security.add_user(user) {
        return Json(ApiResponse {
            success: false,
            data: SecurityResponse {
                auth_enabled: security.auth_enabled,
                user_count: security.users.len(),
                users: security.users.iter().map(UserInfo::from).collect(),
            },
            message: Some("User already exists".to_string()),
        });
    }

    let _ = state.config_manager.update_security(security.clone()).await;

    let users: Vec<UserInfo> = security.users.iter().map(UserInfo::from).collect();
    ApiResponse::ok(SecurityResponse {
        auth_enabled: security.auth_enabled,
        user_count: users.len(),
        users,
    })
}

/// Update user request.
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Update an existing user.
pub async fn update_user(
    State(state): State<AppState>,
    Json(req): Json<UpdateUserRequest>,
) -> Json<ApiResponse<SecurityResponse>> {
    let mut security = state.config_manager.get_security().await;

    if let Some(existing) = security
        .users
        .iter_mut()
        .find(|u| u.username == req.username)
    {
        if let Some(pwd) = req.password {
            existing.password = pwd;
        }
        if let Some(enabled) = req.enabled {
            existing.enabled = enabled;
        }
        if let Some(desc) = req.description {
            existing.description = Some(desc);
        }

        let _ = state.config_manager.update_security(security.clone()).await;
    }

    let users: Vec<UserInfo> = security.users.iter().map(UserInfo::from).collect();
    ApiResponse::ok(SecurityResponse {
        auth_enabled: security.auth_enabled,
        user_count: users.len(),
        users,
    })
}

/// Remove user request.
#[derive(Debug, Deserialize)]
pub struct RemoveUserRequest {
    pub username: String,
}

/// Remove a user.
pub async fn remove_user(
    State(state): State<AppState>,
    Json(req): Json<RemoveUserRequest>,
) -> Json<ApiResponse<SecurityResponse>> {
    let mut security = state.config_manager.get_security().await;

    security.remove_user(&req.username);

    let _ = state.config_manager.update_security(security.clone()).await;

    let users: Vec<UserInfo> = security.users.iter().map(UserInfo::from).collect();
    ApiResponse::ok(SecurityResponse {
        auth_enabled: security.auth_enabled,
        user_count: users.len(),
        users,
    })
}

/// Get per-user statistics.
pub async fn get_user_stats(State(state): State<AppState>) -> Json<ApiResponse<Vec<UserStats>>> {
    let user_stats = state.stats.get_user_stats().await;
    ApiResponse::ok(user_stats)
}

// ==================== Authentication API ====================

/// Login request.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response.
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub authenticated: bool,
    pub username: Option<String>,
}

/// Auth check response.
#[derive(Debug, Serialize)]
pub struct AuthCheckResponse {
    pub auth_enabled: bool,
    pub authenticated: bool,
    pub username: Option<String>,
}

/// Check authentication status.
pub async fn auth_check(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<AuthCheckResponse>> {
    let auth_enabled = state.config_manager.is_dashboard_auth_enabled().await;

    if !auth_enabled {
        return ApiResponse::ok(AuthCheckResponse {
            auth_enabled: false,
            authenticated: true,
            username: None,
        });
    }

    // Check for session cookie and validate
    let cookie_header = headers
        .get(axum::http::header::COOKIE)
        .and_then(|h| h.to_str().ok());

    let username = match cookie_header {
        Some(cookies) => match extract_session_token(cookies) {
            Some(token) => state.session_store.validate(&token).await,
            None => None,
        },
        None => None,
    };

    let authenticated = username.is_some();

    ApiResponse::ok(AuthCheckResponse {
        auth_enabled,
        authenticated,
        username,
    })
}

/// Login handler.
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> (HeaderMap, Json<ApiResponse<LoginResponse>>) {
    let mut headers = HeaderMap::new();

    // Check credentials
    if state
        .config_manager
        .authenticate_dashboard(&req.username, &req.password)
        .await
    {
        // Create session
        let token = state
            .session_store
            .create_session(req.username.clone())
            .await;

        // Set cookie
        let cookie = format!(
            "net_relay_session={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=86400",
            token
        );
        headers.insert(SET_COOKIE, cookie.parse().unwrap());

        (
            headers,
            ApiResponse::ok(LoginResponse {
                authenticated: true,
                username: Some(req.username),
            }),
        )
    } else {
        (
            headers,
            Json(ApiResponse {
                success: false,
                data: LoginResponse {
                    authenticated: false,
                    username: None,
                },
                message: Some("Invalid username or password".to_string()),
            }),
        )
    }
}

/// Logout handler.
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> (HeaderMap, Json<ApiResponse<bool>>) {
    let mut response_headers = HeaderMap::new();

    // Get and remove session
    if let Some(cookies) = headers
        .get(axum::http::header::COOKIE)
        .and_then(|h| h.to_str().ok())
    {
        if let Some(token) = extract_session_token(cookies) {
            state.session_store.remove(&token).await;
        }
    }

    // Clear cookie
    let cookie = "net_relay_session=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0";
    response_headers.insert(SET_COOKIE, cookie.parse().unwrap());

    (response_headers, ApiResponse::ok(true))
}

/// Extract session token from cookie header.
fn extract_session_token(cookies: &str) -> Option<String> {
    for cookie in cookies.split(';') {
        let cookie = cookie.trim();
        if let Some(value) = cookie.strip_prefix("net_relay_session=") {
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

// ==================== Server Configuration API ====================

/// Server configuration response.
#[derive(Debug, Serialize)]
pub struct ServerConfigResponse {
    pub host: String,
    pub socks_port: u16,
    pub http_port: u16,
    pub api_port: u16,
    pub requires_restart: bool,
}

impl From<ServerConfig> for ServerConfigResponse {
    fn from(config: ServerConfig) -> Self {
        Self {
            host: config.host,
            socks_port: config.socks_port,
            http_port: config.http_port,
            api_port: config.api_port,
            requires_restart: false,
        }
    }
}

/// Get server configuration.
pub async fn get_server_config(
    State(state): State<AppState>,
) -> Json<ApiResponse<ServerConfigResponse>> {
    let server = state.config_manager.get_server().await;
    ApiResponse::ok(ServerConfigResponse::from(server))
}

/// Update server configuration request.
#[derive(Debug, Deserialize)]
pub struct UpdateServerRequest {
    pub host: Option<String>,
    pub socks_port: Option<u16>,
    pub http_port: Option<u16>,
    pub api_port: Option<u16>,
}

/// Update server configuration.
pub async fn update_server_config(
    State(state): State<AppState>,
    Json(req): Json<UpdateServerRequest>,
) -> Json<ApiResponse<ServerConfigResponse>> {
    let mut server = state.config_manager.get_server().await;

    if let Some(host) = req.host {
        server.host = host;
    }
    if let Some(port) = req.socks_port {
        server.socks_port = port;
    }
    if let Some(port) = req.http_port {
        server.http_port = port;
    }
    if let Some(port) = req.api_port {
        server.api_port = port;
    }

    match state.config_manager.update_server(server.clone()).await {
        Ok(_) => {
            let mut response = ServerConfigResponse::from(server);
            response.requires_restart = true;
            ApiResponse::ok(response)
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: ServerConfigResponse::from(server),
            message: Some(format!("Failed to save: {}", e)),
        }),
    }
}
