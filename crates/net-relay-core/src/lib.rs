//! # Net-Relay Core
//!
//! Core library for the net-relay proxy service.
//! Provides SOCKS5 and HTTP CONNECT proxy implementations.

pub mod config;
pub mod connection;
pub mod error;
pub mod proxy;
pub mod stats;

pub use config::{
    AccessControlConfig, AccessRule, Config, ConfigManager, DashboardConfig, RuleAction,
    ServerConfig, User,
};
pub use connection::{Connection, ConnectionInfo, ConnectionState};
pub use error::{Error, Result};
pub use stats::{ConnectionStats, Stats, UserStats};
