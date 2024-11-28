use std::{fs::File, io::Write, path::PathBuf, sync::Arc};

use cesium_crypto::mldsa::keypair::{SignerPair, ViewOperations};
use cesium_nucleus::graph::mempool::Graph;
use cesium_rpc::start_rpc;

fn handle_account(cesium_dir: &PathBuf) -> SignerPair {
    let account_sk_path = cesium_dir.join("account.sk");
    let account_pk_path = cesium_dir.join("account.pk");

    if !account_sk_path.exists() && !account_pk_path.exists() {
        let account = SignerPair::create();

        let (pk, sk) = account.to_bytes();

        let mut sk_file = File::create(&account_sk_path).expect("Unable to create secret key file");
        sk_file
            .write_all(&sk)
            .expect("Unable to write secret key to file");

        let mut pk_file = File::create(&account_pk_path).expect("Unable to create public key file");
        pk_file
            .write_all(&pk)
            .expect("Unable to write public key to file");

        println!("Account created and saved to disk");
        return account;
    } else if !account_sk_path.exists() || !account_pk_path.exists() {
        panic!("Account secret key or public key not found");
    }

    let sk_bytes = match std::fs::read(account_sk_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            panic!("Unable to read account secret key file: {}", e);
        }
    };

    let pk_bytes = match std::fs::read(account_pk_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            panic!("Unable to read account public key file: {}", e);
        }
    };

    let account = match SignerPair::from_bytes(&pk_bytes, &sk_bytes) {
        Ok(account) => account,
        Err(e) => {
            panic!("Unable to create account from keys: {}", e);
        }
    };

    account
}

#[tokio::main]
async fn main() {
    let home_dir = match dirs::home_dir() {
        Some(path) => path,
        None => {
            println!("Unable to get home directory path");
            return;
        }
    };

    let cesium_dir = home_dir.join(".cesium");
    if !cesium_dir.exists() {
        std::fs::create_dir(&cesium_dir).expect("Unable to create Cesium directory");
    }

    let account = handle_account(&cesium_dir);
    let account_da = account.get_da();

    println!(
        r#"
   ____ _____ ____ ___ _   _ __  __ 
  / ___| ____/ ___|_ _| | | |  \/  |
 | |   |  _| \___ \| || | | | |\/| |
 | |___| |___ ___) | || |_| | |  | |
  \____|_____|____/___|\___/|_|  |_|
                                    
Address: {}"#,
        account_da.as_str()
    );

    // Get a dag instance
    let acc = Box::leak(Box::new(account));
    let dag = Arc::new(Graph::default(acc));

    // This will also spawn a tokio process
    let url = start_rpc(&dag).await.unwrap();
    println!(
        "RPC server started at: {}",
        url.split("://").collect::<Vec<&str>>()[1]
    );

    // Now we just need to keep this process running
    // TODO: Handle this properly
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
