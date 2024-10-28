// runtime.rs

use std::collections::HashMap;
use wasmedge_sdk::{
    params, vm::SyncInst, AsInstance, ImportObjectBuilder, Module, Store, Vm, WasmValue,
};
use wasmedge_sys::WasiModule;

use crate::functions::{get_state, update_state, ContractState};

pub fn execute_contract(
    wasm_bytes: &[u8],
) -> Result<Vec<WasmValue>, Box<dyn std::error::Error + Send + Sync>> {
    let state = ContractState::new();
    let mut wasi_module = WasiModule::create(None, None, None).unwrap();

    let mut import_builder = ImportObjectBuilder::new("extern", state).unwrap();
    import_builder
        .with_func::<i32, ()>("get_state", get_state)
        .unwrap();
    import_builder
        .with_func::<(), i32>("update_state", update_state)
        .unwrap();
    let mut import_object = import_builder.build();

    let mut instances: HashMap<String, &mut dyn SyncInst> = HashMap::new();
    instances.insert(wasi_module.name().to_string(), wasi_module.as_mut());
    instances.insert(import_object.name().unwrap(), &mut import_object);

    let mut vm = Vm::new(Store::new(None, instances).unwrap());

    let module = Module::from_bytes(None, wasm_bytes)?;
    vm.register_module(Some("wasm-app"), module)?;

    // Args can be provided using params macro
    let result = vm.run_func(Some("wasm-app"), "entry_proc", params!())?;
    Ok(result)
}

// write a test that reads the bytes from target/wasm32-wasi/debug/example.wasm and executes execute_contract
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::process::Command;

    fn compile_hello() {
        Command::new("cargo")
            .args([
                "build",
                "--target",
                "wasm32-wasi",
                "--release",
                "--package",
                "hello",
            ])
            .status()
            .expect("Failed to compile hello contract");
    }

    fn compile_to_aot() {
        Command::new("wasmedge")
            .args([
                "compile",
                "../../target/wasm32-wasi/release/hello.wasm",
                "../../target/wasm32-wasi/release/hello_aot.wasm",
            ])
            .status()
            .expect("Failed to compile hello contract to AOT");
    }

    #[test]
    fn test_execute_contract() {
        compile_hello();
        compile_to_aot();

        let mut file = File::open("../../target/wasm32-wasi/release/hello_aot.wasm").unwrap();
        let mut wasm_bytes = Vec::new();
        file.read_to_end(&mut wasm_bytes).unwrap();
        let result = execute_contract(&wasm_bytes).unwrap();

        assert_eq!(result.len(), 0);
    }
}
