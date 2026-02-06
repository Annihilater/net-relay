# macOS 客户端配置指南

本指南帮助你在 Mac 上配置代理，使内网流量通过公司电脑的 Net-Relay 服务访问。

## 方法一：系统代理设置（推荐简单使用）

### 设置 SOCKS5 代理

1. 打开 **系统设置** → **网络** → 选择你的网络连接 (如 Wi-Fi)
2. 点击 **详细信息...** → **代理**
3. 勾选 **SOCKS 代理**
4. 填写：
   - 服务器：`<公司电脑IP>`
   - 端口：`1080`
5. 点击 **好**

### 设置 HTTP 代理

如果需要 HTTP 代理：
1. 勾选 **Web 代理 (HTTP)** 和 **安全 Web 代理 (HTTPS)**
2. 填写：
   - 服务器：`<公司电脑IP>`
   - 端口：`8080`

## 方法二：仅代理特定内网地址（推荐）

### 使用 PAC 文件

创建一个 PAC（代理自动配置）文件：

```javascript
// proxy.pac
function FindProxyForURL(url, host) {
    // 内网地址走代理
    if (
        isInNet(host, "10.0.0.0", "255.0.0.0") ||      // 10.x.x.x
        isInNet(host, "172.16.0.0", "255.240.0.0") ||  // 172.16-31.x.x
        isInNet(host, "192.168.0.0", "255.255.0.0") || // 192.168.x.x
        dnsDomainIs(host, ".company.internal") ||      // 公司内网域名
        dnsDomainIs(host, ".corp.example.com")         // 替换为你的内网域名
    ) {
        return "SOCKS5 <公司电脑IP>:1080";
    }
    
    // 其他走直连
    return "DIRECT";
}
```

使用方法：
1. 将上面的内容保存为 `proxy.pac`
2. 系统设置 → 网络 → 详细信息 → 代理
3. 勾选 **自动代理配置**
4. 填入文件路径：`file:///Users/你的用户名/proxy.pac`

## 方法三：命令行配置路由表

适合需要精确控制的用户：

```bash
# 假设公司电脑 IP 是 192.168.1.100
# 假设内网网段是 10.0.0.0/8

# 添加路由规则（需要代理支持 tun 模式，这里只是示例）
# 实际上 SOCKS5 代理需要应用层支持，不能直接修改路由

# 对于命令行工具，可以设置环境变量
export ALL_PROXY=socks5://192.168.1.100:1080
export HTTP_PROXY=http://192.168.1.100:8080
export HTTPS_PROXY=http://192.168.1.100:8080

# 或者添加到 ~/.zshrc 或 ~/.bashrc
```

## 方法四：使用 proxychains（命令行工具）

安装 proxychains-ng：

```bash
brew install proxychains-ng
```

配置 `/usr/local/etc/proxychains.conf`：

```conf
strict_chain
proxy_dns
tcp_read_time_out 15000
tcp_connect_time_out 8000

[ProxyList]
socks5 192.168.1.100 1080
```

使用：

```bash
# 让 curl 走代理
proxychains4 curl http://internal.company.com

# 让 ssh 走代理
proxychains4 ssh user@internal-server
```

## 常用应用配置

### Chrome / Edge

使用系统代理设置，或安装 [SwitchyOmega](https://github.com/AcroMace/SwitchyOmega) 扩展进行精细控制。

### Firefox

1. 设置 → 网络设置
2. 选择 **手动代理配置**
3. SOCKS 主机：`<公司电脑IP>`，端口：`1080`
4. 勾选 **SOCKS v5**
5. 勾选 **使用 SOCKS v5 时代理 DNS 请求**

### Git

```bash
# 全局设置
git config --global http.proxy socks5://192.168.1.100:1080
git config --global https.proxy socks5://192.168.1.100:1080

# 只对特定域名设置
git config --global http.https://github.company.com.proxy socks5://192.168.1.100:1080
```

### SSH

编辑 `~/.ssh/config`：

```ssh-config
Host internal-*
    ProxyCommand nc -x 192.168.1.100:1080 %h %p

Host *.company.internal
    ProxyCommand nc -x 192.168.1.100:1080 %h %p
```

### VS Code Remote SSH

在 `~/.ssh/config` 配置后，VS Code 会自动使用。

## 验证代理是否工作

```bash
# 测试 SOCKS5 代理
curl --socks5 192.168.1.100:1080 http://internal.company.com

# 测试 HTTP 代理
curl -x http://192.168.1.100:8080 http://internal.company.com

# 查看当前外网 IP（应该显示公司网络的出口 IP）
curl --socks5 192.168.1.100:1080 ifconfig.me
```

## 故障排除

### 连接超时

1. 确认公司电脑的 Net-Relay 服务正在运行
2. 确认防火墙允许 1080/8080 端口
3. 确认两台电脑在同一网络或有网络可达性

### DNS 解析失败

SOCKS5 代理支持远程 DNS 解析。如果遇到问题：
- 确保使用 `socks5://` 而不是 `socks5h://`（某些客户端区分）
- 或者在 Mac 上配置公司 DNS 服务器

### 部分应用不走代理

某些应用可能忽略系统代理设置，需要：
1. 查看应用自身的代理设置
2. 使用 proxychains 强制代理
3. 使用 PAC 文件

## 方法五：通过 SSH 隧道加密连接（类似 ssh -D）

**推荐用于远程访问，提供加密传输**

这种方法将 SSH 隧道与 Net-Relay 结合，等同于 `ssh -D` 动态端口转发，但具备 Net-Relay 的管理功能。

### 步骤 1：配置 Net-Relay 只监听本地

修改公司电脑上的 `config.toml`：

```toml
[server]
socks5_listen = "127.0.0.1:1080"    # 只接受本地连接
http_listen = "127.0.0.1:8080"
api_listen = "127.0.0.1:3000"
```

### 步骤 2：创建 SSH 隧道

在 Mac 上执行：

```bash
# 转发本地端口到远程 Net-Relay
ssh -L 1080:localhost:1080 -L 3000:localhost:3000 user@公司电脑IP

# 后台运行方式：
ssh -f -N -L 1080:localhost:1080 -L 3000:localhost:3000 user@公司电脑IP
```

可以添加到 `~/.ssh/config`：

```ssh-config
Host company-relay
    HostName 公司电脑IP
    User 你的用户名
    LocalForward 1080 localhost:1080
    LocalForward 3000 localhost:3000
```

然后只需运行 `ssh -f -N company-relay`

### 步骤 3：配置 Mac 使用隧道

现在将 SOCKS 代理设置为 `localhost:1080`，所有流量通过加密的 SSH 隧道到达 Net-Relay。

### SSH 隧道方式的优势

- **加密传输**：Mac 与公司电脑之间的流量完全加密
- **防火墙友好**：只需 SSH 端口（22），无需开放其他端口
- **远程访问**：可从任何有 SSH 访问的地方使用
- **管理功能**：保留 Net-Relay 的访问控制、统计、Web 界面

### 与 ssh -D 的对比

| 功能 | ssh -D | Net-Relay + SSH 隧道 |
|------|--------|---------------------|
| SOCKS5 代理 | ✅ | ✅ |
| 加密传输 | ✅ | ✅ |
| IP 黑白名单 | ❌ | ✅ |
| 域名/路径规则 | ❌ | ✅ |
| 连接监控 | ❌ | ✅ |
| Web 管理界面 | ❌ | ✅ |
| 配置持久化 | ❌ | ✅ |
