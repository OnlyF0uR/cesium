pub mod utils;

// This serves as the token factory contract, nomisma comes from ancient Greek for
// money/coin/token. Since all tokens are considered coins in cesium, including the native token
// of the blockchain, we shall use this contract to create new tokens, mint new tokens, transfer etc.
// It is to be entirely written without any external dependencies, and thus without sdk. It shall
// be optimized for speed, efficiency, security, and low fees.

extern "C" {
    fn h_gen_id() -> i64;
}

#[no_mangle]
pub unsafe extern "C" fn initialize() -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn create() -> i32 {
    let token_id_result = h_gen_id();
    let (data_ptr, data_len) = utils::unfold_ptr(token_id_result);

    let _token_id = std::slice::from_raw_parts(data_ptr, data_len);
    // sum the token id to a i32

    // create infinite loop

    // TODO: We shall first create a data account for the token mint,
    // the update authority shall be this contract, but the owner minting authority will
    // be set to the caller of this function. The token id will also be used to
    // represent the data account for the token mint.

    // TODO: Create data account for the initial set of tokens

    1
}

#[no_mangle]
pub unsafe extern "C" fn mint_to() -> i32 {
    // TODO: Takes in a parameter of the token id, token id length, amount of mint, recipient
    // and recipient length. Checks if the mint authority is in fact the caller of this function
    // and if so mints the tokens to the recipient
    // also not that on the host, there will be checked if the token already has a token
    // data account for the specific token, and if not it will be automatically created
    2
}

#[no_mangle]
pub unsafe extern "C" fn disable_mint() -> i32 {
    // TODO: Check if the caller actually has the minting authority, if so and we shall
    // set it to None, so that no more tokens can ever be minted
    3
}

#[no_mangle]
pub unsafe extern "C" fn transfer() -> i32 {
    // TODO: Check if the caller is the owner of the token data account
    // Then check if there are sufficient funds
    // and if so send the tokens/coins on their way to the recipient
    // also not that on the host, there will be checked if the token already has a token
    // data account for the specific token, and if not it will be automatically created
    4
}

#[no_mangle]
pub unsafe extern "C" fn transfer_with_moderator() -> i32 {
    // TODO: Check if the caller is the owner of the token data account
    // Then check if there are sufficient funds
    // and if so send the tokens/coins on their way to the recipient
    // and also send a fee to the specified moderator/facilitator
    // also not that on the host, there will be checked if the token already has a token
    // data account for the specific token, and if not it will be automatically created
    5
}

#[no_mangle]
pub unsafe extern "C" fn burn() -> i32 {
    // TODO: Similar to transfer, checks if the token data accounts specified belongs
    // to the caller, and if so sends the tokens into the void
    6
}

#[no_mangle]
pub unsafe extern "C" fn balance_of() -> i32 {
    // TODO: Returns the balance of a token data account
    7
}
