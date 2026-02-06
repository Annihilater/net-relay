//! Statistics collection and aggregation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::connection::ConnectionInfo;

/// Statistics for a single connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    /// Connection info.
    #[serde(flatten)]
    pub info: ConnectionInfo,
}

/// Per-user statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserStats {
    /// Username.
    pub username: String,

    /// Total connections by this user.
    pub total_connections: u64,

    /// Currently active connections.
    pub active_connections: u64,

    /// Total bytes sent.
    pub total_bytes_sent: u64,

    /// Total bytes received.
    pub total_bytes_received: u64,

    /// Last activity time.
    pub last_activity: Option<DateTime<Utc>>,
}

/// Aggregated statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStats {
    /// Total connections since start.
    pub total_connections: u64,

    /// Currently active connections.
    pub active_connections: u64,

    /// Total bytes sent.
    pub total_bytes_sent: u64,

    /// Total bytes received.
    pub total_bytes_received: u64,

    /// Server uptime in seconds.
    pub uptime_secs: i64,

    /// Server start time.
    pub started_at: DateTime<Utc>,

    /// Per-user statistics.
    #[serde(default)]
    pub users: Vec<UserStats>,
}

/// Thread-safe statistics collector.
#[derive(Debug)]
pub struct Stats {
    /// Total connections counter.
    total_connections: AtomicU64,

    /// Total bytes sent.
    total_bytes_sent: AtomicU64,

    /// Total bytes received.
    total_bytes_received: AtomicU64,

    /// Server start time.
    started_at: DateTime<Utc>,

    /// Recent connection history.
    history: Arc<RwLock<VecDeque<ConnectionStats>>>,

    /// Active connections.
    active: Arc<RwLock<Vec<ConnectionInfo>>>,

    /// Per-user statistics.
    user_stats: Arc<RwLock<HashMap<String, UserStats>>>,

    /// Maximum history size.
    max_history: usize,
}

impl Stats {
    /// Create a new statistics collector.
    pub fn new(max_history: usize) -> Self {
        Self {
            total_connections: AtomicU64::new(0),
            total_bytes_sent: AtomicU64::new(0),
            total_bytes_received: AtomicU64::new(0),
            started_at: Utc::now(),
            history: Arc::new(RwLock::new(VecDeque::with_capacity(max_history))),
            active: Arc::new(RwLock::new(Vec::new())),
            user_stats: Arc::new(RwLock::new(HashMap::new())),
            max_history,
        }
    }

    /// Record a new connection.
    pub async fn add_connection(&self, info: ConnectionInfo) {
        self.total_connections.fetch_add(1, Ordering::Relaxed);

        // Update per-user stats
        if let Some(ref username) = info.username {
            let mut user_stats = self.user_stats.write().await;
            let stats = user_stats
                .entry(username.clone())
                .or_insert_with(|| UserStats {
                    username: username.clone(),
                    ..Default::default()
                });
            stats.total_connections += 1;
            stats.active_connections += 1;
            stats.last_activity = Some(Utc::now());
        }

        self.active.write().await.push(info);
    }

    /// Update connection bytes.
    pub fn add_bytes(&self, sent: u64, received: u64) {
        self.total_bytes_sent.fetch_add(sent, Ordering::Relaxed);
        self.total_bytes_received
            .fetch_add(received, Ordering::Relaxed);
    }

    /// Mark a connection as closed and move to history.
    pub async fn close_connection(&self, id: uuid::Uuid, bytes_sent: u64, bytes_received: u64) {
        let mut active = self.active.write().await;

        if let Some(pos) = active.iter().position(|c| c.id == id) {
            let mut info = active.remove(pos);
            info.set_closed();
            info.bytes_sent = bytes_sent;
            info.bytes_received = bytes_received;

            self.add_bytes(bytes_sent, bytes_received);

            // Update per-user stats
            if let Some(ref username) = info.username {
                let mut user_stats = self.user_stats.write().await;
                if let Some(stats) = user_stats.get_mut(username) {
                    stats.active_connections = stats.active_connections.saturating_sub(1);
                    stats.total_bytes_sent += bytes_sent;
                    stats.total_bytes_received += bytes_received;
                    stats.last_activity = Some(Utc::now());
                }
            }

            let mut history = self.history.write().await;
            if history.len() >= self.max_history {
                history.pop_front();
            }
            history.push_back(ConnectionStats { info });
        }
    }

    /// Get aggregated statistics.
    pub async fn get_aggregated(&self) -> AggregatedStats {
        let active_count = self.active.read().await.len() as u64;
        let user_stats: Vec<UserStats> = self.user_stats.read().await.values().cloned().collect();

        AggregatedStats {
            total_connections: self.total_connections.load(Ordering::Relaxed),
            active_connections: active_count,
            total_bytes_sent: self.total_bytes_sent.load(Ordering::Relaxed),
            total_bytes_received: self.total_bytes_received.load(Ordering::Relaxed),
            uptime_secs: (Utc::now() - self.started_at).num_seconds(),
            started_at: self.started_at,
            users: user_stats,
        }
    }

    /// Get per-user statistics.
    pub async fn get_user_stats(&self) -> Vec<UserStats> {
        self.user_stats.read().await.values().cloned().collect()
    }

    /// Get statistics for a specific user.
    pub async fn get_user(&self, username: &str) -> Option<UserStats> {
        self.user_stats.read().await.get(username).cloned()
    }

    /// Get active connections.
    pub async fn get_active(&self) -> Vec<ConnectionInfo> {
        self.active.read().await.clone()
    }

    /// Get connection history.
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<ConnectionStats> {
        let history = self.history.read().await;
        let limit = limit.unwrap_or(history.len()).min(history.len());
        history.iter().rev().take(limit).cloned().collect()
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new(1000)
    }
}
