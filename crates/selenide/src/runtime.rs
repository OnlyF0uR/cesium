use std::sync::{Arc, Mutex};

use wasmer::{
    imports, wasmparser::Operator, Cranelift, EngineBuilder, Instance, Module, Store, TypedFunction,
};
use wasmer::{CompilerConfig, Function, FunctionEnv};
use wasmer_middlewares::metering::MeteringPoints;
use wasmer_middlewares::{metering::get_remaining_points, Metering};

use crate::env::{ContractDataAccounts, ContractEnv, ContractState};
use crate::errors::RuntimeError;
use crate::functions::change_state::h_change_state;
use crate::functions::commit_state::h_commit_state;
use crate::functions::define_state::h_define_state;
use crate::functions::get_state::h_get_state;

pub fn initialize_function(
    wasm_bytes: &[u8],
    metering_points: u64,
    program_id: &str,
    caller_id: &str,
) -> Result<(i32, MeteringPoints), RuntimeError> {
    let cost_function = |operator: &Operator| -> u64 {
        match operator {
            Operator::LocalGet { .. } | Operator::I32Const { .. } => 1,
            Operator::I32Add { .. } => 2,
            _ => 0,
        }
    };

    let metering = Arc::new(Metering::new(metering_points, cost_function));
    let mut compiler_config = Cranelift::default();
    compiler_config.push_middleware(metering);

    let mut store = Store::new(EngineBuilder::new(compiler_config));
    let module = Module::new(&store, wasm_bytes)?;

    // Contract environment
    let cntr_state: Arc<Mutex<ContractState>> = Arc::new(Mutex::new(ContractState::new()));
    let cntr_data_accounts: Arc<Mutex<ContractDataAccounts>> =
        Arc::new(Mutex::new(ContractDataAccounts::new()));

    let cntr_env = FunctionEnv::new(
        &mut store,
        ContractEnv::new(program_id, caller_id, cntr_state, cntr_data_accounts),
    );

    let import_object = imports! {
      "env" => {
        "h_define_state" => Function::new_typed_with_env(&mut store, &cntr_env, h_define_state),
        "h_get_state" => Function::new_typed_with_env(&mut store, &cntr_env, h_get_state),
        "h_change_state" => Function::new_typed_with_env(&mut store, &cntr_env, h_change_state),
        "h_commit_state" => Function::new_typed_with_env(&mut store, &cntr_env, h_commit_state),
      }
    };

    let instance = Instance::new(&mut store, &module, &import_object)?;
    let memory = instance.exports.get_memory("memory").unwrap().clone();
    cntr_env.as_mut(&mut store).memory = Some(memory);

    let func: TypedFunction<(), i32> = instance
        .exports
        .get_function("initialize")?
        .typed(&mut store)?;

    let result: i32;
    match func.call(&mut store) {
        Ok(r) => {
            println!("Result: {}", r);
            result = r;
        }
        Err(e) => {
            println!("Error2: {:?}", e);
            return Err(e.into());
        }
    }

    let mp = get_remaining_points(&mut store, &instance);
    Ok((result, mp))
}

pub fn execute_function(
    wasm_bytes: &[u8],
    func_name: &str,
    metering_points: u64,
    program_id: &str,
    caller_id: &str,
    contract_state: ContractState,
) -> Result<(i32, MeteringPoints), RuntimeError> {
    let cost_function = |operator: &Operator| -> u64 {
        match operator {
            Operator::LocalGet { .. } | Operator::I32Const { .. } => 1,
            Operator::I32Add { .. } => 2,
            _ => 0,
        }
    };

    let metering = Arc::new(Metering::new(metering_points, cost_function));
    let mut compiler_config = Cranelift::default();
    compiler_config.push_middleware(metering);

    let mut store = Store::new(EngineBuilder::new(compiler_config));
    let module = Module::new(&store, wasm_bytes)?;

    // Contract environment
    let cntr_state: Arc<Mutex<ContractState>> = Arc::new(Mutex::new(contract_state));
    let cntr_data_accounts: Arc<Mutex<ContractDataAccounts>> =
        Arc::new(Mutex::new(ContractDataAccounts::new()));

    let cntr_env = FunctionEnv::new(
        &mut store,
        ContractEnv::new(program_id, caller_id, cntr_state, cntr_data_accounts),
    );

    let import_object = imports! {
      "env" => {
        "h_define_state" => Function::new_typed_with_env(&mut store, &cntr_env, h_define_state),
        "h_get_state" => Function::new_typed_with_env(&mut store, &cntr_env, h_get_state),
        "h_change_state" => Function::new_typed_with_env(&mut store, &cntr_env, h_change_state),
        "h_commit_state" => Function::new_typed_with_env(&mut store, &cntr_env, h_commit_state),
      }
    };

    let instance = Instance::new(&mut store, &module, &import_object)?;
    let memory = instance.exports.get_memory("memory").unwrap().clone();
    cntr_env.as_mut(&mut store).memory = Some(memory);

    let func: TypedFunction<(), i32> = instance
        .exports
        .get_function(func_name)?
        .typed(&mut store)?;

    let result: i32;
    println!("Executing function: {}", func_name);
    match func.call(&mut store) {
        Ok(r) => {
            println!("Result: {}", r);
            result = r;
        }
        Err(e) => {
            println!("Error2: {:?}", e);
            return Err(e.into());
        }
    }

    let mp = get_remaining_points(&mut store, &instance);
    Ok((result, mp))
}

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
                "wasm32-unknown-unknown",
                "--release",
                "--package",
                package,
            ])
            .status()
            .expect("Failed to compile contract");
    }

    #[test]
    fn test_initialize_state_contract() {
        compile("state");

        let mut file =
            File::open("../../target/wasm32-unknown-unknown/release/state.wasm").unwrap();
        let mut wasm_bytes = Vec::new();
        file.read_to_end(&mut wasm_bytes).unwrap();

        let result = initialize_function(&wasm_bytes, 10, "111examplecontract", "111exampleuser");
        assert!(result.is_ok());
        let result = result.unwrap();

        println!("Remaining points: {:?}", result.1);
        assert_eq!(result.0, 0);
    }
}
