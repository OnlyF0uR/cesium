use std::collections::HashMap;

use cesium_material::keys::PublicKeyBytes;

use crate::errors::RegistryError;

#[derive(Debug, Clone)]
pub enum StateValue {
    Uint8(u8),
    Uint128(u128),
    String(String),
    Bool(bool),
    ByteArray(Vec<u8>),
    Address(PublicKeyBytes),
    NumericMap(HashMap<String, u128>),
    AddressMap(HashMap<String, PublicKeyBytes>),
}

#[derive(Debug, Clone)]
pub enum Value {
    Uint8(u8),
    Uint128(u128),
    String(String),
    Bool(bool),
    ByteArray(Vec<u8>),
    Address(PublicKeyBytes),
}

impl Value {
    pub fn as_uint8(&self) -> Option<u8> {
        if let Value::Uint8(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn as_uint128(&self) -> Option<u128> {
        if let Value::Uint128(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        if let Value::String(ref val) = self {
            Some(val)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn as_byte_array(&self) -> Option<&Vec<u8>> {
        if let Value::ByteArray(ref val) = self {
            Some(val)
        } else {
            None
        }
    }

    pub fn as_address(&self) -> Option<&PublicKeyBytes> {
        if let Value::Address(ref val) = self {
            Some(val)
        } else {
            None
        }
    }
}

pub struct ExecutionContext {
    state: HashMap<String, StateValue>, // State variables stored by name
    memory: Vec<Value>,                 // Registers (local variables for function execution)
}

impl ExecutionContext {
    pub fn new_empty() -> Self {
        ExecutionContext {
            state: HashMap::new(),
            memory: Vec::new(),
        }
    }

    pub fn new_with_state(state: HashMap<String, StateValue>) -> Self {
        ExecutionContext {
            state,
            memory: Vec::new(),
        }
    }

    // Function to handle GET_STATE, retrieving state by name and type
    pub fn get_state(&self, key: &str) -> Result<&StateValue, RegistryError> {
        match self.state.get(key) {
            Some(value) => Ok(value),
            None => Err(RegistryError::InvalidStateRegister(key.to_owned())),
        }
    }

    // Function to handle SET_STATE, storing a value in the state
    pub fn set_state(&mut self, key: &str, value: StateValue) -> Result<(), RegistryError> {
        // TODO: Type checking for value and matching against existing state value
        match self.state.get_mut(key) {
            Some(entry) => {
                *entry = value;
                Ok(())
            }
            None => {
                self.state.insert(key.to_owned(), value);
                Ok(())
            }
        }
    }

    pub fn malloc(&mut self, value: Value) -> usize {
        self.memory.push(value);
        self.memory.len() - 1
    }

    pub fn delloc(&mut self, index: usize) -> Result<(), RegistryError> {
        if index < self.memory.len() {
            self.memory.remove(index);
            Ok(())
        } else {
            Err(RegistryError::OutOfBounds(index, self.memory.len()))
        }
    }

    pub fn clear_memory(&mut self) {
        self.memory.clear();
    }
}
