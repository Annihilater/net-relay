# Surge 使用 Net-Relay 代理指南

本文说明如何在 Surge（macOS / iOS）中配置 Net-Relay 作为内网分流代理，仅让指定域名或内网 IP 走 Net-Relay，其余流量直连。

## 前置条件

- 已在公司电脑或服务器上部署并运行 Net-Relay（参见 [部署文档](DEPLOYMENT.md)）。
- 已知 Net-Relay 的访问地址与端口：
  - **SOCKS5**：默认 `1080`
  - **HTTP 代理**：默认 `8080`
- Surge 已安装并可编辑配置或安装模块。

## 端口与协议对应关系

| 协议   | 默认端口 | 说明                         |
| ------ | -------- | ---------------------------- |
| SOCKS5 | 1080     | 推荐，支持 TCP 及远程 DNS    |
| HTTP   | 8080     | 适用于仅支持 HTTP 代理的场景 |

Surge 中两种均可使用，优先推荐 SOCKS5（端口 1080）。

---

## 方式一：使用模块配置（推荐）

将内网分流写成一个 Surge 模块，便于复用和更新。

### 1. 新建模块文件

在 Surge 的「模块」目录或自定义目录下新建文件，例如：`LightAhead-Internal-Proxy.sgmodule`。

### 2. 模块内容示例

以下示例将 `lightahead.cn` 及其子域名、以及若干内网网段分流到 Net-Relay。请将 `服务器IP`、端口和规则按你的环境修改。

**使用 HTTP 代理（端口 8080）时：**

```ini
#!name=LightAhead Internal Proxy
#!desc=将 lightahead.cn 及内网 IP 分流至内网代理（Net-Relay）

[Proxy]
# 代理名称可自定义；协议为 http 时端口填 8080
InternalProxy = http, 172.16.10.168, 8080

[Rule]
DOMAIN-SUFFIX,lightahead.cn,InternalProxy
IP-CIDR,192.168.5.0/24,InternalProxy
IP-CIDR,192.168.15.0/24,InternalProxy
IP-CIDR,192.168.17.0/24,InternalProxy
IP-CIDR,192.168.19.0/24,InternalProxy
IP-CIDR,192.168.20.0/24,InternalProxy
FINAL,DIRECT
```

**使用 SOCKS5 代理（端口 1080）时：**

```ini
#!name=LightAhead Internal Proxy
#!desc=将 lightahead.cn 及内网 IP 分流至内网代理（Net-Relay）

[Proxy]
# 代理名称可自定义；协议为 socks5 时端口填 1080
InternalProxy = socks5, 172.16.10.168, 1080

[Rule]
DOMAIN-SUFFIX,lightahead.cn,InternalProxy
IP-CIDR,192.168.5.0/24,InternalProxy
IP-CIDR,192.168.15.0/24,InternalProxy
IP-CIDR,192.168.17.0/24,InternalProxy
IP-CIDR,192.168.19.0/24,InternalProxy
IP-CIDR,192.168.20.0/24,InternalProxy
FINAL,DIRECT
```

### 3. 参数说明

**`[Proxy]`**

- `InternalProxy`：代理名称，可随意命名。
- 协议与端口：
  - `http, <IP>, 8080`：Net-Relay 的 HTTP 代理。
  - `socks5, <IP>, 1080`：Net-Relay 的 SOCKS5 代理（推荐）。
- 若 Net-Relay 通过 SSH 隧道转发到本机，则 IP 填 `127.0.0.1`，端口填本地转发端口（如 1080 / 8080）。

**`[Rule]`**

- `DOMAIN-SUFFIX,lightahead.cn,InternalProxy`：匹配 `lightahead.cn` 及其所有子域名，走 InternalProxy。
- `IP-CIDR,192.168.x.0/24,InternalProxy`：匹配该网段 IP，走 InternalProxy。
- `FINAL,DIRECT`：未匹配到的流量直连，避免误走代理。

### 4. 在 Surge 中安装并启用模块

1. 打开 Surge → **模块**（Module）。
2. 若使用本地文件：点 **+**，选择 **从文件导入**，选中 `LightAhead-Internal-Proxy.sgmodule`。
3. 若使用 URL：点 **+**，选择 **从 URL 安装**，填入模块的 raw 地址。
4. 在模块列表中勾选 **LightAhead Internal Proxy**，并确保 Surge 已开启（连接模式中会应用规则）。

安装并启用后，规则会并入 Surge 的规则列表，仅匹配到的域名和 IP 会走 Net-Relay，其余保持直连。

---

## 方式二：在 Surge 主配置中直接编写

不单独做模块时，可在 Surge 的「编辑配置文件」里直接写 `[Proxy]` 和 `[Rule]`。

1. 打开 Surge → **配置** → 当前使用的配置 → **编辑配置文件**。
2. 在 `[Proxy]` 中增加一条，例如：
   - `InternalProxy = socks5, 172.16.10.168, 1080`
3. 在 `[Rule]` 中增加与上面相同的规则（DOMAIN-SUFFIX、IP-CIDR、FINAL）。
4. 保存并重载配置。

注意：若已有其他代理或规则，请把 Net-Relay 的代理名和规则插入到合适位置，并保留一条 `FINAL` 规则。

---

## 按需修改的项

| 项目      | 说明                                                                              |
| --------- | --------------------------------------------------------------------------------- |
| 服务器 IP | 将 `172.16.10.168` 改为你 Net-Relay 实际监听地址（或 SSH 隧道时的 `127.0.0.1`）。 |
| 端口      | HTTP 用 8080，SOCKS5 用 1080（与 Net-Relay 配置一致）。                           |
| 域名      | 将 `lightahead.cn` 改为你的内网域名，或增加多行 `DOMAIN-SUFFIX`。                 |
| 内网网段  | 按实际内网修改或增加 `IP-CIDR` 行，例如 `192.168.0.0/24`。                        |

---

## 通过 SSH 隧道使用 Net-Relay

若 Net-Relay 只监听在公司电脑本机（如 `127.0.0.1:1080`），你在本机用 SSH 做端口转发后，Surge 应指向本机端口。

1. 在 Mac 上建立隧道（示例）：
   ```bash
   ssh -L 1080:localhost:1080 -L 8080:localhost:8080 user@公司电脑IP
   ```
2. Surge 模块或主配置中，代理地址填：
   - `InternalProxy = socks5, 127.0.0.1, 1080`  
     或
   - `InternalProxy = http, 127.0.0.1, 8080`

这样只有匹配到规则的内网流量会经本机端口 → SSH 隧道 → Net-Relay，其余流量仍直连。

---

## 验证是否生效

1. 开启 Surge 并确保已启用含 Net-Relay 的配置/模块。
2. 在浏览器访问 `https://lightahead.cn` 或内网地址，应能正常打开（说明走了 InternalProxy）。
3. 在 Surge 的 **请求查看** / 日志里确认对应请求的出口为 `InternalProxy`。

若无法访问，请检查：Net-Relay 是否运行、防火墙是否放行 1080/8080、Surge 中代理 IP/端口是否正确、规则是否被其他规则优先匹配。

---

## 与你提供的模块的对应关系

你当前的模块示例为：

```ini
InternalProxy = http, 172.16.10.168, 1080
```

Net-Relay 默认约定为：**1080 = SOCKS5**，**8080 = HTTP**。若你当前这样配置能正常使用，说明该环境上 HTTP 代理被放在了 1080 端口（或由其他转发提供）。若无法使用，可先改为：

- `socks5, 172.16.10.168, 1080`，或
- `http, 172.16.10.168, 8080`

再按上文规则示例配置域名与 IP-CIDR 即可。
