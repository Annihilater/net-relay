# Net-Relay

[![CI](https://github.com/yourusername/net-relay/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/net-relay/actions/workflows/ci.yml)
[![Release](https://github.com/yourusername/net-relay/actions/workflows/release.yml/badge.svg)](https://github.com/yourusername/net-relay/actions/workflows/release.yml)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A network relay proxy service that allows routing internal network traffic through authorized devices. Perfect for scenarios where your personal device cannot access the corporate intranet directly.

## 📦 Download

Pre-built binaries are available for multiple platforms:

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x86_64 (glibc) | [net-relay-x86_64-unknown-linux-gnu.tar.gz](https://github.com/yourusername/net-relay/releases/latest) |
| Linux | x86_64 (musl, static) | [net-relay-x86_64-unknown-linux-musl.tar.gz](https://github.com/yourusername/net-relay/releases/latest) |
| Linux | ARM64 | [net-relay-aarch64-unknown-linux-gnu.tar.gz](https://github.com/yourusername/net-relay/releases/latest) |
| macOS | Intel | [net-relay-x86_64-apple-darwin.tar.gz](https://github.com/yourusername/net-relay/releases/latest) |
| macOS | Apple Silicon | [net-relay-aarch64-apple-darwin.tar.gz](https://github.com/yourusername/net-relay/releases/latest) |
| Windows | x86_64 | [net-relay-x86_64-pc-windows-msvc.zip](https://github.com/yourusername/net-relay/releases/latest) |
| Windows | ARM64 | [net-relay-aarch64-pc-windows-msvc.zip](https://github.com/yourusername/net-relay/releases/latest) |

## 🎯 Use Case

```
┌─────────────────┐          ┌──────────────────┐          ┌─────────────────┐
│  Personal Mac   │ ──────▶  │  Company Device  │ ──────▶  │  Internal       │
│  (No Access)    │   SOCKS  │  (Net-Relay)     │  Direct  │  Services       │
└─────────────────┘          └──────────────────┘          └─────────────────┘
```

## ✨ Features

- **SOCKS5 Proxy**: Full SOCKS5 protocol support for TCP connections
- **HTTP Proxy**: HTTP/HTTPS CONNECT method support
- **Web Dashboard**: Real-time connection statistics and monitoring
- **Connection Logging**: Track all proxied connections
- **Configurable**: YAML/TOML configuration file support
- **Lightweight**: Minimal resource footprint

## 🚀 Quick Start

### Prerequisites

- Rust 1.75 or higher
- A device with access to the target network

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/net-relay.git
cd net-relay

# Build the project
cargo build --release

# Run the proxy server
./target/release/net-relay
```

### Configuration

Create a `config.toml` file:

```toml
[server]
host = "0.0.0.0"
socks_port = 1080
http_port = 8080
api_port = 3000

[logging]
level = "info"

[security]
# Optional: Add authentication
# username = "admin"
# password = "secret"
```

### Client Setup (macOS)

Configure your Mac to use the proxy for internal network addresses:

```bash
# Add route for internal network through proxy
# Example: Route 10.0.0.0/8 through the proxy
networksetup -setsocksfirewallproxy "Wi-Fi" <company-device-ip> 1080
```

Or configure in System Preferences → Network → Advanced → Proxies.

## 📁 Project Structure

```
net-relay/
├── Cargo.toml              # Workspace configuration
├── README.md
├── LICENSE
├── config.example.toml     # Example configuration
├── crates/
│   ├── net-relay-core/     # Core proxy logic
│   ├── net-relay-server/   # Server binary
│   └── net-relay-api/      # REST API for dashboard
└── frontend/               # Web dashboard
    ├── index.html
    ├── src/
    │   ├── main.js
    │   └── style.css
    └── package.json
```

## 🔧 Development

```bash
# Run in development mode with hot reload
cargo watch -x run -p net-relay-server

# Run tests
cargo test --workspace

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy --workspace
```

## 📊 Dashboard

Access the web dashboard at `http://localhost:3000` to view:

- Active connections
- Traffic statistics
- Connection history
- Server status

## 🔒 Security Considerations

- Run the proxy only on trusted networks
- Enable authentication for production use
- Consider using TLS for the admin API
- Limit access using firewall rules

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! Please read the contributing guidelines before submitting a pull request.
