use ethers::types::{Address, H256, U64, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub txs: Vec<String>,
    pub block_number: Option<U64>,
    pub min_timestamp: Option<u64>,
    pub max_timestamp: Option<u64>,
    pub reverting_tx_hashes: Option<Vec<H256>>,
    pub replacement_uuid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleStats {
    pub is_high_priority: bool,
    pub is_mev: bool,
    pub eligible_after: Option<U64>,
    pub eligible_until: Option<U64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendBundleResponse {
    pub bundle_hash: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleReceipt {
    pub bundle_hash: H256,
    pub timestamp: U64,
    pub block_number: U64,
    pub transactions: Vec<H256>,
    pub state_block_number: U64,
    pub status: String,
    pub mev_reward: Option<U256>,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub is_high_priority: bool,
    pub all_time_mev_reward: U256,
    pub all_time_gas_spent: U256,
    pub all_time_bundles_received: U64,
    pub all_time_valid_bundles_received: U64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateBundleRequest {
    pub bundle: Bundle,
    pub state_block_number: Option<U64>,
    pub parent_block: Option<U64>,
    pub block_number: Option<U64>,
    pub timestamp: Option<u64>,
    pub gas_limit: Option<U64>,
    pub base_fee: Option<U256>,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateBundleResponse {
    pub success: bool,
    pub error: Option<String>,
    pub state_block_number: U64,
    pub gas_used: U64,
    pub mev_reward: Option<U256>,
    pub logs: Vec<String>,
    pub coinbase_diff: U256,
    pub eth_sent_to_coinbase: U256,
    pub gas_fees: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayStatus {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundlePriceResponse {
    pub percentage: Option<f64>,
    pub number: Option<U64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelBundlesRequest {
    pub bundle_hashes: Vec<H256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelBundlesResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPriceResponse {
    pub last_block_gas_price: u64,
    pub safe_low_gas_price: u64,
    pub standard_gas_price: u64,
    pub fast_gas_price: u64,
    pub fastest_gas_price: u64,
    pub block_number: U64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResponse {
    pub block_number: U64,
    pub miner: Address,
    pub base_fee_per_gas: U256,
    pub gas_used: U256,
    pub gas_limit: U256,
    pub timestamp: u64,
    pub bundles: Vec<BundleReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatus {
    pub is_high_priority: bool,
    pub reputation: Option<f64>,
    pub blacklisted: bool,
    pub max_gas_price: Option<U256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayInfo {
    pub name: String,
    pub version: String,
    pub supported_apis: Vec<String>,
    pub network: String,
    pub chain_id: u64,
    pub builder: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleByHashResponse {
    pub bundle: Bundle,
    pub receipt: Option<BundleReceipt>,
}

#[derive(Debug)]
pub enum FlashbotsError {
    HttpError(String),
    JsonError(String),
    ApiError(String),
    ValidationError(String),
    Timeout,
    InvalidResponse(String),
    EthersError(String),
    Error(String),
}

impl From<ethers::providers::ProviderError> for FlashbotsError {
    fn from(err: ethers::providers::ProviderError) -> Self {
        FlashbotsError::EthersError(err.to_string())
    }
}

pub type FlashbotsResult<T> = std::result::Result<T, FlashbotsError>;
