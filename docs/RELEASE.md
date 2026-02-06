# 发布指南 (Release Guide)

本文档说明如何为 net-relay 项目打标签、推送到 GitHub 并自动生成发布制品。

## 📋 目录

- [发布流程概述](#发布流程概述)
- [版本号规范](#版本号规范)
- [创建和推送 Tag](#创建和推送-tag)
- [自动发布制品](#自动发布制品)
- [手动触发发布](#手动触发发布)
- [验证发布](#验证发布)
- [故障排查](#故障排查)

---

## 🚀 发布流程概述

项目使用 GitHub Actions 实现自动化发布流程：

1. 开发者在本地创建版本标签（tag）
2. 推送标签到 GitHub
3. GitHub Actions 自动触发 Release workflow
4. 自动编译多平台二进制文件
5. 自动创建 GitHub Release 并上传制品
6. 自动生成 SHA256 校验和

---

## 📌 版本号规范

遵循 [语义化版本 (Semantic Versioning)](https://semver.org/lang/zh-CN/) 规范：

```
v主版本号.次版本号.修订号[-预发布标识]
```

### 示例

- **正式版本**: `v0.1.0`, `v1.0.0`, `v1.2.3`
- **预发布版本**: `v0.1.0-alpha`, `v1.0.0-beta.1`, `v2.0.0-rc.1`

### 版本号递增规则

- **主版本号 (MAJOR)**: 不兼容的 API 变更
- **次版本号 (MINOR)**: 向下兼容的功能新增
- **修订号 (PATCH)**: 向下兼容的问题修复

---

## 🏷️ 创建和推送 Tag

### 步骤 1: 确保代码通过所有检查

在打标签前，确保代码已通过所有测试：

```bash
# 运行预推送检查
make pre-push

# 或者手动执行
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

### 步骤 2: 更新版本号

更新 `Cargo.toml` 中的版本号：

```toml
[workspace.package]
version = "0.2.0"  # 更新为新版本号
```

### 步骤 3: 更新 CHANGELOG

在 `CHANGELOG.md` 中记录版本变更：

```markdown
## [0.2.0] - 2026-02-06

### Added

- 新功能描述

### Changed

- 变更描述

### Fixed

- 修复描述
```

### 步骤 4: 提交变更

```bash
# 提交版本号和 CHANGELOG 变更
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"

# 推送到远程仓库
git push origin main
```

### 步骤 5: 创建 Git Tag

#### 方式 1: 创建带注释的标签（推荐）

```bash
# 创建带注释的标签
git tag -a v0.2.0 -m "Release version 0.2.0

主要更新:
- 新增 XXX 功能
- 优化 YYY 性能
- 修复 ZZZ 问题
"
```

#### 方式 2: 创建轻量级标签

```bash
# 创建轻量级标签（简单场景）
git tag v0.2.0
```

### 步骤 6: 推送 Tag 到 GitHub

```bash
# 推送单个标签
git push origin v0.2.0

# 或者推送所有本地标签
git push origin --tags
```

---

## 🤖 自动发布制品

### 触发条件

推送以 `v` 开头的标签后，GitHub Actions 会自动：

1. **多平台编译**，生成以下平台的二进制文件：
   - Linux x86_64 (glibc)
   - Linux x86_64 (musl - 静态链接)
   - Linux ARM64
   - macOS x86_64 (Intel)
   - macOS ARM64 (Apple Silicon)
   - Windows x86_64
   - Windows ARM64

2. **打包制品**，每个平台的包含：
   - 编译后的二进制文件
   - Frontend 前端文件
   - 配置文件示例 (`config.example.toml`)
   - README 和 LICENSE

3. **创建 GitHub Release**：
   - 自动生成 Release Notes
   - 上传所有平台的压缩包
   - 生成 SHA256 校验和文件

### 制品命名规范

```
net-relay-<target-platform>.tar.gz  # Linux/macOS
net-relay-<target-platform>.zip     # Windows
```

示例：

- `net-relay-x86_64-unknown-linux-gnu.tar.gz`
- `net-relay-x86_64-apple-darwin.tar.gz`
- `net-relay-x86_64-pc-windows-msvc.zip`

---

## 🎬 手动触发发布

如果需要重新发布或修复发布问题，可以手动触发：

### 在 GitHub 网页操作

1. 进入项目的 GitHub 页面
2. 点击 **Actions** 标签
3. 选择左侧的 **Release** workflow
4. 点击右上角 **Run workflow**
5. 输入标签名称（如 `v0.2.0`）
6. 点击 **Run workflow** 确认

### 使用 GitHub CLI

```bash
# 安装 GitHub CLI (如果未安装)
brew install gh  # macOS
# 或从 https://cli.github.com/ 下载

# 手动触发 release workflow
gh workflow run release.yml -f tag=v0.2.0
```

---

## ✅ 验证发布

### 1. 检查 GitHub Actions 状态

```bash
# 访问 Actions 页面
https://github.com/<你的用户名>/net-relay/actions

# 或使用 gh CLI
gh run list --workflow=release.yml
```

### 2. 检查 Release 页面

```bash
# 访问 Releases 页面
https://github.com/<你的用户名>/net-relay/releases

# 或使用 gh CLI
gh release view v0.2.0
```

### 3. 下载并验证制品

```bash
# 下载 release 制品
gh release download v0.2.0

# 验证校验和
sha256sum -c checksums.sha256

# 解压并测试
tar -xzf net-relay-x86_64-unknown-linux-gnu.tar.gz
cd net-relay
./net-relay --version
```

---

## 🔧 故障排查

### 问题 1: 推送标签失败

**错误信息**:

```
! [rejected]        v0.2.0 -> v0.2.0 (already exists)
```

**解决方案**:

```bash
# 删除本地标签
git tag -d v0.2.0

# 删除远程标签（谨慎操作！）
git push origin :refs/tags/v0.2.0

# 重新创建标签
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

### 问题 2: GitHub Actions 构建失败

**排查步骤**:

1. 查看 Actions 日志：

   ```bash
   gh run list --workflow=release.yml
   gh run view <run-id> --log
   ```

2. 常见原因：
   - 编译错误：在本地先运行 `cargo build --release` 测试
   - 依赖问题：检查 `Cargo.toml` 依赖配置
   - 权限问题：确保仓库的 Actions 权限已启用

3. 本地测试交叉编译：

   ```bash
   # 安装 cross
   cargo install cross

   # 测试 Linux 编译
   cross build --release --target x86_64-unknown-linux-gnu
   ```

### 问题 3: Release 创建失败

**可能原因**:

1. **权限不足**:
   - 进入仓库 Settings → Actions → General
   - 确保 "Workflow permissions" 设置为 "Read and write permissions"

2. **标签已存在**:
   ```bash
   # 删除已存在的 release (谨慎操作！)
   gh release delete v0.2.0 --yes
   ```

### 问题 4: 制品缺失或不完整

检查构建日志中的 artifact 上传部分：

```bash
# 查看具体的 job 日志
gh run view <run-id> --log --job <job-id>
```

---

## 📚 相关命令速查

### Git 标签操作

```bash
# 列出所有标签
git tag

# 查看标签详情
git show v0.2.0

# 删除本地标签
git tag -d v0.2.0

# 删除远程标签
git push origin :refs/tags/v0.2.0

# 获取最新标签
git describe --tags --abbrev=0
```

### GitHub CLI 操作

```bash
# 查看所有 releases
gh release list

# 查看特定 release
gh release view v0.2.0

# 下载 release 制品
gh release download v0.2.0

# 删除 release
gh release delete v0.2.0

# 查看 workflow 运行状态
gh run list --workflow=release.yml

# 查看运行日志
gh run view --log
```

---

## 🎯 最佳实践

1. **发布前检查清单**:
   - [ ] 所有测试通过
   - [ ] 代码格式化检查通过
   - [ ] Clippy 静态分析无警告
   - [ ] CHANGELOG 已更新
   - [ ] 版本号已更新
   - [ ] 文档已更新

2. **标签命名**:
   - ✅ 使用 `v` 前缀: `v1.0.0`
   - ✅ 遵循语义化版本
   - ❌ 不要使用 `latest`, `stable` 等动态标签

3. **发布节奏**:
   - 主要功能使用 minor 版本
   - 紧急修复使用 patch 版本
   - 破坏性变更使用 major 版本

4. **沟通**:
   - 在 Release Notes 中清晰描述变更
   - 对于破坏性变更，提供迁移指南
   - 在社区渠道公告重要版本

---

## 📝 示例：完整发布流程

```bash
# 1. 确保在主分支并且是最新代码
git checkout main
git pull origin main

# 2. 运行测试
make pre-push

# 3. 更新版本号和 CHANGELOG
vim Cargo.toml CHANGELOG.md

# 4. 提交变更
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"
git push origin main

# 5. 创建并推送标签
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0

# 6. 等待 GitHub Actions 完成（约 10-15 分钟）
gh run list --workflow=release.yml

# 7. 验证发布
gh release view v0.2.0

# 8. 下载并测试
gh release download v0.2.0
sha256sum -c checksums.sha256
```

---

## 🔗 相关文档

- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [语义化版本规范](https://semver.org/lang/zh-CN/)
- [Git 标签文档](https://git-scm.com/book/zh/v2/Git-基础-打标签)
- [Rust 交叉编译指南](https://rust-lang.github.io/rustup/cross-compilation.html)

---

## ❓ 需要帮助？

如果在发布过程中遇到问题：

1. 查看 [GitHub Issues](https://github.com/<你的用户名>/net-relay/issues)
2. 查看 [GitHub Actions 日志](https://github.com/<你的用户名>/net-relay/actions)
3. 提交新的 Issue 寻求帮助

---

**最后更新**: 2026-02-06
