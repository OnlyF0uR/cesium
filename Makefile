.PHONY: all build wasm
all: build wasm
build:
	cargo build --exclude selenide-sdk --exclude state --exclude state-sdk --workspace
wasm:
	cargo build --target wasm32-unknown-unknown --release --package selenide-sdk
	cargo build --target wasm32-unknown-unknown --release --package state
	cargo build --target wasm32-unknown-unknown --release --package state-sdk
wasm-c:
ifeq ($(OS),Windows_NT)
	@echo Compiling example-c on Windows is currently not supported
else
	$(MAKE) -C contracts/example-c
endif
fmt:
	cargo fmt --all -- --check
test:
	cargo test --all-features --exclude selenide-sdk --exclude state --exclude state-sdk --workspace
