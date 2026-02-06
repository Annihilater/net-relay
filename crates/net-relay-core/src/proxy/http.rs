//! HTTP CONNECT proxy implementation.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};

use crate::connection::{Connection, Protocol};
use crate::error::{Error, Result};
use crate::proxy::relay::relay_tcp;
use crate::stats::Stats;

/// HTTP CONNECT proxy server.
pub struct HttpProxy {
    /// Bind address.
    bind_addr: SocketAddr,

    /// Authentication credentials.
    auth: Option<(String, String)>,

    /// Statistics collector.
    stats: Arc<Stats>,
}

impl HttpProxy {
    /// Create a new HTTP CONNECT proxy.
    pub fn new(bind_addr: SocketAddr, auth: Option<(String, String)>, stats: Arc<Stats>) -> Self {
        Self {
            bind_addr,
            auth,
            stats,
        }
    }

    /// Start the HTTP proxy server.
    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(self.bind_addr).await?;
        info!("HTTP CONNECT proxy listening on {}", self.bind_addr);

        loop {
            match listener.accept().await {
                Ok((stream, client_addr)) => {
                    let auth = self.auth.clone();
                    let stats = Arc::clone(&self.stats);

                    tokio::spawn(async move {
                        if let Err(e) = handle_client(stream, client_addr, auth, stats).await {
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
    auth: Option<(String, String)>,
    stats: Arc<Stats>,
) -> Result<()> {
    debug!("New HTTP CONNECT connection from {}", client_addr);

    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;

    // Parse request line: CONNECT host:port HTTP/1.1
    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();

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
    let mut has_auth = false;
    let mut auth_header = String::new();

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        if line.trim().is_empty() {
            break;
        }

        if line.to_lowercase().starts_with("proxy-authorization:") {
            has_auth = true;
            auth_header = line.trim().to_string();
        }
    }

    // Check authentication
    if let Some((username, password)) = auth.as_ref() {
        if !has_auth || !verify_auth(&auth_header, username, password) {
            let mut stream = reader.into_inner();
            stream.write_all(b"HTTP/1.1 407 Proxy Authentication Required\r\nProxy-Authenticate: Basic realm=\"Proxy\"\r\n\r\n").await?;
            return Err(Error::AuthenticationFailed);
        }
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

    // Create connection for tracking
    let conn = Connection::new(
        Protocol::HttpConnect,
        client_addr.to_string(),
        target_addr.clone(),
        target_port,
    );
    let conn_id = conn.info.id;
    stats.add_connection(conn.info).await;

    // Relay traffic
    let (bytes_sent, bytes_received) = relay_tcp(stream, target_stream).await;

    // Record stats
    stats
        .close_connection(conn_id, bytes_sent, bytes_received)
        .await;

    info!(
        "HTTP CONNECT closed: {} -> {}:{} (sent: {}, recv: {})",
        client_addr, target_addr, target_port, bytes_sent, bytes_received
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

/// Verify proxy authentication header.
fn verify_auth(header: &str, username: &str, password: &str) -> bool {
    // Parse "Proxy-Authorization: Basic base64..."
    let parts: Vec<&str> = header.splitn(2, ':').collect();
    if parts.len() != 2 {
        return false;
    }

    let auth_parts: Vec<&str> = parts[1].trim().splitn(2, ' ').collect();
    if auth_parts.len() != 2 || auth_parts[0].to_lowercase() != "basic" {
        return false;
    }

    // Decode base64
    use std::str;
    let decoded = match base64_decode(auth_parts[1].trim()) {
        Some(d) => d,
        None => return false,
    };

    let expected = format!("{}:{}", username, password);
    decoded == expected
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
