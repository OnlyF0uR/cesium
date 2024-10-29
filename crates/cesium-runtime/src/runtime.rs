// runtime.rs

use std::collections::HashMap;
use wasmedge_sdk::{
    params, vm::SyncInst, AsInstance, ImportObjectBuilder, Module, Store, Vm, WasmValue,
};
use wasmedge_sys::WasiModule;

use crate::functions::{h_get_state, h_write_state_mem, ContractState};

pub fn execute_contract(
    wasm_bytes: &[u8],
) -> Result<Vec<WasmValue>, Box<dyn std::error::Error + Send + Sync>> {
    let state = ContractState::new();
    let mut wasi_module = WasiModule::create(None, None, None).unwrap();

    let mut import_builder = ImportObjectBuilder::new("env", state).unwrap();
    import_builder
        .with_func::<(i32, i32), i32>("h_get_state", h_get_state)
        .unwrap();
    import_builder
        .with_func::<i32, ()>("h_write_state_mem", h_write_state_mem)
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

    fn compile(package: &str) {
        Command::new("cargo")
            .args([
                "build",
                "--target",
                "wasm32-wasi",
                "--release",
                "--package",
                package,
            ])
            .status()
            .expect("Failed to compile contract");
    }

    fn compile_to_aot(package: &str) {
        Command::new("wasmedge")
            .args([
                "compile",
                &format!("../../target/wasm32-wasi/release/{}.wasm", package),
                &format!("../../target/wasm32-wasi/release/{}_aot.wasm", package),
            ])
            .status()
            .expect("Failed to compile contract to AOT");
    }

    #[test]
    fn test_hello_contract() {
        compile("hello");
        compile_to_aot("hello");

        let mut file = File::open("../../target/wasm32-wasi/release/hello_aot.wasm").unwrap();
        let mut wasm_bytes = Vec::new();
        file.read_to_end(&mut wasm_bytes).unwrap();
        let result = execute_contract(&wasm_bytes).unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_get_state_contract() {
        compile("get-state");
        compile_to_aot("get_state");

        let mut file = File::open("../../target/wasm32-wasi/release/get_state_aot.wasm").unwrap();
        let mut wasm_bytes = Vec::new();
        file.read_to_end(&mut wasm_bytes).unwrap();
        let result = execute_contract(&wasm_bytes).unwrap();

        assert_eq!(result.len(), 0);
    }
}
