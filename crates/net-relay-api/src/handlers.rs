//! API route handlers.

use axum::extract::State;
use axum::Json;
use net_relay_core::stats::{AggregatedStats, ConnectionStats, Stats, UserStats};
use net_relay_core::{
    AccessControlConfig, AccessRule, Config, ConfigManager, ConnectionInfo, User,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub stats: Arc<Stats>,
    pub config_manager: ConfigManager,
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
