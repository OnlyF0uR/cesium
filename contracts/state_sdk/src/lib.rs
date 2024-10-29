use cesium_contract_sdk::state::State;

#[no_mangle]
pub extern "C" fn entry_proc() -> i32 {
    let state = State::new();

    let value = "my_value";

    // Store raw string data
    if let Err(_) = state.set("my_key", value.as_bytes()) {
        return 1;
    }

    // Retrieve raw string data
    match state.get("my_key") {
        Ok(Some(v)) => {
            let s = std::str::from_utf8(&v).unwrap();
            if s != value {
                println!("State value is {} (expected: my_value)", s);
                return 1;
            }
        }
        Ok(None) => println!("Key not found"),
        Err(_) => return 1,
    }

    0
}
