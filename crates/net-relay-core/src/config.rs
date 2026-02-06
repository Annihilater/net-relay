//! Configuration structures for net-relay.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

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

    /// Access control configuration.
    #[serde(default)]
    pub access_control: AccessControlConfig,
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a TOML file.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Runtime configuration manager for hot-reload support.
#[derive(Clone)]
pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    config_path: Option<String>,
}

impl ConfigManager {
    pub fn new(config: Config, config_path: Option<String>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
        }
    }

    /// Get current configuration.
    pub async fn get(&self) -> Config {
        self.config.read().await.clone()
    }

    /// Update configuration and optionally save to file.
    pub async fn update(&self, config: Config) -> anyhow::Result<()> {
        let mut current = self.config.write().await;
        if let Some(path) = &self.config_path {
            config.save_to_file(path)?;
        }
        *current = config;
        Ok(())
    }

    /// Update access control rules only.
    pub async fn update_access_control(
        &self,
        access_control: AccessControlConfig,
    ) -> anyhow::Result<()> {
        let mut config = self.config.write().await;
        config.access_control = access_control;
        if let Some(path) = &self.config_path {
            config.save_to_file(path)?;
        }
        Ok(())
    }

    /// Check if an IP is allowed.
    pub async fn is_ip_allowed(&self, ip: &str) -> bool {
        let config = self.config.read().await;
        config.access_control.is_ip_allowed(ip)
    }

    /// Check if a target (domain + path) is allowed.
    pub async fn is_target_allowed(&self, host: &str, path: Option<&str>) -> bool {
        let config = self.config.read().await;
        config.access_control.is_target_allowed(host, path)
    }
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

/// Access control configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlConfig {
    /// IP whitelist - if not empty, only these IPs are allowed.
    #[serde(default)]
    pub ip_whitelist: Vec<String>,

    /// IP blacklist - these IPs are blocked.
    #[serde(default)]
    pub ip_blacklist: Vec<String>,

    /// Domain/path rules.
    #[serde(default)]
    pub rules: Vec<AccessRule>,

    /// Default behavior: true = allow all (blacklist mode), false = deny all (whitelist mode).
    #[serde(default = "default_allow_by_default")]
    pub allow_by_default: bool,
}

impl Default for AccessControlConfig {
    fn default() -> Self {
        Self {
            ip_whitelist: Vec::new(),
            ip_blacklist: Vec::new(),
            rules: Vec::new(),
            allow_by_default: true, // Blacklist mode by default
        }
    }
}

fn default_allow_by_default() -> bool {
    true
}

impl AccessControlConfig {
    /// Check if an IP is allowed.
    pub fn is_ip_allowed(&self, ip: &str) -> bool {
        // Check blacklist first
        if self.ip_blacklist.iter().any(|b| ip_matches(ip, b)) {
            return false;
        }

        // If whitelist is not empty, check whitelist
        if !self.ip_whitelist.is_empty() {
            return self.ip_whitelist.iter().any(|w| ip_matches(ip, w));
        }

        true
    }

    /// Check if a target (domain + optional path) is allowed.
    pub fn is_target_allowed(&self, host: &str, path: Option<&str>) -> bool {
        // Find matching rules
        for rule in &self.rules {
            if rule.matches(host, path) {
                return rule.action == RuleAction::Allow;
            }
        }

        // No matching rule, use default behavior
        self.allow_by_default
    }
}

/// Access control rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRule {
    /// Rule name/description.
    #[serde(default)]
    pub name: String,

    /// Domain pattern (supports wildcards: *.example.com).
    pub domain: String,

    /// Path pattern (optional, supports prefix match).
    #[serde(default)]
    pub path: Option<String>,

    /// Action to take.
    pub action: RuleAction,

    /// Whether this rule is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl AccessRule {
    /// Check if this rule matches the given host and path.
    pub fn matches(&self, host: &str, path: Option<&str>) -> bool {
        if !self.enabled {
            return false;
        }

        // Check domain
        if !domain_matches(host, &self.domain) {
            return false;
        }

        // Check path if specified
        if let Some(rule_path) = &self.path {
            if let Some(request_path) = path {
                return request_path.starts_with(rule_path);
            }
            return false;
        }

        true
    }
}

/// Rule action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow,
    Deny,
}

/// Check if an IP matches a pattern (supports exact match and CIDR).
fn ip_matches(ip: &str, pattern: &str) -> bool {
    if pattern.contains('/') {
        // CIDR notation - simplified check (exact implementation would use ipnetwork crate)
        ip.starts_with(pattern.split('/').next().unwrap_or(""))
    } else {
        ip == pattern
    }
}

/// Check if a domain matches a pattern (supports wildcards).
fn domain_matches(domain: &str, pattern: &str) -> bool {
    if pattern.starts_with("*.") {
        // Wildcard match
        let suffix = &pattern[1..]; // ".example.com"
        domain.ends_with(suffix) || domain == &pattern[2..]
    } else {
        domain == pattern
    }
}
