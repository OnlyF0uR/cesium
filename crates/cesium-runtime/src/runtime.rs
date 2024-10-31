// runtime.rs

use std::collections::HashMap;
use wasmedge_sdk::{
    params, vm::SyncInst, AsInstance, ImportObjectBuilder, Module, Store, Vm, WasmValue,
};
use wasmedge_sys::ImportModule;

use crate::{
    data::MAX_MEMORY_OFFSET,
    env::ContractEnv,
    functions::{
        accounts::{
            h_commit_account_data, h_initialize_data_account,
            h_initialize_independent_data_account, h_update_data_account,
        },
        misc::{h_commit_all, h_gen_id},
        state::{h_change_state, h_commit_state, h_define_state, h_get_state},
    },
};

fn create_import_builder(state: ContractEnv) -> ImportModule<ContractEnv> {
    let mut import_builder = ImportObjectBuilder::new("env", state).unwrap();
    import_builder
        .with_func::<i32, ()>("h_define_state", h_define_state)
        .unwrap();
    import_builder
        .with_func::<i32, i64>("h_get_state", h_get_state)
        .unwrap();
    import_builder
        .with_func::<(i32, i32, i32), ()>("h_change_state", h_change_state)
        .unwrap();
    import_builder
        .with_func::<(), ()>("h_commit_state", h_commit_state)
        .unwrap();
    import_builder
        .with_func::<(i32, i32, i32, i32), i32>(
            "h_initialize_data_account",
            h_initialize_data_account,
        )
        .unwrap();
    import_builder
        .with_func::<(i32, i32, i32, i32, i32, i32), i32>(
            "h_initialize_independent_data_account",
            h_initialize_independent_data_account,
        )
        .unwrap();
    import_builder
        .with_func::<(i32, i32, i32, i32), ()>("h_update_data_account", h_update_data_account)
        .unwrap();
    import_builder
        .with_func::<(), ()>("h_commit_account_data", h_commit_account_data)
        .unwrap();
    import_builder
        .with_func::<(), ()>("h_commit_all", h_commit_all)
        .unwrap();
    import_builder
        .with_func::<(), i64>("h_gen_id", h_gen_id)
        .unwrap();
    // TODO: Provide more functions that can be used in initialize, like define_state etc.
    import_builder.build()
}

pub fn initialize_contract(
    wasm_bytes: &[u8],
    account_id: &str,
    caller_id: &str,
) -> Result<Vec<WasmValue>, Box<dyn std::error::Error + Send + Sync>> {
    let env = ContractEnv::new(account_id, caller_id);

    let mut import_object: ImportModule<ContractEnv> = create_import_builder(env);

    let mut instances: HashMap<String, &mut dyn SyncInst> = HashMap::new();
    instances.insert(import_object.name().unwrap(), &mut import_object);

    let mut vm = Vm::new(Store::new(None, instances).unwrap());

    let module = Module::from_bytes(None, wasm_bytes)?;
    vm.register_module(Some("wasm-app"), module)?;

    let result = vm.run_func(Some("wasm-app"), "initialize", params!())?;
    Ok(result)
}

pub fn execute_contract_function(
    wasm_bytes: &[u8],
    function_name: &str,
    mut env: ContractEnv,
    params: Vec<&[u8]>,
) -> Result<Vec<WasmValue>, Box<dyn std::error::Error + Send + Sync>> {
    if function_name.is_empty() || function_name == "initialize" {
        return Err("Invalid function name".into());
    }

    // Based on the params we need to already set an offset here
    env.mem_offset = params.iter().map(|p| p.len() as u32).sum();

    let mut import_object: ImportModule<ContractEnv> = create_import_builder(env);

    let mut instances: HashMap<String, &mut dyn SyncInst> = HashMap::new();
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
        if offset + param.len() as u32 > MAX_MEMORY_OFFSET {
            return Err("Memory overflow".into());
        }

        mem.set_data(param, offset).unwrap();

        wasm_params.push(WasmValue::from_i32(offset as i32));
        wasm_params.push(WasmValue::from_i32(param.len() as i32));

        offset += param.len() as u32;
    }

    // let mut func = extern_instance.get_func_mut(function_name)?;
    // let result = executor.call_func(&mut func, wasm_params).unwrap();

    let result = vm.run_func(Some("wasm-app"), function_name, wasm_params)?;
    Ok(result)
}

// write a test that reads the bytes from target/wasm32-wasi/debug/example.wasm and executes execute_contract
#[cfg(test)]
mod tests {
    use crate::bytecode::aot::compile_to_aot;

    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::process::Command;

    fn compile(package: &str) {
        Command::new("cargo")
            .args([
                "build",
                "--target",
                "wasm32-unknown-unknown",
                "--release",
                "--package",
                package,
            ])
            .status()
            .expect("Failed to compile contract");

        let package = package.replace("-", "_");
        if let Err(e) = compile_to_aot(&format!(
            "../../target/wasm32-unknown-unknown/release/{}.wasm",
            package
        )) {
            panic!("Failed to compile to AOT: {:?}", e);
        }
    }

    #[test]
    fn test_initialize_state_contract() {
        compile("state");

        let mut file =
            File::open("../../target/wasm32-unknown-unknown/release/state_aot.wasm").unwrap();
        let mut wasm_bytes = Vec::new();
        file.read_to_end(&mut wasm_bytes).unwrap();
        let result =
            initialize_contract(&wasm_bytes, "111examplecontract", "111exampleuser").unwrap();
        assert_eq!(result.len(), 1);

        let v = result.get(0).unwrap();
        assert_eq!(v.to_i32(), 0);
    }

    #[test]
    fn test_initialize_state_sdk_contract() {
        compile("state-sdk");

        let mut file =
            File::open("../../target/wasm32-unknown-unknown/release/state_sdk_aot.wasm").unwrap();
        let mut wasm_bytes = Vec::new();
        file.read_to_end(&mut wasm_bytes).unwrap();

        // First initialize the contract
        let result =
            initialize_contract(&wasm_bytes, "111examplecontract", "111exampleuser").unwrap();
        assert_eq!(result.len(), 1);

        let v = result.get(0).unwrap();
        assert_eq!(v.to_i32(), 0);

        // Specify the current state prior to running the function
        let data = vec!["my_value".as_bytes().to_vec()]; // "my_value" on index 0
        let state = ContractEnv::new_with_state("111examplecontract", "111exampleuser", data);

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
