//! TCP relay implementation.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::debug;

/// Relay data between two TCP streams.
///
/// Returns (bytes_sent_to_target, bytes_received_from_target).
pub async fn relay_tcp(client: TcpStream, target: TcpStream) -> (u64, u64) {
    let (mut client_read, mut client_write) = client.into_split();
    let (mut target_read, mut target_write) = target.into_split();

    let client_to_target = async {
        let mut buf = [0u8; 8192];
        let mut total: u64 = 0;

        loop {
            match client_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if target_write.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                    total += n as u64;
                }
                Err(_) => break,
            }
        }

        let _ = target_write.shutdown().await;
        total
    };

    let target_to_client = async {
        let mut buf = [0u8; 8192];
        let mut total: u64 = 0;

        loop {
            match target_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if client_write.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                    total += n as u64;
                }
                Err(_) => break,
            }
        }

        let _ = client_write.shutdown().await;
        total
    };

    let (bytes_sent, bytes_received) = tokio::join!(client_to_target, target_to_client);

    debug!(
        "Relay complete: sent={}, received={}",
        bytes_sent, bytes_received
    );

    (bytes_sent, bytes_received)
}
