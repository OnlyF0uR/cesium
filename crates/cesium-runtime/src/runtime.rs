// runtime.rs

use std::collections::HashMap;
use wasmedge_sdk::{
    params, vm::SyncInst, AsInstance, ImportObjectBuilder, Module, Store, Vm, WasmValue,
};
use wasmedge_sys::WasiModule;

use crate::{
    functions::{h_change_state, h_commit_state, h_get_state, h_write_state_mem},
    state::ContractState,
};

pub fn initialize_contract(
    wasm_bytes: &[u8],
    account_id: &str,
) -> Result<Vec<WasmValue>, Box<dyn std::error::Error + Send + Sync>> {
    let state = ContractState::new(account_id.to_owned());
    let mut wasi_module = WasiModule::create(None, None, None).unwrap();

    let mut import_builder = ImportObjectBuilder::new("env", state).unwrap();
    import_builder
        .with_func::<(i32, i32), i32>("h_get_state", h_get_state)
        .unwrap();
    import_builder
        .with_func::<i32, ()>("h_write_state_mem", h_write_state_mem)
        .unwrap();
    import_builder
        .with_func::<(i32, i32, i32, i32), ()>("h_change_state", h_change_state)
        .unwrap();
    import_builder
        .with_func::<(), ()>("h_commit_state", h_commit_state)
        .unwrap();
    // TODO: Provide more functions that can be used in initialize, like define_state etc.
    let mut import_object = import_builder.build();

    let mut instances: HashMap<String, &mut dyn SyncInst> = HashMap::new();
    instances.insert(wasi_module.name().to_string(), wasi_module.as_mut());
    instances.insert(import_object.name().unwrap(), &mut import_object);

    let mut vm = Vm::new(Store::new(None, instances).unwrap());

    let module = Module::from_bytes(None, wasm_bytes)?;
    vm.register_module(Some("wasm-app"), module)?;

    // Args can be provided using params macro
    let result = vm.run_func(Some("wasm-app"), "initialize", params!())?;
    Ok(result)
}

pub fn execute_contract_function(
    wasm_bytes: &[u8],
    function_name: &str,
    state: ContractState,
    params: Vec<&[u8]>,
) -> Result<Vec<WasmValue>, Box<dyn std::error::Error + Send + Sync>> {
    if function_name.is_empty() || function_name == "initialize" {
        return Err("Invalid function name".into());
    }

    let mut wasi_module = WasiModule::create(None, None, None).unwrap();

    let mut import_builder = ImportObjectBuilder::new("env", state).unwrap();
    import_builder
        .with_func::<(i32, i32), i32>("h_get_state", h_get_state)
        .unwrap();
    import_builder
        .with_func::<i32, ()>("h_write_state_mem", h_write_state_mem)
        .unwrap();
    import_builder
        .with_func::<(i32, i32, i32, i32), ()>("h_change_state", h_change_state)
        .unwrap();
    import_builder
        .with_func::<(), ()>("h_commit_state", h_commit_state)
        .unwrap();
    let mut import_object = import_builder.build();

    let mut instances: HashMap<String, &mut dyn SyncInst> = HashMap::new();
    instances.insert(wasi_module.name().to_string(), wasi_module.as_mut());
    instances.insert(import_object.name().unwrap(), &mut import_object);

    let mut vm = Vm::new(Store::new(None, instances).unwrap());

    let module = Module::from_bytes(None, wasm_bytes)?;
    vm.register_module(Some("wasm-app"), module)?;

    // If we need to supply parameters to the function we must allocate memory for them

    let (extern_instance, _) = vm
        .store_mut()
        .get_named_wasm_and_executor("wasm-app")
        .unwrap();

    let mut mem = extern_instance.get_memory_mut("memory")?;
    let mut wasm_params: Vec<WasmValue> = Vec::new();

    let mut offset = 0;
    for param in &params {
        mem.set_data(param, offset).unwrap();

        wasm_params.push(WasmValue::from_i32(offset as i32));
        wasm_params.push(WasmValue::from_i32(param.len() as i32));

        offset += param.len() as u32;
    }

    // let mut func = extern_instance.get_func_mut(function_name)?;
    // let result = executor.call_func(&mut func, wasm_params).unwrap();

    // Args can be provided using params macro
    let result = vm.run_func(Some("wasm-app"), function_name, wasm_params)?;
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
    fn test_initialize_state_contract() {
        compile("state");
        compile_to_aot("state");

        let mut file = File::open("../../target/wasm32-wasi/release/state_aot.wasm").unwrap();
        let mut wasm_bytes = Vec::new();
        file.read_to_end(&mut wasm_bytes).unwrap();
        let result = initialize_contract(&wasm_bytes, "").unwrap();
        assert_eq!(result.len(), 1);

        let v = result.get(0).unwrap();
        assert_eq!(v.to_i32(), 0);
    }

    #[test]
    fn test_initialize_state_sdk_contract() {
        compile("state-sdk");
        compile_to_aot("state_sdk");

        let mut file = File::open("../../target/wasm32-wasi/release/state_sdk_aot.wasm").unwrap();
        let mut wasm_bytes = Vec::new();
        file.read_to_end(&mut wasm_bytes).unwrap();

        // First initialize the contract
        let result = initialize_contract(&wasm_bytes, "").unwrap();
        assert_eq!(result.len(), 1);

        let v = result.get(0).unwrap();
        assert_eq!(v.to_i32(), 0);

        // Specify the current state prior to running the function
        let mut data: HashMap<String, Vec<u8>> = HashMap::new();
        data.insert("example_str".to_string(), "my_value".as_bytes().to_vec());
        let state = ContractState::new_with_storage("", data);

        // Define the parameters that will be passed in the function
        // Execute the function
        let result = execute_contract_function(
            &wasm_bytes,
            "compare_state",
            state,
            vec!["my_value".as_bytes()],
        )
        .unwrap();

        assert_eq!(result.len(), 1);

        let v = result.get(0).unwrap();
        assert_eq!(v.to_i32(), 0);
    }
}
