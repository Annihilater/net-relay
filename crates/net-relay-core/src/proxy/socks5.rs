//! SOCKS5 proxy implementation.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};

use crate::connection::{Connection, Protocol};
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
#[allow(dead_code)]
const REP_ADDR_NOT_SUPPORTED: u8 = 0x08;

/// SOCKS5 proxy server.
pub struct Socks5Proxy {
    /// Bind address.
    bind_addr: SocketAddr,

    /// Authentication credentials.
    auth: Option<(String, String)>,

    /// Statistics collector.
    stats: Arc<Stats>,
}

impl Socks5Proxy {
    /// Create a new SOCKS5 proxy.
    pub fn new(bind_addr: SocketAddr, auth: Option<(String, String)>, stats: Arc<Stats>) -> Self {
        Self {
            bind_addr,
            auth,
            stats,
        }
    }

    /// Start the SOCKS5 proxy server.
    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(self.bind_addr).await?;
        info!("SOCKS5 proxy listening on {}", self.bind_addr);

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

/// Handle a single SOCKS5 client connection.
async fn handle_client(
    mut stream: TcpStream,
    client_addr: SocketAddr,
    auth: Option<(String, String)>,
    stats: Arc<Stats>,
) -> Result<()> {
    debug!("New SOCKS5 connection from {}", client_addr);

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

    // Handle authentication
    if auth.is_some() {
        if !methods.contains(&AUTH_PASSWORD) {
            stream
                .write_all(&[SOCKS_VERSION, AUTH_NO_ACCEPTABLE])
                .await?;
            return Err(Error::AuthenticationFailed);
        }
        stream.write_all(&[SOCKS_VERSION, AUTH_PASSWORD]).await?;

        // Read username/password auth
        let (username, password) = auth.as_ref().unwrap();
        if !authenticate(&mut stream, username, password).await? {
            return Err(Error::AuthenticationFailed);
        }
    } else {
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

    // Create connection for tracking
    let conn = Connection::new(
        Protocol::Socks5,
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
        "SOCKS5 connection closed: {} -> {}:{} (sent: {}, recv: {})",
        client_addr, target_addr, target_port, bytes_sent, bytes_received
    );

    Ok(())
}

/// Authenticate using username/password.
async fn authenticate(
    stream: &mut TcpStream,
    expected_user: &str,
    expected_pass: &str,
) -> Result<bool> {
    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf).await?;

    // Auth version (should be 0x01)
    if buf[0] != 0x01 {
        return Ok(false);
    }

    // Read username
    stream.read_exact(&mut buf).await?;
    let ulen = buf[0] as usize;
    let mut username = vec![0u8; ulen];
    stream.read_exact(&mut username).await?;

    // Read password
    stream.read_exact(&mut buf).await?;
    let plen = buf[0] as usize;
    let mut password = vec![0u8; plen];
    stream.read_exact(&mut password).await?;

    let username = String::from_utf8_lossy(&username);
    let password = String::from_utf8_lossy(&password);

    if username == expected_user && password == expected_pass {
        stream.write_all(&[0x01, 0x00]).await?;
        Ok(true)
    } else {
        stream.write_all(&[0x01, 0x01]).await?;
        Ok(false)
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
