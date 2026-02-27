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

## 方法四：作为 Systemd 服务运行（推荐生产环境）

将 net-relay 安装为 systemd 服务，像 nginx/sshd 一样管理，支持开机自启、自动重启、日志集成。

### 快速安装（一键脚本）

从 [GitHub Releases](https://github.com/yourusername/net-relay/releases) 下载 Linux 版本后：

```bash
# 解压
tar -xzf net-relay-x86_64-unknown-linux-gnu.tar.gz
cd net-relay-x86_64-unknown-linux-gnu

# 一键安装为 systemd 服务（默认安装到 /opt/net-relay）
sudo ./scripts/install-service.sh

# 或自定义安装目录
sudo ./scripts/install-service.sh /usr/local/net-relay
```

安装脚本会自动完成：
- 创建专用系统用户 `net-relay`（无登录权限）
- 复制文件到安装目录
- 安装并启用 systemd 服务
- 启动服务

### 日常管理

```bash
# 启动/停止/重启
sudo systemctl start net-relay
sudo systemctl stop net-relay
sudo systemctl restart net-relay

# 查看状态
sudo systemctl status net-relay

# 查看日志（实时跟踪）
journalctl -u net-relay -f

# 查看最近 100 行日志
journalctl -u net-relay -n 100 --no-pager

# 按时间查看日志
journalctl -u net-relay --since "2024-01-01 00:00:00"
```

### 修改配置

```bash
sudo vim /opt/net-relay/config.toml
sudo systemctl restart net-relay
```

### 升级版本

下载新版本后，使用升级脚本：

```bash
cd net-relay-x86_64-unknown-linux-gnu  # 新版本目录
sudo ./scripts/upgrade-service.sh
```

升级脚本会自动备份旧二进制、替换新版本、重启服务。如果新版本启动失败会自动回滚。

### 使用 Makefile 远程部署

```bash
# 首次部署为 systemd 服务
make deploy-systemd REMOTE=user@ubuntu-ip

# 后续升级
make upgrade-systemd REMOTE=user@ubuntu-ip
```

### 卸载

```bash
sudo /opt/net-relay/scripts/uninstall-service.sh
# 或从解压包目录运行
sudo ./scripts/uninstall-service.sh
```

### 手动安装（不使用脚本）

如果需要手动操作：

```bash
# 1. 创建用户和目录
sudo useradd --system --no-create-home --shell /usr/sbin/nologin net-relay
sudo mkdir -p /opt/net-relay/logs

# 2. 复制文件
sudo cp net-relay /opt/net-relay/
sudo cp config.example.toml /opt/net-relay/config.toml
sudo cp -r frontend /opt/net-relay/
sudo chown -R net-relay:net-relay /opt/net-relay

# 3. 安装 service 文件
sudo cp scripts/net-relay.service /etc/systemd/system/
# 编辑 service 文件确认路径正确
sudo vim /etc/systemd/system/net-relay.service

# 4. 启用并启动
sudo systemctl daemon-reload
sudo systemctl enable net-relay
sudo systemctl start net-relay
```

### Service 特性说明

systemd 服务配置包含以下特性：

| 特性 | 说明 |
|------|------|
| 开机自启 | `WantedBy=multi-user.target` |
| 自动重启 | 异常退出后 5 秒自动重启，60 秒内最多 5 次 |
| 日志集成 | 日志直接输出到 journald，统一管理 |
| 安全加固 | `NoNewPrivileges`、`ProtectSystem=strict`、`ProtectHome` 等 |
| 资源限制 | 文件描述符上限 65535，进程数上限 4096 |
| 专用用户 | 使用无登录权限的 `net-relay` 用户运行 |

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
