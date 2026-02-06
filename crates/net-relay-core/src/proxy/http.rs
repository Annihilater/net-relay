//! HTTP CONNECT proxy implementation.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};

use crate::config::ConfigManager;
use crate::connection::Protocol;
use crate::error::{Error, Result};
use crate::proxy::relay::relay_tcp;
use crate::stats::Stats;

/// HTTP CONNECT proxy server.
pub struct HttpProxy {
    /// Bind address.
    bind_addr: SocketAddr,

    /// Statistics collector.
    stats: Arc<Stats>,

    /// Configuration manager.
    config_manager: ConfigManager,
}

impl HttpProxy {
    /// Create a new HTTP CONNECT proxy.
    pub fn new(
        bind_addr: SocketAddr,
        _auth: Option<(String, String)>, // Deprecated, uses config_manager now
        stats: Arc<Stats>,
        config_manager: ConfigManager,
    ) -> Self {
        Self {
            bind_addr,
            stats,
            config_manager,
        }
    }

    /// Start the HTTP proxy server.
    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(self.bind_addr).await?;
        info!("HTTP CONNECT proxy listening on {}", self.bind_addr);

        loop {
            match listener.accept().await {
                Ok((stream, client_addr)) => {
                    let stats = Arc::clone(&self.stats);
                    let config_manager = self.config_manager.clone();

                    tokio::spawn(async move {
                        if let Err(e) =
                            handle_client(stream, client_addr, stats, config_manager).await
                        {
                            debug!("Connection from {} error: {}", client_addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

/// Handle a single HTTP CONNECT client.
async fn handle_client(
    stream: TcpStream,
    client_addr: SocketAddr,
    stats: Arc<Stats>,
    config_manager: ConfigManager,
) -> Result<()> {
    debug!("New HTTP CONNECT connection from {}", client_addr);

    // Check IP access control
    let client_ip = client_addr.ip().to_string();
    if !config_manager.is_ip_allowed(&client_ip).await {
        warn!("IP blocked: {}", client_ip);
        return Err(Error::AccessDenied(format!("IP blocked: {}", client_ip)));
    }

    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;

    // Parse request line: CONNECT host:port HTTP/1.1
    let parts: Vec<&str> = request_line.split_whitespace().collect();

    if parts.len() < 3 {
        return Err(Error::InvalidHttpProtocol("Invalid request line".into()));
    }

    let method = parts[0];
    let target = parts[1];

    if method != "CONNECT" {
        let mut stream = reader.into_inner();
        stream
            .write_all(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n")
            .await?;
        return Err(Error::InvalidHttpProtocol(format!(
            "Method not allowed: {}",
            method
        )));
    }

    // Parse host:port
    let (target_addr, target_port) = parse_host_port(target)?;

    // Read headers
    let mut auth_header = String::new();

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        if line.trim().is_empty() {
            break;
        }

        if line.to_lowercase().starts_with("proxy-authorization:") {
            auth_header = line.trim().to_string();
        }
    }

    // Check authentication using config_manager (multi-user support)
    let auth_enabled = config_manager.is_auth_enabled().await;
    let authenticated_user: Option<String>;

    if auth_enabled {
        authenticated_user = extract_and_verify_auth(&auth_header, &config_manager).await;
        if authenticated_user.is_none() {
            let mut stream = reader.into_inner();
            stream.write_all(b"HTTP/1.1 407 Proxy Authentication Required\r\nProxy-Authenticate: Basic realm=\"Proxy\"\r\n\r\n").await?;
            return Err(Error::AuthenticationFailed);
        }
    } else {
        authenticated_user = None;
    }

    // Check target access control
    if !config_manager.is_target_allowed(&target_addr, None).await {
        warn!("Target blocked: {}:{}", target_addr, target_port);
        let mut stream = reader.into_inner();
        stream.write_all(b"HTTP/1.1 403 Forbidden\r\n\r\n").await?;
        return Err(Error::AccessDenied(format!(
            "Target blocked: {}:{}",
            target_addr, target_port
        )));
    }

    debug!("HTTP CONNECT to {}:{}", target_addr, target_port);

    // Connect to target
    let target = format!("{}:{}", target_addr, target_port);
    let target_stream = match TcpStream::connect(&target).await {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to connect to {}: {}", target, e);
            let mut stream = reader.into_inner();
            stream
                .write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n")
                .await?;
            return Err(Error::ConnectionRefused(target));
        }
    };

    // Send success response
    let mut stream = reader.into_inner();
    stream
        .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
        .await?;

    // Create connection for tracking with user info
    let conn_info = crate::connection::ConnectionInfo::with_user(
        Protocol::HttpConnect,
        client_addr.to_string(),
        target_addr.clone(),
        target_port,
        authenticated_user.clone(),
    );
    let conn_id = conn_info.id;
    stats.add_connection(conn_info).await;

    // Relay traffic
    let (bytes_sent, bytes_received) = relay_tcp(stream, target_stream).await;

    // Record stats
    stats
        .close_connection(conn_id, bytes_sent, bytes_received)
        .await;

    let user_info = authenticated_user
        .map(|u| format!(" (user: {})", u))
        .unwrap_or_default();
    info!(
        "HTTP CONNECT closed: {} -> {}:{}{} (sent: {}, recv: {})",
        client_addr, target_addr, target_port, user_info, bytes_sent, bytes_received
    );

    Ok(())
}

/// Parse host:port string.
fn parse_host_port(target: &str) -> Result<(String, u16)> {
    let parts: Vec<&str> = target.rsplitn(2, ':').collect();

    if parts.len() != 2 {
        return Err(Error::InvalidHttpProtocol(format!(
            "Invalid target: {}",
            target
        )));
    }

    let port: u16 = parts[0]
        .parse()
        .map_err(|_| Error::InvalidHttpProtocol(format!("Invalid port: {}", parts[0])))?;

    let host = parts[1].to_string();

    Ok((host, port))
}

/// Extract and verify proxy authentication header using multi-user config.
/// Returns the authenticated username on success.
async fn extract_and_verify_auth(header: &str, config_manager: &ConfigManager) -> Option<String> {
    if header.is_empty() {
        return None;
    }

    // Parse "Proxy-Authorization: Basic base64..."
    let parts: Vec<&str> = header.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }

    let auth_parts: Vec<&str> = parts[1].trim().splitn(2, ' ').collect();
    if auth_parts.len() != 2 || auth_parts[0].to_lowercase() != "basic" {
        return None;
    }

    // Decode base64
    let decoded = base64_decode(auth_parts[1].trim())?;

    // Parse username:password
    let cred_parts: Vec<&str> = decoded.splitn(2, ':').collect();
    if cred_parts.len() != 2 {
        return None;
    }

    let username = cred_parts[0];
    let password = cred_parts[1];

    // Authenticate using config_manager (supports multi-user)
    config_manager.authenticate(username, password).await
}

/// Simple base64 decode.
fn base64_decode(input: &str) -> Option<String> {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut output = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits = 0;

    for c in input.chars() {
        if c == '=' {
            break;
        }

        let value = CHARSET.iter().position(|&x| x as char == c)? as u32;
        buffer = (buffer << 6) | value;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            output.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }

    String::from_utf8(output).ok()
}
