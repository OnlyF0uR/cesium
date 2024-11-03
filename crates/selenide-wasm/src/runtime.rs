use std::sync::{Arc, Mutex};

use wasmer::{
    imports, wasmparser::Operator, EngineBuilder, Instance, Module, Store, TypedFunction,
};
use wasmer::{CompilerConfig, Function, FunctionEnv, Singlepass, Value};
use wasmer_middlewares::metering::MeteringPoints;
use wasmer_middlewares::{metering::get_remaining_points, Metering};

use crate::env::{ContractDataAccounts, ContractEnv, ContractState};
use crate::errors::RuntimeError;
use crate::functions::debug_log::h_debug_log;
use crate::functions::define_state::h_define_state;
use crate::functions::generate_id::h_gen_id;
use crate::functions::get_account_data::h_get_account_data;
use crate::functions::update_account_data::h_update_account_data;
use crate::functions::get_state::h_get_state;
use crate::functions::write_state::h_write_state;

pub fn execute_function(
    wasm_bytes: &[u8],
    func_name: &str,
    metering_points: u64,
    program_id: &str,
    caller_id: &str,
    contract_state: ContractState,
    params: &[&[u8]],
) -> Result<(i32, MeteringPoints), RuntimeError> {
    let cost_function = |operator: &Operator| -> u64 {
        match operator {
            Operator::LocalGet { .. } | Operator::I32Const { .. } => 1,
            Operator::I32Add { .. } => 2,
            _ => 0,
        }
    };

    let metering = Arc::new(Metering::new(metering_points, cost_function));
    let mut compiler_config = Singlepass::new();
    compiler_config.push_middleware(metering);

    let mut store = Store::new(EngineBuilder::new(compiler_config));
    let module = Module::new(&store, wasm_bytes)?;

    // Contract environment
    let cntr_state: Arc<Mutex<ContractState>> = Arc::new(Mutex::new(contract_state));
    let cntr_data_accounts: Arc<Mutex<ContractDataAccounts>> =
        Arc::new(Mutex::new(ContractDataAccounts::new()));

    let new_offset = params.iter().map(|p| p.len() as u64).sum();
    let cntr_env = FunctionEnv::new(
        &mut store,
        ContractEnv::new(
            program_id,
            caller_id,
            cntr_state,
            cntr_data_accounts,
            new_offset,
        ),
    );

    let import_object = imports! {
      "env" => {
        "h_define_state" => Function::new_typed_with_env(&mut store, &cntr_env, h_define_state),
        "h_get_state" => Function::new_typed_with_env(&mut store, &cntr_env, h_get_state),
        "h_write_state" => Function::new_typed_with_env(&mut store, &cntr_env, h_write_state),
        "h_gen_id" => Function::new_typed_with_env(&mut store, &cntr_env, h_gen_id),
        "h_debug_log" => Function::new_typed_with_env(&mut store, &cntr_env, h_debug_log),
        "h_get_account_data" => Function::new_typed_with_env(&mut store, &cntr_env, h_get_account_data),
        "h_update_account_data" => Function::new_typed_with_env(&mut store, &cntr_env, h_update_account_data),
      }
    };

    let instance = Instance::new(&mut store, &module, &import_object)?;
    let memory = instance.exports.get_memory("memory").unwrap().clone();
    let view = memory.view(&mut store);

    let mut wasm_params: Vec<Value> = Vec::new();

    let mut offset: i32 = 0;
    for param in params {
        // Write the param to memory
        view.write(offset as u64, param)?;

        // For parameter we need to push the offset and length
        let len = param.len() as i32;

        wasm_params.push(Value::I32(offset));
        wasm_params.push(Value::I32(len));

        offset += len;
    }

    let mut_env = cntr_env.as_mut(&mut store);
    mut_env.memory = Some(memory);
    *mut_env.mem_offset.lock().unwrap() = offset as u64;

    if func_name == "initialize" {
        let func: TypedFunction<(), i32> = instance
            .exports
            .get_function("initialize")?
            .typed(&mut store)?;

        let call_result = func.call(&mut store);
        let mp = get_remaining_points(&mut store, &instance);
        if mp == MeteringPoints::Exhausted {
            return Err(RuntimeError::OutOfGas);
        }

        match call_result {
            Ok(_) => Ok((0, mp)),
            Err(e) => Err(e.into()),
        }
    } else {
        let func = instance.exports.get_function(func_name)?;
        let params = [Value::I32(1), Value::I32(2)];

        // TODO: Parse params

        let call_result = func.call(&mut store, &params);
        let mp = get_remaining_points(&mut store, &instance);
        if mp == MeteringPoints::Exhausted {
            return Err(RuntimeError::OutOfGas);
        }

        match call_result {
            Ok(b) => {
                let result = match b[0].i32() {
                    Some(r) => r,
                    None => 0, // We currently assume success, for void compat
                };
                Ok((result, mp))
            }
            Err(e) => Err(e.into()),
        }
    }
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

        // Execution that will run out of gas
        println!("\n === Running out of gas test ===");
        let result = execute_function(
            &wasm_bytes,
            "initialize",
            100,
            "111examplecontract",
            "111exampleuser",
            ContractState::new(),
            &[],
        );
        assert!(result.is_err());
        let result = result.unwrap_err();
        assert_eq!(result.to_string(), "Out of gas");

        // Execution that will succeed
        println!("\n === Running success test ===");
        let result = execute_function(
            &wasm_bytes,
            "initialize",
            1000,
            "111examplecontract",
            "111exampleuser",
            ContractState::new(),
            &[],
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0, 0);
    }
}
