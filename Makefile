# Makefile for git-selective-ignore

# Default target: display help
.PHONY: help
help:
	@echo "Available commands:"
	@echo "  make check            - Run fmt check, clippy, and cargo check"
	@echo "  make test             - Run tests"
	@echo "  make build            - Build release binary"
	@echo "  make fmt              - Format code"
	@echo "  make fix              - Fix code (allows dirty/staged files)"
	@echo "  make changelog        - Generate CHANGELOG.md using git-cliff"
	@echo "  make install-hooks    - Install git hooks (using cargo run)"
	@echo "  make uninstall-hooks  - Uninstall git hooks (using cargo run)"
	@echo "  make clean            - Clean build artifacts"
	@echo "  make all              - Run check, test, and build"

.PHONY: all
all: check test build

# Check formatting, linting, and compile check
.PHONY: check
check:
	@echo "Checking formatting..."
	cargo fmt -- --check
	@echo "Running clippy..."
	cargo clippy -- -D warnings
	@echo "Running cargo check..."
	cargo check

# Run tests
.PHONY: test
test:
	@echo "Running tests..."
	cargo test --verbose

# Build release binary
.PHONY: build
build:
	@echo "Building release binary..."
	cargo build --release

# Format code
.PHONY: fmt
fmt:
	@echo "Formatting code..."
	cargo fmt

# Fix code
.PHONY: fix
fix:
	@echo "Fixing code..."
	cargo fix --allow-dirty --allow-staged

# Clean project
.PHONY: clean
clean:
	@echo "Cleaning project..."
	cargo clean

# Install hooks (dev helper)
.PHONY: install-hooks
install-hooks:
	cargo run -- install-hooks

# Uninstall hooks (dev helper)
.PHONY: uninstall-hooks
uninstall-hooks:
	cargo run -- uninstall-hooks

# Generate changelog
.PHONY: changelog
changelog:
	git cliff -o CHANGELOG.md
