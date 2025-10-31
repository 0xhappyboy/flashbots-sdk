use crate::types::Bundle;
use ethers::types::{H256, U64};

/// A builder for creating `Bundle` instances with a fluent interface.
///
/// # Example
/// ```
/// use bundle_builder::BundleBuilder;
/// use ethers::types::U64;
///
/// let bundle = BundleBuilder::new()
///     .add_transaction("0x1234...".to_string())
///     .block_number(U64::from(12345))
///     .min_timestamp(1625097600)
///     .build();
/// ```
#[derive(Debug)]
pub struct BundleBuilder {
    txs: Vec<String>,
    block_number: Option<U64>,
    min_timestamp: Option<u64>,
    max_timestamp: Option<u64>,
    reverting_tx_hashes: Option<Vec<H256>>,
    replacement_uuid: Option<String>,
}

impl BundleBuilder {
    /// Creates a new `BundleBuilder` with default values.
    ///
    /// # Example
    /// ```
    /// use bundle_builder::BundleBuilder;
    ///
    /// let builder = BundleBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            txs: Vec::new(),
            block_number: None,
            min_timestamp: None,
            max_timestamp: None,
            reverting_tx_hashes: None,
            replacement_uuid: None,
        }
    }

    /// Adds a single transaction to the bundle.
    ///
    /// # Example
    /// ```
    /// use bundle_builder::BundleBuilder;
    ///
    /// let builder = BundleBuilder::new()
    ///     .add_transaction("0xabcd...".to_string());
    /// ```
    pub fn add_transaction(mut self, tx: String) -> Self {
        self.txs.push(tx);
        self
    }

    /// Adds multiple transactions to the bundle.
    ///
    /// # Example
    /// ```
    /// use bundle_builder::BundleBuilder;
    ///
    /// let transactions = vec![
    ///     "0x1234...".to_string(),
    ///     "0x5678...".to_string(),
    /// ];
    /// let builder = BundleBuilder::new()
    ///     .add_transactions(transactions);
    /// ```
    pub fn add_transactions(mut self, transactions: Vec<String>) -> Self {
        self.txs.extend(transactions);
        self
    }

    /// Sets the target block number for the bundle.
    ///
    /// # Example
    /// ```
    /// use bundle_builder::BundleBuilder;
    /// use ethers::types::U64;
    ///
    /// let builder = BundleBuilder::new()
    ///     .block_number(U64::from(12345));
    /// ```
    pub fn block_number(mut self, block_number: U64) -> Self {
        self.block_number = Some(block_number);
        self
    }

    /// Sets the minimum timestamp for the bundle.
    ///
    /// # Example
    /// ```
    /// use bundle_builder::BundleBuilder;
    ///
    /// let builder = BundleBuilder::new()
    ///     .min_timestamp(1625097600);
    /// ```
    pub fn min_timestamp(mut self, timestamp: u64) -> Self {
        self.min_timestamp = Some(timestamp);
        self
    }

    /// Sets the maximum timestamp for the bundle.
    ///
    /// # Example
    /// ```
    /// use bundle_builder::BundleBuilder;
    ///
    /// let builder = BundleBuilder::new()
    ///     .max_timestamp(1625184000);
    /// ```
    pub fn max_timestamp(mut self, timestamp: u64) -> Self {
        self.max_timestamp = Some(timestamp);
        self
    }

    /// Sets the reverting transaction hashes for the bundle.
    ///
    /// # Example
    /// ```
    /// use bundle_builder::BundleBuilder;
    /// use ethers::types::H256;
    ///
    /// let hashes = vec![
    ///     H256::zero(),
    ///     H256::repeat_byte(0xab),
    /// ];
    /// let builder = BundleBuilder::new()
    ///     .reverting_tx_hashes(hashes);
    /// ```
    pub fn reverting_tx_hashes(mut self, hashes: Vec<H256>) -> Self {
        self.reverting_tx_hashes = Some(hashes);
        self
    }

    /// Sets the replacement UUID for the bundle.
    ///
    /// # Example
    /// ```
    /// use bundle_builder::BundleBuilder;
    ///
    /// let builder = BundleBuilder::new()
    ///     .replacement_uuid("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx".to_string());
    /// ```
    pub fn replacement_uuid(mut self, uuid: String) -> Self {
        self.replacement_uuid = Some(uuid);
        self
    }

    /// Builds and returns the final `Bundle` instance.
    ///
    /// # Example
    /// ```
    /// use bundle_builder::BundleBuilder;
    /// use ethers::types::U64;
    ///
    /// let bundle = BundleBuilder::new()
    ///     .add_transaction("0x1234...".to_string())
    ///     .block_number(U64::from(12345))
    ///     .build();
    /// ```
    pub fn build(self) -> Bundle {
        Bundle {
            txs: self.txs,
            block_number: self.block_number,
            min_timestamp: self.min_timestamp,
            max_timestamp: self.max_timestamp,
            reverting_tx_hashes: self.reverting_tx_hashes,
            replacement_uuid: self.replacement_uuid,
        }
    }
}
