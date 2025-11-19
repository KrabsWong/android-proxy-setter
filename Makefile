# Android Proxy Setter Makefile

# Project configuration
PROJECT_NAME = android-proxy-setter
BINARY_NAME = android_proxy_setter
INSTALL_DIR = $(HOME)/.local/bin

# Colors for output
RED = \033[0;31m
GREEN = \033[0;32m
YELLOW = \033[1;33m
BLUE = \033[0;34m
NC = \033[0m # No Color

# Default target
.PHONY: all
all: build

# Build the project
.PHONY: build
build:
	@echo "$(BLUE)[INFO]$(NC) Building $(PROJECT_NAME)..."
	@cargo build --release
	@echo "$(GREEN)[SUCCESS]$(NC) Build completed: target/release/$(BINARY_NAME)"

# Build for development
.PHONY: dev
dev:
	@echo "$(BLUE)[INFO]$(NC) Building in development mode..."
	@cargo build
	@echo "$(GREEN)[SUCCESS]$(NC) Development build completed: target/debug/$(BINARY_NAME)"

# Install to user's PATH
.PHONY: install
install: build
	@echo "$(BLUE)[INFO]$(NC) Installing $(PROJECT_NAME)..."
	@./scripts/install.sh

# Uninstall from user's PATH
.PHONY: uninstall
uninstall:
	@echo "$(BLUE)[INFO]$(NC) Uninstalling $(PROJECT_NAME)..."
	@./scripts/uninstall.sh

# Run tests
.PHONY: test
test:
	@echo "$(BLUE)[INFO]$(NC) Running tests..."
	@cargo test
	@echo "$(GREEN)[SUCCESS]$(NC) Tests passed!"

# Clean build artifacts
.PHONY: clean
clean:
	@echo "$(BLUE)[INFO]$(NC) Cleaning build artifacts..."
	@cargo clean
	@echo "$(GREEN)[SUCCESS]$(NC) Clean completed!"

# Check code without building
.PHONY: check
check:
	@echo "$(BLUE)[INFO]$(NC) Checking code..."
	@cargo check
	@echo "$(GREEN)[SUCCESS]$(NC) Code check passed!"

# Format code
.PHONY: fmt
fmt:
	@echo "$(BLUE)[INFO]$(NC) Formatting code..."
	@cargo fmt
	@echo "$(GREEN)[SUCCESS]$(NC) Code formatted!"

# Lint code
.PHONY: lint
lint:
	@echo "$(BLUE)[INFO]$(NC) Linting code..."
	@cargo clippy
	@echo "$(GREEN)[SUCCESS]$(NC) Lint passed!"

# Run in development mode
.PHONY: run
dev-run:
	@echo "$(BLUE)[INFO]$(NC) Running in development mode..."
	@cargo run

# Run in release mode
.PHONY: run-release
run-release:
	@echo "$(BLUE)[INFO]$(NC) Running in release mode..."
	@cargo run --release

# Show help
.PHONY: help
help:
	@echo "$(BLUE)Android Proxy Setter - Available commands:$(NC)"
	@echo ""
	@echo "$(GREEN)Development:$(NC)"
	@echo "  make build        - Build release version"
	@echo "  make dev          - Build development version"
	@echo "  make run          - Run development version"
	@echo "  make run-release  - Run release version"
	@echo "  make test         - Run tests"
	@echo "  make check        - Check code without building"
	@echo "  make fmt          - Format code"
	@echo "  make lint         - Lint code"
	@echo "  make clean        - Clean build artifacts"
	@echo ""
	@echo "$(GREEN)Installation:$(NC)"
	@echo "  make install      - Build and install to user's PATH"
	@echo "  make uninstall    - Remove from user's PATH"
	@echo ""
	@echo "$(GREEN)Manual installation:$(NC)"
	@echo "  ./scripts/install.sh      - Install script"
	@echo "  ./scripts/uninstall.sh    - Uninstall script"
	@echo ""
	@echo "$(YELLOW)After installation, you can use:$(NC)"
	@echo "  aps               - Interactive mode"
	@echo "  aps-set           - Set proxy directly"
	@echo "  aps-clear         - Clear proxy directly"
	@echo "  aps-view          - View current proxy settings"
	@echo "  aps-restart       - Restart ADB server"