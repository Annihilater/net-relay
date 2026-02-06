//! SOCKS5 proxy implementation.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};

use crate::config::ConfigManager;
use crate::connection::Protocol;
use crate::error::{Error, Result};
use crate::proxy::relay::relay_tcp;
use crate::stats::Stats;

// SOCKS5 constants
const SOCKS_VERSION: u8 = 0x05;
const AUTH_NONE: u8 = 0x00;
const AUTH_PASSWORD: u8 = 0x02;
const AUTH_NO_ACCEPTABLE: u8 = 0xFF;
const CMD_CONNECT: u8 = 0x01;
const ADDR_TYPE_IPV4: u8 = 0x01;
const ADDR_TYPE_DOMAIN: u8 = 0x03;
const ADDR_TYPE_IPV6: u8 = 0x04;
const REP_SUCCESS: u8 = 0x00;
#[allow(dead_code)]
const REP_GENERAL_FAILURE: u8 = 0x01;
const REP_CONNECTION_REFUSED: u8 = 0x05;
const REP_CMD_NOT_SUPPORTED: u8 = 0x07;
const REP_NOT_ALLOWED: u8 = 0x02;
#[allow(dead_code)]
const REP_ADDR_NOT_SUPPORTED: u8 = 0x08;

/// SOCKS5 proxy server.
pub struct Socks5Proxy {
    /// Bind address.
    bind_addr: SocketAddr,

    /// Statistics collector.
    stats: Arc<Stats>,

    /// Configuration manager.
    config_manager: ConfigManager,
}

impl Socks5Proxy {
    /// Create a new SOCKS5 proxy.
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

    /// Start the SOCKS5 proxy server.
    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(self.bind_addr).await?;
        info!("SOCKS5 proxy listening on {}", self.bind_addr);

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

/// Handle a single SOCKS5 client connection.
async fn handle_client(
    mut stream: TcpStream,
    client_addr: SocketAddr,
    stats: Arc<Stats>,
    config_manager: ConfigManager,
) -> Result<()> {
    debug!("New SOCKS5 connection from {}", client_addr);

    // Check IP access control
    let client_ip = client_addr.ip().to_string();
    if !config_manager.is_ip_allowed(&client_ip).await {
        warn!("IP blocked: {}", client_ip);
        return Err(Error::AccessDenied(format!("IP blocked: {}", client_ip)));
    }

    // Read version and auth methods
    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf).await?;

    if buf[0] != SOCKS_VERSION {
        return Err(Error::InvalidSocks5Protocol(format!(
            "Invalid version: {}",
            buf[0]
        )));
    }

    let nmethods = buf[1] as usize;
    let mut methods = vec![0u8; nmethods];
    stream.read_exact(&mut methods).await?;

    // Handle authentication based on config
    let auth_enabled = config_manager.is_auth_enabled().await;
    let authenticated_user: Option<String>;

    if auth_enabled {
        if !methods.contains(&AUTH_PASSWORD) {
            stream
                .write_all(&[SOCKS_VERSION, AUTH_NO_ACCEPTABLE])
                .await?;
            return Err(Error::AuthenticationFailed);
        }
        stream.write_all(&[SOCKS_VERSION, AUTH_PASSWORD]).await?;

        // Read and verify username/password auth
        authenticated_user = authenticate_user(&mut stream, &config_manager).await?;
        if authenticated_user.is_none() {
            return Err(Error::AuthenticationFailed);
        }
    } else {
        authenticated_user = None;
        if !methods.contains(&AUTH_NONE) {
            stream
                .write_all(&[SOCKS_VERSION, AUTH_NO_ACCEPTABLE])
                .await?;
            return Err(Error::AuthenticationFailed);
        }
        stream.write_all(&[SOCKS_VERSION, AUTH_NONE]).await?;
    }

    // Read connection request
    let mut header = [0u8; 4];
    stream.read_exact(&mut header).await?;

    if header[0] != SOCKS_VERSION {
        return Err(Error::InvalidSocks5Protocol(
            "Invalid request version".into(),
        ));
    }

    let cmd = header[1];
    let atyp = header[3];

    if cmd != CMD_CONNECT {
        send_reply(&mut stream, REP_CMD_NOT_SUPPORTED).await?;
        return Err(Error::UnsupportedCommand(cmd));
    }

    // Parse target address
    let (target_addr, target_port) = parse_address(&mut stream, atyp).await?;

