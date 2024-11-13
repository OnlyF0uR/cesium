use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New,
    Balance { account: Option<String> },
    Deploy { wasm_file: String },
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::New => {
            println!("Creating new account");
        }
        Commands::Balance { account } => {
            if let Some(account) = account {
                println!("Checking balance for account: {}", account);
            } else {
                println!("Checking balance for default account");
            }
        }
        Commands::Deploy { wasm_file } => {
            println!("Deploying contract from file: {}", wasm_file);
        }
    }
}
