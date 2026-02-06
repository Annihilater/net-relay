# 部署指南

## 目标平台

- **开发环境**: macOS (Apple Silicon M1/M2)
- **部署环境**: Ubuntu 25.10 (x86_64/AMD64)

## 方法一：在 Ubuntu 上直接编译（推荐）

这是最简单可靠的方式。

### 1. 安装 Rust

```bash
# 在 Ubuntu 上运行
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.bashrc  # 或重新打开终端
```

### 2. 安装依赖

```bash
sudo apt update
sudo apt install -y build-essential pkg-config
```

### 3. 获取代码并编译

```bash
# 方式 A: 从 Git 克隆
git clone https://github.com/yourusername/net-relay.git
cd net-relay

# 方式 B: 从 Mac 拷贝（使用 scp/rsync）
# 在 Mac 上执行:
# scp -r /Users/ziji/github/net-relay user@ubuntu-ip:~/

# 编译
cargo build --release
```

### 4. 运行

```bash
./target/release/net-relay
```

---

## 方法二：在 Mac 上交叉编译（使用 cross）

使用 [cross](https://github.com/cross-rs/cross) 工具，它使用 Docker 进行交叉编译。

### 1. 安装 Docker Desktop

确保 Docker Desktop 已安装并运行。

### 2. 安装 cross

```bash
cargo install cross --git https://github.com/cross-rs/cross
```

### 3. 交叉编译

```bash
cd /Users/ziji/github/net-relay

# 编译 Linux x86_64 版本 (glibc，适合大多数 Linux 发行版)
cross build --release --target x86_64-unknown-linux-gnu

# 或使用 musl（静态链接，更便携）
cross build --release --target x86_64-unknown-linux-musl
```

### 4. 输出文件

编译后的二进制文件位于：
- `target/x86_64-unknown-linux-gnu/release/net-relay`
- 或 `target/x86_64-unknown-linux-musl/release/net-relay`

### 5. 部署到 Ubuntu

```bash
# 拷贝二进制和前端文件
scp target/x86_64-unknown-linux-gnu/release/net-relay user@ubuntu-ip:~/net-relay/
scp -r frontend user@ubuntu-ip:~/net-relay/
scp config.example.toml user@ubuntu-ip:~/net-relay/config.toml

# 在 Ubuntu 上运行
ssh user@ubuntu-ip
cd ~/net-relay
chmod +x net-relay
./net-relay
```

---

## 方法三：使用 Makefile（推荐日常使用）

项目包含 Makefile，简化常用操作：

```bash
# 本地编译
make build

# 交叉编译到 Linux
make build-linux

# 部署到远程服务器
make deploy REMOTE=user@ubuntu-ip
```

---

## 作为 Systemd 服务运行

在 Ubuntu 上创建系统服务，使其开机自启：

### 1. 创建服务文件

```bash
sudo tee /etc/systemd/system/net-relay.service << 'EOF'
[Unit]
Description=Net-Relay Proxy Service
After=network.target

[Service]
Type=simple
User=nobody
Group=nogroup
WorkingDirectory=/opt/net-relay
ExecStart=/opt/net-relay/net-relay
Restart=on-failure
RestartSec=5

# Security
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
EOF
```

### 2. 安装文件

```bash
sudo mkdir -p /opt/net-relay
sudo cp net-relay /opt/net-relay/
sudo cp -r frontend /opt/net-relay/
sudo cp config.toml /opt/net-relay/  # 如果有配置文件
sudo chown -R nobody:nogroup /opt/net-relay
```

### 3. 启动服务

```bash
sudo systemctl daemon-reload
sudo systemctl enable net-relay
sudo systemctl start net-relay

# 查看状态
sudo systemctl status net-relay

# 查看日志
sudo journalctl -u net-relay -f
```

---

## 防火墙配置

确保 Ubuntu 防火墙允许代理端口：

```bash
# 使用 ufw
sudo ufw allow 1080/tcp   # SOCKS5
sudo ufw allow 8080/tcp   # HTTP proxy
sudo ufw allow 3000/tcp   # Dashboard (可选，仅内网访问)

# 或使用 iptables
sudo iptables -A INPUT -p tcp --dport 1080 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 8080 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 3000 -j ACCEPT
```

---

## 验证部署

```bash
# 在 Ubuntu 上检查服务
curl http://localhost:3000/api/health

# 在 Mac 上测试代理连接
curl --socks5 ubuntu-ip:1080 http://example.com
curl -x http://ubuntu-ip:8080 http://example.com
```
