use std::process::Command;

pub fn compile_to_aot(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !path.ends_with(".wasm") {
        return Err("File is not a wasm file".into());
    }

    let path_without_suffix = path.trim_end_matches(".wasm");
    let new_path = format!("{}_aot.wasm", path_without_suffix);
    Command::new("wasmedge")
        .args(["compile", path, &new_path])
        .status()
        .expect("Failed to compile contract to AOT");

    Ok(())
}
