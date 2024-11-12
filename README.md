![banner](https://github.com/user-attachments/assets/16e11c3b-dc01-4924-ba5d-ca28f55d5287)

### Dependencies

- LLVM (for compiling rocksdb)
- Emscripten (optional, for compiling C to wasm)

### Smart contracts

Smart contracts are written in WASM and executed in a control wasmer virtual machine named Selenide.

### Note of Caution

The current implementation, specifically the current cryptography (cesium-crypto), is highly experimental and not yet audited for security. While the SPHINX+ and Keccak support is somewhat robust and based on established practices, the bulletsproofs and (non-)interactive zero-knowledge tranformations are merely based on established theory.
