use crate::FlashbotsError;
use crate::FlashbotsResult;
use crate::types::Bundle;
use ethers::types::H256;
use ethers::types::U64;
use ethers::utils;

/// Validator for Flashbots bundles
pub struct BundleValidator;

impl BundleValidator {
    /// Validates a bundle for correctness and Flashbots compatibility
    ///
    /// # Params
    /// bundle - The bundle to validate
    ///
    /// # Returns
    /// FlashbotsResult<()> - Ok if valid, Err with validation error otherwise
    ///
    /// # Example
    /// ```
    /// let bundle = Bundle {
    ///     txs: vec!["0x02f87401843b9aca00843b9aca0082520894...".to_string()],
    ///     block_number: Some(12345678.into()),
    ///     min_timestamp: None,
    ///     max_timestamp: None,
    ///     reverting_tx_hashes: None,
    /// };
    ///
    /// match BundleValidator::validate_bundle(&bundle) {
    ///     Ok(()) => println!("Bundle is valid"),
    ///     Err(e) => println!("Bundle validation failed: {}", e),
    /// }
    /// ```
    pub fn validate_bundle(bundle: &Bundle) -> FlashbotsResult<()> {
        if bundle.txs.is_empty() {
            return Err(FlashbotsError::ValidationError(
                "Bundle must contain at least one transaction".to_string(),
            ));
        }
        if let (Some(min), Some(max)) = (bundle.min_timestamp, bundle.max_timestamp) {
            if min > max {
                return Err(FlashbotsError::ValidationError(
                    "min_timestamp cannot be greater than max_timestamp".to_string(),
                ));
            }
        }
        for tx in &bundle.txs {
            Self::validate_transaction_format(tx)?;
        }
        Ok(())
    }

    fn validate_transaction_format(tx: &str) -> FlashbotsResult<()> {
        if tx.len() < 20 {
            return Err(FlashbotsError::ValidationError(
                "Transaction data too short".to_string(),
            ));
        }
        if !tx.starts_with("0x") {
            return Err(FlashbotsError::ValidationError(
                "Transaction must start with 0x".to_string(),
            ));
        }
        if let Err(e) = hex::decode(&tx[2..]) {
            return Err(FlashbotsError::ValidationError(format!(
                "Invalid hex in transaction: {}",
                e
            )));
        }
        Ok(())
    }

    /// Validates that the target block number is in the future and within acceptable range
    ///
    /// # Params
    /// current_block - The current block number
    /// target_block - The target block number for bundle inclusion
    ///
    /// # Returns
    /// FlashbotsResult<()> - Ok if valid, Err with validation error otherwise
    ///
    /// # Example
    /// ```
    /// let current_block = U64::from(12345678);
    /// let target_block = U64::from(12345680);
    ///
    /// match BundleValidator::validate_block_number(current_block, target_block) {
    ///     Ok(()) => println!("Block number validation passed"),
    ///     Err(e) => println!("Block number validation failed: {}", e),
    /// }
    /// ```
    pub fn validate_block_number(current_block: U64, target_block: U64) -> FlashbotsResult<()> {
        if target_block <= current_block {
            return Err(FlashbotsError::ValidationError(
                "Target block must be in the future".to_string(),
            ));
        }
        if target_block - current_block > U64::from(25) {
            return Err(FlashbotsError::ValidationError(
                "Target block too far in the future".to_string(),
            ));
        }
        Ok(())
    }

    /// Calculates the Keccak-256 hash of a serialized bundle
    ///
    /// # Params
    /// bundle - The bundle to hash
    ///
    /// # Returns
    /// FlashbotsResult<H256> - The bundle hash if successful
    ///
    /// # Example
    /// ```
    /// let bundle = Bundle {
    ///     txs: vec!["0x02f87401843b9aca00843b9aca0082520894...".to_string()],
    ///     block_number: Some(12345678.into()),
    ///     min_timestamp: None,
    ///     max_timestamp: None,
    ///     reverting_tx_hashes: None,
    /// };
    ///
    /// match BundleValidator::calculate_bundle_hash(&bundle) {
    ///     Ok(hash) => println!("Bundle hash: {:?}", hash),
    ///     Err(e) => println!("Failed to calculate bundle hash: {}", e),
    /// }
    /// ```
    pub fn calculate_bundle_hash(bundle: &Bundle) -> FlashbotsResult<H256> {
        let serialized = serde_json::to_vec(bundle)
            .map_err(|e| FlashbotsError::ValidationError(e.to_string()))?;
        Ok(utils::keccak256(serialized).into())
    }
}
