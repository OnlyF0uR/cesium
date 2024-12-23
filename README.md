![banner](https://github.com/user-attachments/assets/16e11c3b-dc01-4924-ba5d-ca28f55d5287)

### Dependencies

- LLVM (for compiling rocksdb)
- Emscripten (optional, for compiling C to wasm)

### Structure

Cesium uses a Directed Acyclic Graph (DAG) for processing transactions asynchronously. Checkpoints (batches of transactions) are validated using a (Delegated) Proof of Stake consensus mechanism.

### Smart contracts

Smart contracts are compiled in WASM and executed in a controlled wasmer virtual sandbox, this runtime is referred to as Selenide. An sdk is provided to improve DX.
