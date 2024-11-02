.PHONY: all wasm build

# Build all non-WASM packages
all: build

# Build the specific packages with the default target
build:
	cargo build --exclude selenide-sdk --exclude state --exclude state-sdk --exclude nomisma --workspace --verbose

# Build the WASM packages
wasm:
	cargo build --target wasm32-unknown-unknown --release --package selenide-sdk
	cargo build --target wasm32-unknown-unknown --release --package state
	cargo build --target wasm32-unknown-unknown --release --package state-sdk
	cargo build --target wasm32-unknown-unknown --release --package nomisma

test: 
	cargo test --exclude selenide-sdk --exclude state --exclude state-sdk --exclude nomisma --workspace --verbose

# Build both
all: build wasm