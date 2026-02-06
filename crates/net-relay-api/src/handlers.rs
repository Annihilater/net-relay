//! API route handlers.

use axum::extract::State;
use axum::Json;
use net_relay_core::stats::{AggregatedStats, ConnectionStats, Stats};
use net_relay_core::{AccessControlConfig, AccessRule, Config, ConfigManager, ConnectionInfo};
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
