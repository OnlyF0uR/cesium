use std::{fs::File, io::Write, path::PathBuf};

use cesium_crypto::keys::Account;

fn handle_account(cesium_dir: &PathBuf) -> Account {
    let account_sk_path = cesium_dir.join("account.sk");
    let account_pk_path = cesium_dir.join("account.pk");

    if !account_sk_path.exists() && !account_pk_path.exists() {
        let account = Account::create();

        let (pk, sk) = match account.to_bytes() {
            Ok(bytes) => bytes,
            Err(e) => {
                panic!("Error: Unable to serialize account: {}", e);
            }
        };

        let mut sk_file =
            File::create(&account_sk_path).expect("Error: Unable to create secret key file");
        sk_file
            .write_all(sk)
            .expect("Error: Unable to write secret key to file");

        let mut pk_file =
            File::create(&account_pk_path).expect("Error: Unable to create public key file");
        pk_file
            .write_all(pk)
            .expect("Error: Unable to write public key to file");

        println!("Account created and saved to disk");
        return account;
    } else if !account_sk_path.exists() || !account_pk_path.exists() {
        panic!("Error: Account secret key or public key not found");
    }

    let sk_bytes = match std::fs::read(account_sk_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            panic!("Error: Unable to read account secret key file: {}", e);
        }
    };

    let pk_bytes = match std::fs::read(account_pk_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            panic!("Error: Unable to read account public key file: {}", e);
        }
    };

    let account = match Account::from(&pk_bytes, &sk_bytes) {
        Ok(account) => account,
        Err(e) => {
            panic!("Error: Unable to create account from keys: {}", e);
        }
    };

    account
}

fn main() {
    let home_dir = match dirs::home_dir() {
        Some(path) => path,
        None => {
            println!("Error: Unable to get home directory path");
            return;
        }
    };

    let cesium_dir = home_dir.join(".cesium");
    if !cesium_dir.exists() {
        match std::fs::create_dir(&cesium_dir) {
            Ok(_) => {
                println!("Created Cesium directory at: {:?}", cesium_dir);
            }
            Err(e) => {
                println!("Error: Unable to create Cesium directory: {}", e);
                return;
            }
        }
    }

    let account = handle_account(&cesium_dir);

    println!(
        r#"
   ____ _____ ____ ___ _   _ __  __ 
  / ___| ____/ ___|_ _| | | |  \/  |
 | |   |  _| \___ \| || | | | |\/| |
 | |___| |___ ___) | || |_| | |  | |
  \____|_____|____/___|\___/|_|  |_|
                                    
Address: {}"#,
        account.to_public_key_readable()
    );
}
