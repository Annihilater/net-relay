//! Error types for the net-relay proxy.

use thiserror::Error;

/// Result type alias for net-relay operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that can occur during proxy operations.
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error during network operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid SOCKS5 protocol data.
    #[error("Invalid SOCKS5 protocol: {0}")]
    InvalidSocks5Protocol(String),

    /// Invalid HTTP protocol data.
    #[error("Invalid HTTP protocol: {0}")]
    InvalidHttpProtocol(String),

    /// Authentication failed.
    #[error("Authentication failed")]
    AuthenticationFailed,

    /// Connection refused by target.
    #[error("Connection refused: {0}")]
    ConnectionRefused(String),

    /// Connection timeout.
    #[error("Connection timeout")]
    Timeout,

    /// Address resolution failed.
    #[error("Failed to resolve address: {0}")]
    AddressResolution(String),

    /// Unsupported proxy command.
    #[error("Unsupported command: {0}")]
    UnsupportedCommand(u8),

    /// Unsupported address type.
    #[error("Unsupported address type: {0}")]
    UnsupportedAddressType(u8),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Maximum connections reached.
    #[error("Maximum connections limit reached")]
    MaxConnectionsReached,
}
