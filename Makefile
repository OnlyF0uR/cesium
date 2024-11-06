.PHONY: all build wasm
# Build all non-WASM packages
all: build wasm
# Build the specific packages with the default target
build:
	cargo build --exclude selenide-sdk --exclude state --exclude state-sdk --workspace
# Build the WASM packages
wasm:
	cargo build --target wasm32-unknown-unknown --release --package selenide-sdk
	cargo build --target wasm32-unknown-unknown --release --package state
	cargo build --target wasm32-unknown-unknown --release --package state-sdk
ifeq ($(OS),Windows_NT)
	@echo "Compiling example-c on Windows is currently not supported"
else
	$(MAKE) -C contracts/example-c
endif
test:
	cargo test --exclude selenide-sdk --exclude state --exclude state-sdk --workspace
# Build both
all: build wasm