    // Check target access control
    if !config_manager.is_target_allowed(&target_addr, None).await {
        warn!("Target blocked: {}:{}", target_addr, target_port);
        send_reply(&mut stream, REP_NOT_ALLOWED).await?;
        return Err(Error::AccessDenied(format!(
            "Target blocked: {}:{}",
            target_addr, target_port
        )));
    }

    debug!("SOCKS5 CONNECT to {}:{}", target_addr, target_port);

    // Connect to target
    let target = format!("{}:{}", target_addr, target_port);
    let target_stream = match TcpStream::connect(&target).await {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to connect to {}: {}", target, e);
            send_reply(&mut stream, REP_CONNECTION_REFUSED).await?;
            return Err(Error::ConnectionRefused(target));
        }
    };

    // Send success reply
    send_reply(&mut stream, REP_SUCCESS).await?;

    // Create connection for tracking with user info
    let conn_info = crate::connection::ConnectionInfo::with_user(
        Protocol::Socks5,
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
        "SOCKS5 connection closed: {} -> {}:{}{} (sent: {}, recv: {})",
        client_addr, target_addr, target_port, user_info, bytes_sent, bytes_received
    );

    Ok(())
}

/// Authenticate using username/password with multi-user support.
/// Returns the authenticated username on success, None on failure.
async fn authenticate_user(
    stream: &mut TcpStream,
    config_manager: &ConfigManager,
) -> Result<Option<String>> {
    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf).await?;

    // Auth version (should be 0x01)
    if buf[0] != 0x01 {
        stream.write_all(&[0x01, 0x01]).await?;
        return Ok(None);
    }

    // Read username
    stream.read_exact(&mut buf).await?;
    let ulen = buf[0] as usize;
    let mut username_bytes = vec![0u8; ulen];
    stream.read_exact(&mut username_bytes).await?;

    // Read password
    stream.read_exact(&mut buf).await?;
    let plen = buf[0] as usize;
    let mut password_bytes = vec![0u8; plen];
    stream.read_exact(&mut password_bytes).await?;

    let username = String::from_utf8_lossy(&username_bytes);
    let password = String::from_utf8_lossy(&password_bytes);

    // Authenticate using config_manager (supports multi-user)
    if let Some(authenticated_user) = config_manager.authenticate(&username, &password).await {
        stream.write_all(&[0x01, 0x00]).await?;
        Ok(Some(authenticated_user))
    } else {
        stream.write_all(&[0x01, 0x01]).await?;
        Ok(None)
    }
}

/// Parse SOCKS5 address.
async fn parse_address(stream: &mut TcpStream, atyp: u8) -> Result<(String, u16)> {
    let addr = match atyp {
        ADDR_TYPE_IPV4 => {
            let mut buf = [0u8; 4];
            stream.read_exact(&mut buf).await?;
            format!("{}.{}.{}.{}", buf[0], buf[1], buf[2], buf[3])
        }
        ADDR_TYPE_DOMAIN => {
            let mut len = [0u8; 1];
            stream.read_exact(&mut len).await?;
            let mut domain = vec![0u8; len[0] as usize];
            stream.read_exact(&mut domain).await?;
            String::from_utf8_lossy(&domain).to_string()
        }
        ADDR_TYPE_IPV6 => {
            let mut buf = [0u8; 16];
            stream.read_exact(&mut buf).await?;
            format!(
                "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
                u16::from_be_bytes([buf[0], buf[1]]),
                u16::from_be_bytes([buf[2], buf[3]]),
                u16::from_be_bytes([buf[4], buf[5]]),
                u16::from_be_bytes([buf[6], buf[7]]),
                u16::from_be_bytes([buf[8], buf[9]]),
                u16::from_be_bytes([buf[10], buf[11]]),
                u16::from_be_bytes([buf[12], buf[13]]),
                u16::from_be_bytes([buf[14], buf[15]]),
            )
        }
        _ => {
            return Err(Error::UnsupportedAddressType(atyp));
        }
    };

    let mut port_buf = [0u8; 2];
    stream.read_exact(&mut port_buf).await?;
    let port = u16::from_be_bytes(port_buf);

    Ok((addr, port))
}

/// Send SOCKS5 reply.
async fn send_reply(stream: &mut TcpStream, rep: u8) -> Result<()> {
    // Reply: VER REP RSV ATYP BND.ADDR BND.PORT
    // We send 0.0.0.0:0 as bound address
    let reply = [
        SOCKS_VERSION,
        rep,
        0x00, // RSV
        ADDR_TYPE_IPV4,
        0,
        0,
        0,
        0, // BND.ADDR
        0,
        0, // BND.PORT
    ];
    stream.write_all(&reply).await?;
    Ok(())
}
