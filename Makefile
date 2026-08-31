PROJECT_NAME := android-proxy-setter
BINARY_NAME := android_proxy_setter
BLUE := \033[0;34m
GREEN := \033[0;32m
NC := \033[0m

.PHONY: all build install uninstall test check fmt lint verify run clean help

all: build

build:
	@echo "$(BLUE)[INFO]$(NC) Building $(PROJECT_NAME)..."
	@cargo build --release
	@echo "$(GREEN)[SUCCESS]$(NC) Built target/release/$(BINARY_NAME)"

install: build
	@./scripts/install.sh

uninstall:
	@./scripts/uninstall.sh

test:
	@cargo test --all-targets

check:
	@cargo check --all-targets

fmt:
	@cargo fmt

lint:
	@cargo clippy --all-targets --all-features -- -D warnings

verify:
	@cargo fmt --check
	@cargo clippy --all-targets --all-features -- -D warnings
	@cargo test --all-targets

run:
	@cargo run

clean:
	@cargo clean

help:
	@echo "Build:        make build"
	@echo "Verify:       make verify"
	@echo "Run:          make run"
	@echo "Install:      make install"
	@echo "Uninstall:    make uninstall"
