.PHONY: all wasm build

# Build all non-WASM packages
all: build

# Build the specific packages with the default target
build:
	cargo build --exclude cesium-contract-sdk --exclude state --exclude state-sdk --workspace

# Build the WASM packages
wasm:
	cargo build --target wasm32-wasi --release --package cesium-contract-sdk
	cargo build --target wasm32-wasi --release --package state
	cargo build --target wasm32-wasi --release --package state-sdk

test: 
	cargo test --exclude cesium-contract-sdk --exclude state --exclude state-sdk --workspace

# Build both
all: build wasm