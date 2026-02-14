# Handy Development Makefile
# ===========================

.PHONY: all dev build build-dmg check lint format clean install test help

# Default target
all: check

# ===========================
# Development
# ===========================

## Install dependencies
install:
	bun install
	@echo "Downloading VAD model if needed..."
	@mkdir -p src-tauri/resources/models
	@test -f src-tauri/resources/models/silero_vad_v4.onnx || \
		curl -o src-tauri/resources/models/silero_vad_v4.onnx https://blob.handy.computer/silero_vad_v4.onnx

## Run in development mode
dev:
	CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri dev

## Build for production
build:
	bun run tauri build

## Build .dmg for macOS testing (debug mode, faster compile)
build-dmg:
	CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri build --debug --bundles dmg

# ===========================
# Code Quality
# ===========================

## Quick syntax check (fast, no codegen)
check-fast:
	@echo "==> Quick Rust syntax check..."
	cd src-tauri && cargo check --lib 2>&1 | head -100

## Full check with all targets
check:
	@echo "==> Checking Rust code..."
	cd src-tauri && cargo check 2>&1
	@echo "==> Checking TypeScript..."
	bun run lint

## Run clippy linter
lint-rust:
	@echo "==> Running clippy..."
	cd src-tauri && cargo clippy -- -D warnings 2>&1 | head -100

## Run ESLint
lint-ts:
	bun run lint

## Run all linters
lint: lint-ts lint-rust

## Format code
format:
	bun run format

## Check formatting
format-check:
	bun run format:check

# ===========================
# Rust-specific
# ===========================

## Check only the lib crate (faster)
check-lib:
	cd src-tauri && cargo check --lib

## Build only Rust backend (no frontend, no bundling)
build-rust:
	cd src-tauri && cargo build

## Build Rust in release mode
build-rust-release:
	cd src-tauri && cargo build --release

## Clean Rust build artifacts
clean-rust:
	cd src-tauri && cargo clean

## Show what's being compiled
build-verbose:
	cd src-tauri && cargo build -v 2>&1 | head -50

# ===========================
# Frontend-specific
# ===========================

## Build frontend only
build-frontend:
	bun run build

## Run frontend dev server only
dev-frontend:
	bun run dev

## Generate TypeScript bindings
generate-bindings:
	cd src-tauri && cargo build --features __SPECTA_GENERATE_BINDINGS__

# ===========================
# Testing
# ===========================

## Run Rust tests
test-rust:
	cd src-tauri && cargo test

## Run frontend tests
test-ts:
	bun run test

## Run all tests
test: test-rust test-ts

# ===========================
# Maintenance
# ===========================

## Clean all build artifacts
clean:
	rm -rf dist node_modules/.cache
	cd src-tauri && cargo clean
	@echo "Cleaned build artifacts"

## Clean and rebuild
rebuild: clean build

## Update Rust dependencies
update-rust:
	cd src-tauri && cargo update

## Update frontend dependencies
update-frontend:
	bun update

## Show outdated Rust dependencies
outdated:
	cd src-tauri && cargo outdated 2>/dev/null || echo "Install cargo-outdated: cargo install cargo-outdated"

# ===========================
# Debugging
# ===========================

## Check what crates are slow to compile
build-timings:
	cd src-tauri && cargo build --timings 2>&1 | tail -20
	@echo "Open src-tauri/target/cargo-timings/cargo-timing.html for details"

## Show dependency tree
deps:
	cd src-tauri && cargo tree --depth 1

## Show why a crate is included
why:
	@echo "Usage: make why CRATE=<crate-name>"
	@test -n "$(CRATE)" && cd src-tauri && cargo tree -i $(CRATE) || echo "Specify CRATE=<name>"

# ===========================
# Help
# ===========================

## Show this help
help:
	@echo "Handy Development Commands"
	@echo "=========================="
	@echo ""
	@echo "Quick Start:"
	@echo "  make install    - Install all dependencies"
	@echo "  make dev        - Run in development mode"
	@echo "  make build      - Build for production"
	@echo "  make build-dmg  - Build .dmg for macOS testing (debug)"
	@echo ""
	@echo "Code Quality:"
	@echo "  make check-fast - Quick syntax check (fastest)"
	@echo "  make check      - Full check (Rust + TypeScript)"
	@echo "  make lint       - Run all linters"
	@echo "  make format     - Format all code"
	@echo ""
	@echo "Rust-specific:"
	@echo "  make check-lib  - Check only Rust lib (faster)"
	@echo "  make build-rust - Build only Rust backend"
	@echo "  make lint-rust  - Run clippy"
	@echo ""
	@echo "Frontend-specific:"
	@echo "  make build-frontend - Build frontend only"
	@echo "  make lint-ts        - Run ESLint"
	@echo ""
	@echo "Debugging:"
	@echo "  make build-timings  - Show what's slow to compile"
	@echo "  make deps           - Show dependency tree"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean      - Clean all build artifacts"
	@echo "  make test       - Run all tests"
