use ethers::{
    core::rand,
    signers::{LocalWallet, MnemonicBuilder, coins_bip39::English},
    types::{H256, transaction::eip2718::TypedTransaction},
    utils::{self, rlp},
};

/// Creates a wallet from a private key string
///
/// # Example
/// ```
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let private_key = "4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d";
/// let wallet = create_wallet_from_private_key(private_key)?;
/// # Ok(())
/// # }
/// ```
pub fn create_wallet_from_private_key(
    private_key: &str,
) -> Result<LocalWallet, Box<dyn std::error::Error>> {
    let wallet = private_key.parse::<LocalWallet>()?;
    Ok(wallet)
}

/// Creates a new random wallet
pub fn create_random_wallet() -> LocalWallet {
    LocalWallet::new(&mut rand::thread_rng())
}

/// Creates a wallet from a mnemonic phrase
///
/// # Example
/// ```
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mnemonic = "test test test test test test test test test test test junk";
/// let wallet = create_wallet_from_mnemonic(mnemonic, Some("m/44'/60'/0'/0/0"))?;
/// # Ok(())
/// # }
/// ```
pub fn create_wallet_from_mnemonic(
    mnemonic: &str,
    derivation_path: Option<&str>,
) -> Result<LocalWallet, Box<dyn std::error::Error>> {
    let path = derivation_path.unwrap_or("m/44'/60'/0'/0/0");
    let wallet = MnemonicBuilder::<English>::default()
        .phrase(mnemonic)
        .derivation_path(path)?
        .build()?;
    Ok(wallet)
}

/// Validates the format of a signed transaction
pub fn validate_signed_transaction(tx_hex: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !tx_hex.starts_with("0x") {
        return Err("Transaction must start with 0x".into());
    }
    let tx_bytes = hex::decode(&tx_hex[2..])?;

    if tx_bytes.len() < 1 {
        return Err("Transaction data too short".into());
    }
    Ok(())
}

/// Calculates transaction hash from signed transaction hex
///
/// # Example
/// ```
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let tx_hex = "0x02f86b0180843b9aca00852ecc889a0082520894f39fd6e51aad88f6f4ce6ab8827279cfffb92266880de0b6b3a764000080c080a0f67141f7b16b0b61d1ce4f5c5c6b7b7d7e51a7c5e5b5a5a5a5a5a5a5a5a5a5a5a5a0a05a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a";
/// let hash = TransactionValidator::calculate_transaction_hash(tx_hex)?;
/// println!("Transaction hash: {:?}", hash);
/// # Ok(())
/// # }
/// ```
pub fn calculate_transaction_hash(tx_hex: &str) -> Result<H256, Box<dyn std::error::Error>> {
    let tx_bytes = hex::decode(&tx_hex[2..])?;
    let hash = utils::keccak256(tx_bytes);
    Ok(H256::from_slice(&hash))
}

/// Decodes a signed transaction back to its typed form
pub fn decode_transaction(tx_hex: &str) -> Result<TypedTransaction, Box<dyn std::error::Error>> {
    let tx_bytes = hex::decode(&tx_hex[2..])?;
    let rlp = rlp::Rlp::new(&tx_bytes);
    let (tx, _signature) = TypedTransaction::decode_signed(&rlp)?;
    Ok(tx)
}
