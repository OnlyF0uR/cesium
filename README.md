![banner](https://github.com/user-attachments/assets/54d8b664-7266-4d9c-b520-48b161de54f2)

### Dependencies

- LLVM (for compiling rocksdb)
- WasmEdge (for contracts in wasm)

### Programs

Programs, like smart contract, are a set of instructions that act on provided parameters. A program may include state variables that will be stored in a data account. Programs are written in Selenide (.se for source, and .seh for headers). This code gets compiled to an interpretable medium that is validated and if found valid deployed onchain. Presets are a core functionality of the Selenide Compiler to improve DX and security. See examples for more information.
However currently experimenting with using a WASM runtime instead.
