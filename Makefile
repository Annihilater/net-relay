.PHONY: build build-linux build-musl run clean deploy help

# 默认目标
all: build

# 本地编译
build:
	cargo build --release

# 交叉编译到 Linux x86_64 (glibc)
build-linux:
	cross build --release --target x86_64-unknown-linux-gnu

# 交叉编译到 Linux x86_64 (musl, 静态链接)
build-musl:
	cross build --release --target x86_64-unknown-linux-musl

# 运行
run:
	cargo run --release

# 开发模式运行
dev:
	RUST_LOG=debug cargo run

# 清理
clean:
	cargo clean

# 格式化代码
fmt:
	cargo fmt

# 检查代码
check:
	cargo check
	cargo clippy -- -D warnings

# 运行测试
test:
	cargo test --workspace

# 预推送检查 (与 CI 一致)
pre-push:
	@echo "🔍 Running pre-push checks..."
	cargo fmt --check
	cargo clippy -- -D warnings
	cargo build --release
	cargo test
	@echo "✓ All checks passed! Safe to push."

# 安装 git pre-push hook
install-hooks:
	@echo "Installing git hooks..."
	@mkdir -p .git/hooks
	@echo '#!/bin/bash' > .git/hooks/pre-push
	@echo 'make pre-push' >> .git/hooks/pre-push
	@chmod +x .git/hooks/pre-push
	@echo "✓ Pre-push hook installed"

# 部署到远程服务器
# 使用: make deploy REMOTE=user@host
deploy: build-linux
ifndef REMOTE
	$(error REMOTE is not set. Usage: make deploy REMOTE=user@host)
endif
	ssh $(REMOTE) 'mkdir -p ~/net-relay'
	scp target/x86_64-unknown-linux-gnu/release/net-relay $(REMOTE):~/net-relay/
	scp -r frontend $(REMOTE):~/net-relay/
	scp config.example.toml $(REMOTE):~/net-relay/config.toml
	@echo "Deployed to $(REMOTE):~/net-relay/"
	@echo "Run: ssh $(REMOTE) 'cd ~/net-relay && ./net-relay'"

# 快速部署（只更新二进制）
deploy-bin: build-linux
ifndef REMOTE
	$(error REMOTE is not set. Usage: make deploy-bin REMOTE=user@host)
endif
	scp target/x86_64-unknown-linux-gnu/release/net-relay $(REMOTE):~/net-relay/
	@echo "Binary updated on $(REMOTE)"

help:
	@echo "Net-Relay Makefile"
	@echo ""
	@echo "Usage:"
	@echo "  make build        - Build for current platform"
	@echo "  make build-linux  - Cross-compile for Linux x86_64 (requires cross)"
	@echo "  make build-musl   - Cross-compile static binary for Linux"
	@echo "  make run          - Run the server"
	@echo "  make dev          - Run in development mode with debug logging"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make fmt          - Format code"
	@echo "  make check        - Run checks and clippy"
	@echo "  make test         - Run tests"
	@echo "  make deploy REMOTE=user@host  - Deploy to remote server"
	@echo "  make deploy-bin REMOTE=user@host  - Deploy only binary"
