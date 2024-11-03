use clap::Parser;
use std::{fs::File, io::Read};
use wasmer::{Module, Singlepass, Store};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();
    let wasm_bytes = match bytes_from_file(&args.input) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error reading file: {:?}", e);
            return;
        }
    };

    let compiler = Singlepass::default();
    let store = Store::new(compiler);

    let module = match Module::new(&store, &wasm_bytes) {
        Ok(module) => module,
        Err(e) => {
            eprintln!("Error creating module: {:?}", e);
            return;
        }
    };

    let input_path = args.input;
    if !input_path.ends_with(".wasm") {
        eprintln!("Input file must be a .wasm file");
        return;
    }

    let path = match args.output {
        Some(path) => {
            if !path.ends_with(".wasmc") {
                eprintln!("Output file must be a .wasmc file");
                return;
            }
            path
        }
        None => {
            let without_ext = input_path.strip_suffix(".wasm").unwrap();
            format!("{}.wasmc", without_ext)
        }
    };

    match module.serialize_to_file(&path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error serializing module: {:?}", e);
            return;
        }
    };

    println!("Module serialized to: {}", path);
}

fn bytes_from_file(file_path: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut wasm_bytes = Vec::new();
    file.read_to_end(&mut wasm_bytes)?;
    Ok(wasm_bytes)
}
