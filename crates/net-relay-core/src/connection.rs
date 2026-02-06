//! Connection tracking and management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the state of a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    /// Connection is being established.
    Connecting,
    /// Connection is active and transferring data.
    Active,
    /// Connection is closing.
    Closing,
    /// Connection is closed.
    Closed,
}

/// Protocol type for the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    /// SOCKS5 proxy protocol.
    Socks5,
    /// HTTP CONNECT proxy protocol.
    HttpConnect,
}

/// Information about a single connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Unique connection identifier.
    pub id: Uuid,

    /// Protocol used.
    pub protocol: Protocol,

    /// Client address.
    pub client_addr: String,

    /// Target address (destination).
    pub target_addr: String,

    /// Target port.
    pub target_port: u16,

    /// Current state.
    pub state: ConnectionState,

    /// When the connection was established.
    pub connected_at: DateTime<Utc>,

    /// When the connection was closed (if applicable).
    pub closed_at: Option<DateTime<Utc>>,

    /// Bytes sent to target.
    pub bytes_sent: u64,

    /// Bytes received from target.
    pub bytes_received: u64,
}

impl ConnectionInfo {
    /// Create a new connection info.
    pub fn new(protocol: Protocol, client_addr: String, target_addr: String, target_port: u16) -> Self {
        Self {
            id: Uuid::new_v4(),
            protocol,
            client_addr,
            target_addr,
            target_port,
            state: ConnectionState::Connecting,
            connected_at: Utc::now(),
            closed_at: None,
            bytes_sent: 0,
            bytes_received: 0,
        }
    }

    /// Mark the connection as active.
    pub fn set_active(&mut self) {
        self.state = ConnectionState::Active;
    }

    /// Mark the connection as closing.
    pub fn set_closing(&mut self) {
        self.state = ConnectionState::Closing;
    }

    /// Mark the connection as closed.
    pub fn set_closed(&mut self) {
        self.state = ConnectionState::Closed;
        self.closed_at = Some(Utc::now());
    }

    /// Add bytes to the sent counter.
    pub fn add_bytes_sent(&mut self, bytes: u64) {
        self.bytes_sent += bytes;
    }

    /// Add bytes to the received counter.
    pub fn add_bytes_received(&mut self, bytes: u64) {
        self.bytes_received += bytes;
    }

    /// Get connection duration in seconds.
    pub fn duration_secs(&self) -> i64 {
        let end = self.closed_at.unwrap_or_else(Utc::now);
        (end - self.connected_at).num_seconds()
    }
}

/// A wrapper around an active connection for tracking.
#[derive(Debug)]
pub struct Connection {
    /// Connection information.
    pub info: ConnectionInfo,
}

impl Connection {
    /// Create a new connection.
    pub fn new(protocol: Protocol, client_addr: String, target_addr: String, target_port: u16) -> Self {
        Self {
            info: ConnectionInfo::new(protocol, client_addr, target_addr, target_port),
        }
    }
}
