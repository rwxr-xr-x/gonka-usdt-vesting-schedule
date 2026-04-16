.PHONY: build build-dev check test clean help

PROJECT_NAME := gonka-usdt-vesting-schedule
ARTIFACTS_DIR := artifacts

build: clean
	@echo "Building $(PROJECT_NAME) contract..."
	@mkdir -p $(ARTIFACTS_DIR)
	@docker run \
		-v "$(CURDIR)":/code \
		--mount type=volume,source="$(PROJECT_NAME)_cache",target=/code/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		cosmwasm/rust-optimizer:0.17.0
	@echo "Build complete: $(ARTIFACTS_DIR)/$(PROJECT_NAME).wasm"

build-dev:
	@echo "Building $(PROJECT_NAME) (dev wasm)..."
	@cargo build --target wasm32-unknown-unknown --release

check:
	@cargo check

test:
	@cargo test

clean:
	@echo "Cleaning build artifacts..."
	@rm -rf $(ARTIFACTS_DIR) target/

help:
	@echo "Available targets:"
	@echo "  build      - Build optimized WASM contract (Docker)"
	@echo "  build-dev  - Build WASM without optimizer (fast)"
	@echo "  check      - Check compilation"
	@echo "  test       - Run unit tests"
	@echo "  clean      - Clean build artifacts"
	@echo "  help       - Show this help message"
