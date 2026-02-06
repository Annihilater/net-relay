//! API route handlers.

use axum::extract::State;
use axum::Json;
use net_relay_core::stats::{AggregatedStats, ConnectionStats, Stats};
use net_relay_core::ConnectionInfo;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// API response wrapper.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            data,
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
pub async fn get_stats(State(stats): State<Arc<Stats>>) -> Json<ApiResponse<StatsResponse>> {
    let aggregated = stats.get_aggregated().await;
    let active_connections = stats.get_active().await;
    
    ApiResponse::ok(StatsResponse {
        aggregated,
        active_connections,
    })
}

/// Get active connections.
pub async fn get_connections(State(stats): State<Arc<Stats>>) -> Json<ApiResponse<Vec<ConnectionInfo>>> {
    let connections = stats.get_active().await;
    ApiResponse::ok(connections)
}

/// Get connection history.
pub async fn get_history(
    State(stats): State<Arc<Stats>>,
    axum::extract::Query(query): axum::extract::Query<HistoryQuery>,
) -> Json<ApiResponse<Vec<ConnectionStats>>> {
    let history = stats.get_history(query.limit).await;
    ApiResponse::ok(history)
}
