//! Configuration structures for net-relay.

use serde::{Deserialize, Serialize};

/// Main configuration structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration.
    #[serde(default)]
    pub server: ServerConfig,

    /// Logging configuration.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Security configuration.
    #[serde(default)]
    pub security: SecurityConfig,

    /// Connection limits.
    #[serde(default)]
    pub limits: LimitsConfig,

    /// Statistics configuration.
    #[serde(default)]
    pub stats: StatsConfig,
}

/// Server binding configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host address to bind.
    #[serde(default = "default_host")]
    pub host: String,

    /// SOCKS5 proxy port.
    #[serde(default = "default_socks_port")]
    pub socks_port: u16,

    /// HTTP proxy port.
    #[serde(default = "default_http_port")]
    pub http_port: u16,

    /// API/Dashboard port.
    #[serde(default = "default_api_port")]
    pub api_port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            socks_port: default_socks_port(),
            http_port: default_http_port(),
            api_port: default_api_port(),
        }
    }
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_socks_port() -> u16 {
    1080
}

fn default_http_port() -> u16 {
    8080
}

fn default_api_port() -> u16 {
    3000
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level.
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log file path (optional).
    pub file: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: None,
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

/// Security configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable authentication.
    #[serde(default)]
    pub auth_enabled: bool,

    /// Username for authentication.
    pub username: Option<String>,

    /// Password for authentication.
    pub password: Option<String>,

    /// Allowed client IPs (CIDR notation).
    #[serde(default)]
    pub allowed_ips: Vec<String>,
}

/// Connection limits configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    /// Maximum concurrent connections.
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,

    /// Connection timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Idle timeout in seconds.
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_connections: default_max_connections(),
            timeout: default_timeout(),
            idle_timeout: default_idle_timeout(),
        }
    }
}

fn default_max_connections() -> usize {
    1000
}

fn default_timeout() -> u64 {
    300
}

fn default_idle_timeout() -> u64 {
    60
}

/// Statistics configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsConfig {
    /// Enable statistics collection.
    #[serde(default = "default_stats_enabled")]
    pub enabled: bool,

    /// Retention period in hours.
    #[serde(default = "default_retention_hours")]
    pub retention_hours: u64,
}

impl Default for StatsConfig {
    fn default() -> Self {
        Self {
            enabled: default_stats_enabled(),
            retention_hours: default_retention_hours(),
        }
    }
}

fn default_stats_enabled() -> bool {
    true
}

fn default_retention_hours() -> u64 {
    24
}
