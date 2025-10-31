use crate::global::{GOERLI_TEST_URL, MAIN_URL, SEPOLIA_TEST_URL};
use crate::types::{
    BlockResponse, Bundle, BundleByHashResponse, BundlePriceResponse, BundleReceipt, BundleStats,
    CancelBundlesRequest, CancelBundlesResponse, FlashbotsError, GasPriceResponse, RelayInfo,
    SimulateBundleResponse, UserStats, UserStatus,
};
use crate::types::{FlashbotsResult, SendBundleResponse, SimulateBundleRequest};
use ethers::core::k256::elliptic_curve::rand_core::block;
use ethers::types::{Address, H256, U64};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
pub mod bundler;
pub mod global;
pub mod tool;
pub mod transaction;
pub mod types;
pub mod validator;

/// Configuration for Flashbots client
#[derive(Debug, Clone)]
pub struct FlashbotsClientConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
}

impl Default for FlashbotsClientConfig {
    fn default() -> Self {
        Self {
            base_url: MAIN_URL.to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_seconds: 2,
        }
    }
}

/// Main client for interacting with Flashbots relay
#[derive(Debug, Clone)]
pub struct FlashbotsClient {
    client: Client,
    config: FlashbotsClientConfig,
}

impl FlashbotsClient {
    /// Creates a new Flashbots client with custom configuration
    pub fn new(config: FlashbotsClientConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Creates a new client configured for Mainnet
    pub fn new_mainnet() -> Self {
        Self::new(FlashbotsClientConfig {
            base_url: MAIN_URL.to_string(),
            ..Default::default()
        })
    }

    /// Creates a new client configured for Goerli testnet
    pub fn new_goerli() -> Self {
        Self::new(FlashbotsClientConfig {
            base_url: GOERLI_TEST_URL.to_string(),
            ..Default::default()
        })
    }

    /// Creates a new client configured for Sepolia testnet
    pub fn new_sepolia() -> Self {
        Self::new(FlashbotsClientConfig {
            base_url: SEPOLIA_TEST_URL.to_string(),
            ..Default::default()
        })
    }

    /// Sets maximum retry attempts for operations
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    /// Sets request timeout in seconds
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.config.timeout_seconds = timeout_seconds;
        self
    }

    /// Sends a bundle to Flashbots relay
    ///
    /// # Example
    /// ```
    /// use flashbots_rs::{FlashbotsClient, Bundle};
    ///
    /// let client = FlashbotsClient::new_mainnet();
    /// let bundle = Bundle::default(); // Your bundle construction
    /// let response = client.send_bundle(bundle).await?;
    /// println!("Bundle sent with hash: {:?}", response.bundle_hash);
    /// ```
    pub async fn send_bundle(&self, bundle: Bundle) -> FlashbotsResult<SendBundleResponse> {
        let url = format!("{}/", self.config.base_url);
        self.post(&url, &bundle).await
    }

    /// Sends a bundle with automatic retry logic
    pub async fn send_bundle_with_retry(
        &self,
        bundle: Bundle,
    ) -> FlashbotsResult<SendBundleResponse> {
        self.retry_operation(|| self.send_bundle(bundle.clone()))
            .await
    }

    /// Simulates bundle execution without submitting to the network
    ///
    /// # Example
    /// ```
    /// use flashbots_rs::{FlashbotsClient, SimulateBundleRequest};
    ///
    /// let client = FlashbotsClient::new_mainnet();
    /// let simulation = SimulateBundleRequest::default(); // Your simulation parameters
    /// let result = client.simulate_bundle(simulation).await?;
    /// println!("Simulation success: {}, Gas used: {}", result.success, result.gas_used);
    /// ```
    pub async fn simulate_bundle(
        &self,
        simulation: SimulateBundleRequest,
    ) -> FlashbotsResult<SimulateBundleResponse> {
        let url = format!("{}/simulate", self.config.base_url);
        self.post(&url, &simulation).await
    }

    /// Simulates bundle with automatic retry logic
    pub async fn simulate_bundle_with_retry(
        &self,
        simulation: SimulateBundleRequest,
    ) -> FlashbotsResult<SimulateBundleResponse> {
        self.retry_operation(|| self.simulate_bundle(simulation.clone()))
            .await
    }

    /// Gets bundle statistics for a specific block
    pub async fn get_bundle_stats(
        &self,
        bundle_hash: H256,
        block_number: U64,
    ) -> FlashbotsResult<BundleStats> {
        let url = format!(
            "{}/bundleStats/0x{}/{}",
            self.config.base_url,
            hex::encode(bundle_hash.as_bytes()),
            block_number
        );
        self.get(&url).await
    }

    /// Gets bundle receipt by hash
    pub async fn get_bundle_receipt(&self, bundle_hash: H256) -> FlashbotsResult<BundleReceipt> {
        let url = format!(
            "{}/bundles/0x{}",
            self.config.base_url,
            hex::encode(bundle_hash.as_bytes())
        );
        self.get(&url).await
    }

    /// Gets user statistics for a specific address and block
    pub async fn get_user_stats(
        &self,
        address: Address,
        block_number: U64,
    ) -> FlashbotsResult<UserStats> {
        let url = format!(
            "{}/userStats/{}/{}",
            self.config.base_url, address, block_number
        );
        self.get(&url).await
    }

    /// Checks relay health status
    ///
    /// # Example
    /// ```
    /// use flashbots_rs::FlashbotsClient;
    ///
    /// let client = FlashbotsClient::new_mainnet();
    /// let is_healthy = client.get_health().await?;
    /// println!("Relay is healthy: {}", is_healthy);
    /// ```
    pub async fn get_health(&self) -> FlashbotsResult<bool> {
        let url = format!("{}/health", self.config.base_url);
        let response: Value = self.get(&url).await?;
        Ok(response
            .get("status")
            .and_then(|s| s.as_str())
            .map(|s| s == "healthy")
            .unwrap_or(false))
    }

    /// Gets bundle price for a specific block
    pub async fn get_bundle_price(
        &self,
        block_number: U64,
    ) -> FlashbotsResult<BundlePriceResponse> {
        let url = format!("{}/bundlePrice/{}", self.config.base_url, block_number);
        self.get(&url).await
    }

    /// Waits for bundle to be included in a block
    ///
    /// # Example
    /// ```
    /// use flashbots_rs::FlashbotsClient;
    /// use ethers::types::H256;
    ///
    /// let client = FlashbotsClient::new_mainnet();
    /// let bundle_hash = H256::zero(); // Your actual bundle hash
    /// let receipt = client.wait_for_bundle_inclusion(bundle_hash, 10).await?;
    /// match receipt {
    ///     Some(receipt) => println!("Bundle included: {:?}", receipt),
    ///     None => println!("Bundle not included within timeout"),
    /// }
    /// ```
    pub async fn wait_for_bundle_inclusion(
        &self,
        bundle_hash: H256,
        timeout_blocks: u64,
    ) -> FlashbotsResult<Option<BundleReceipt>> {
        let mut current_wait = 0;
        while current_wait < timeout_blocks {
            match self.get_bundle_receipt(bundle_hash).await {
                Ok(receipt) => return Ok(Some(receipt)),
                Err(FlashbotsError::ApiError(e))
                    if e.contains("not found") || e.contains("Bundle not found") =>
                {
                    current_wait += 1;
                    log::info!(
                        "Bundle not yet included, waiting... (attempt {}/{})",
                        current_wait,
                        timeout_blocks
                    );
                    sleep(Duration::from_secs(12)).await;
                }
                Err(e) => return Err(e),
            }
        }

        log::warn!("Bundle not included within {} blocks", timeout_blocks);
        Ok(None)
    }

    /// Sends bundle and waits for inclusion
    ///
    /// # Example
    /// ```
    /// use flashbots_rs::{FlashbotsClient, Bundle};
    ///
    /// let client = FlashbotsClient::new_mainnet();
    /// let bundle = Bundle::default(); // Your bundle construction
    /// let receipt = client.send_and_wait_for_bundle(bundle, 10).await?;
    /// match receipt {
    ///     Some(receipt) => println!("Bundle successfully included: {:?}", receipt),
    ///     None => println!("Bundle not included within timeout"),
    /// }
    /// ```
    pub async fn send_and_wait_for_bundle(
        &self,
        bundle: Bundle,
        timeout_blocks: u64,
    ) -> FlashbotsResult<Option<BundleReceipt>> {
        let response = self.send_bundle_with_retry(bundle).await?;
        log::info!("Bundle sent successfully, hash: {:?}", response.bundle_hash);
        self.wait_for_bundle_inclusion(response.bundle_hash, timeout_blocks)
            .await
    }

    /// Simulates bundle and sends it if simulation is successful
    pub async fn simulate_and_send_bundle(
        &self,
        bundle: Bundle,
        simulation_params: SimulateBundleRequest,
    ) -> FlashbotsResult<Option<SendBundleResponse>> {
        let simulation_result = self.simulate_bundle_with_retry(simulation_params).await?;
        if !simulation_result.success {
            log::warn!("Bundle simulation failed: {:?}", simulation_result.error);
            return Ok(None);
        }
        log::info!(
            "Bundle simulation successful: gas_used={}, mev_reward={:?}",
            simulation_result.gas_used,
            simulation_result.mev_reward
        );
        let response = self.send_bundle_with_retry(bundle).await?;
        Ok(Some(response))
    }

    /// Sends multiple bundles in sequence
    pub async fn send_bundles(
        &self,
        bundles: Vec<Bundle>,
    ) -> FlashbotsResult<Vec<FlashbotsResult<SendBundleResponse>>> {
        let mut results = Vec::new();
        for bundle in bundles {
            let result = self.send_bundle_with_retry(bundle).await;
            results.push(result);
        }
        Ok(results)
    }

    async fn post<T: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> FlashbotsResult<R> {
        let response = self
            .client
            .post(url)
            .timeout(Duration::from_secs(self.config.timeout_seconds))
            .json(body)
            .send()
            .await
            .map_err(|e| FlashbotsError::Error(format!("{:?}", e)))?;
        self.handle_response(response).await
    }

    async fn get<R: serde::de::DeserializeOwned>(&self, url: &str) -> FlashbotsResult<R> {
        let response = self
            .client
            .get(url)
            .timeout(Duration::from_secs(self.config.timeout_seconds))
            .send()
            .await
            .map_err(|e| FlashbotsError::Error(format!("{:?}", e)))?;
        self.handle_response(response).await
    }

    async fn handle_response<R: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> FlashbotsResult<R> {
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            let error_msg = format!("HTTP {}: {}", status, error_text);

            log::error!("API error: {}", error_msg);
            return Err(FlashbotsError::ApiError(error_msg));
        }
        let result: R = response
            .json()
            .await
            .map_err(|e| FlashbotsError::Error(format!("{:?}", e)))?;
        Ok(result)
    }

    async fn retry_operation<F, T, Fut>(&self, mut operation: F) -> FlashbotsResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = FlashbotsResult<T>>,
    {
        let mut last_error = None;
        for attempt in 0..self.config.max_retries {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        log::info!("Operation succeeded after {} retries", attempt);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);
                    log::warn!(
                        "Operation failed (attempt {}): {:?}",
                        attempt + 1,
                        last_error
                    );
                    if attempt < self.config.max_retries - 1 {
                        log::info!("Retrying in {} seconds...", self.config.retry_delay_seconds);
                        sleep(Duration::from_secs(self.config.retry_delay_seconds)).await;
                    }
                }
            }
        }
        let final_error = last_error.unwrap();
        log::error!(
            "Operation failed after {} retries: {:?}",
            self.config.max_retries,
            final_error
        );
        Err(final_error)
    }

    /// Returns current client configuration
    pub fn config(&self) -> &FlashbotsClientConfig {
        &self.config
    }

    /// Creates a new client instance with different configuration
    pub fn with_config(&self, config: FlashbotsClientConfig) -> Self {
        Self::new(config)
    }

    /// Send the raw transaction packet (low-level API)
    pub async fn send_raw_bundle(&self, raw_bundle: Value) -> FlashbotsResult<SendBundleResponse> {
        let url = format!("{}/", self.config.base_url);
        self.post(&url, &raw_bundle).await
    }

    /// Cancel submitted transaction package
    ///
    /// # Example
    /// ```
    /// let client = FlashbotsClient::new_mainnet();
    /// let bundle_hashes = vec![H256::zero()]; // Transaction package hash
    /// let result = client.cancel_bundles(bundle_hashes).await?;
    /// println!("Cancel result: {}", result.success);
    /// ```
    pub async fn cancel_bundles(
        &self,
        bundle_hashes: Vec<H256>,
    ) -> FlashbotsResult<CancelBundlesResponse> {
        let url = format!("{}/cancelBundles", self.config.base_url);
        let request = CancelBundlesRequest { bundle_hashes };
        self.post(&url, &request).await
    }

    /// Retrieve transaction package details and receipts using transaction package hash.
    pub async fn get_bundle_by_hash(
        &self,
        bundle_hash: H256,
    ) -> FlashbotsResult<BundleByHashResponse> {
        let url = format!(
            "{}/bundleByHash/0x{}",
            self.config.base_url,
            hex::encode(bundle_hash.as_bytes())
        );
        self.get(&url).await
    }

    /// Get detailed information of a specified block
    ///
    /// # Example
    /// ```
    /// let client = FlashbotsClient::new_mainnet();
    /// let block_number = U64::from(17000000u64);
    /// let block_info = client.get_block(block_number).await?;
    /// println!("Block miner: {:?}", block_info.miner);
    /// ```
    pub async fn get_block(&self, block_number: U64) -> FlashbotsResult<BlockResponse> {
        let url = format!("{}/block/{}", self.config.base_url, block_number);
        self.get(&url).await
    }

    /// Get the latest block information
    pub async fn get_latest_block(&self) -> FlashbotsResult<BlockResponse> {
        let url = format!("{}/block/latest", self.config.base_url);
        self.get(&url).await
    }

    /// Obtain user status information (without relying on specific blocks)
    ///
    /// # Example
    /// ```
    /// let client = FlashbotsClient::new_mainnet();
    /// let address = Address::zero(); // real address
    /// let user_status = client.get_user_status(address).await?;
    /// println!("User reputation: {:?}", user_status.reputation);
    /// ```
    pub async fn get_user_status(&self, address: Address) -> FlashbotsResult<UserStatus> {
        let url = format!("{}/userStatus/{}", self.config.base_url, address);
        self.get(&url).await
    }

    /// Get recommended gas prices
    ///
    /// # Example
    /// ```
    /// let client = FlashbotsClient::new_mainnet();
    /// let gas_prices = client.get_gas_price().await?;
    /// println!("Fast gas price: {}", gas_prices.fast_gas_price);
    /// ```
    pub async fn get_gas_price(&self) -> FlashbotsResult<GasPriceResponse> {
        let url = format!("{}/gasPrice", self.config.base_url);
        self.get(&url).await
    }

    /// Get the list of API endpoints supported by the repeater.
    pub async fn get_supported_endpoints(&self) -> FlashbotsResult<Vec<String>> {
        let url = format!("{}/", self.config.base_url);
        let response: Value = self.get(&url).await?;
        Ok(response
            .get("supported_apis")
            .and_then(|apis| apis.as_array())
            .map(|apis| {
                apis.iter()
                    .filter_map(|api| api.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Get repeater information
    ///
    /// # Example
    /// ```
    /// use flashbots_rs::FlashbotsClient;
    ///
    /// let client = FlashbotsClient::new_mainnet();
    /// let relay_info = client.get_relay_info().await?;
    /// println!("Relay: {} v{}", relay_info.name, relay_info.version);
    /// ```
    pub async fn get_relay_info(&self) -> FlashbotsResult<RelayInfo> {
        let url = format!("{}/", self.config.base_url);
        self.get(&url).await
    }

    /// Batch Transaction Package Receipts
    pub async fn get_bundle_receipts(
        &self,
        bundle_hashes: Vec<H256>,
    ) -> FlashbotsResult<Vec<BundleReceipt>> {
        let mut receipts = Vec::new();
        for bundle_hash in bundle_hashes {
            match self.get_bundle_receipt(bundle_hash).await {
                Ok(receipt) => receipts.push(receipt),
                Err(e) => {
                    log::warn!(
                        "Failed to get receipt for bundle {:?}: {:?}",
                        bundle_hash,
                        e
                    );
                }
            }
        }
        Ok(receipts)
    }

    /// Check if multiple transaction packages have been included.
    pub async fn check_bundle_inclusions(
        &self,
        bundle_hashes: Vec<H256>,
    ) -> FlashbotsResult<Vec<(H256, bool)>> {
        let mut results = Vec::new();
        for bundle_hash in bundle_hashes {
            let is_included = self.get_bundle_receipt(bundle_hash).await.is_ok();
            results.push((bundle_hash, is_included));
        }
        Ok(results)
    }

    /// Send transaction package and get receipt (simplified version)
    pub async fn send_bundle_and_get_receipt(
        &self,
        bundle: Bundle,
        timeout_blocks: u64,
    ) -> FlashbotsResult<BundleReceipt> {
        let response = self.send_bundle_with_retry(bundle).await?;
        log::info!("Bundle sent successfully, hash: {:?}", response.bundle_hash);
        match self
            .wait_for_bundle_inclusion(response.bundle_hash, timeout_blocks)
            .await?
        {
            Some(receipt) => Ok(receipt),
            None => Err(FlashbotsError::Error(
                "Bundle not included within timeout".to_string(),
            )),
        }
    }

    /// Verify transaction package format
    pub async fn validate_bundle(&self, bundle: &Bundle) -> FlashbotsResult<bool> {
        if bundle.txs.is_empty() {
            return Err(FlashbotsError::Error(
                "Bundle must contain at least one transaction".to_string(),
            ));
        }
        // Check block number
        if bundle
            .block_number
            .ok_or(|| FlashbotsError::Error(format!("block number is empty")))
            .is_err()
            || bundle.block_number.unwrap().is_zero()
        {
            return Err(FlashbotsError::Error(
                "Bundle must have a valid block number".to_string(),
            ));
        }
        // Check the timestamp range (if any).
        if let (Some(min_ts), Some(max_ts)) = (bundle.min_timestamp, bundle.max_timestamp) {
            if min_ts > max_ts {
                return Err(FlashbotsError::Error(
                    "Invalid timestamp range: min_timestamp > max_timestamp".to_string(),
                ));
            }
        }
        Ok(true)
    }

    /// Obtain repeater performance statistics
    pub async fn get_relay_stats(&self) -> FlashbotsResult<Value> {
        let url = format!("{}/relayStats", self.config.base_url);
        self.get(&url).await
    }

    /// Get Builder Information
    pub async fn get_builder_info(&self) -> FlashbotsResult<Value> {
        let url = format!("{}/builder", self.config.base_url);
        self.get(&url).await
    }

    /// Batch simulated trading package
    pub async fn simulate_bundles(
        &self,
        simulations: Vec<SimulateBundleRequest>,
    ) -> FlashbotsResult<Vec<FlashbotsResult<SimulateBundleResponse>>> {
        let mut results = Vec::new();
        for simulation in simulations {
            let result = self.simulate_bundle_with_retry(simulation).await;
            results.push(result);
        }
        Ok(results)
    }

    /// Send the transaction package and cancel it immediately (for testing purposes).
    pub async fn send_and_cancel_bundle(&self, bundle: Bundle) -> FlashbotsResult<bool> {
        let response = self.send_bundle_with_retry(bundle).await?;
        log::info!(
            "Bundle sent, attempting to cancel: {:?}",
            response.bundle_hash
        );
        let cancel_result = self.cancel_bundles(vec![response.bundle_hash]).await?;
        Ok(cancel_result.success)
    }

    /// Get network information
    pub async fn get_network_info(&self) -> FlashbotsResult<Value> {
        let url = format!("{}/network", self.config.base_url);
        self.get(&url).await
    }

    /// Check if the address is blacklisted.
    pub async fn is_blacklisted(&self, address: Address) -> FlashbotsResult<bool> {
        let user_status = self.get_user_status(address).await?;
        Ok(user_status.blacklisted)
    }

    /// Get repeater version
    pub async fn get_version(&self) -> FlashbotsResult<String> {
        let relay_info = self.get_relay_info().await?;
        Ok(relay_info.version)
    }
}

impl FlashbotsClient {
    /// Creates a production-ready client (Mainnet, high retry count)
    pub fn new_production() -> Self {
        Self::new(FlashbotsClientConfig {
            base_url: MAIN_URL.to_string(),
            timeout_seconds: 60,
            max_retries: 5,
            retry_delay_seconds: 3,
        })
    }

    /// Creates a development client (testnet, fast failure)
    pub fn new_development() -> Self {
        Self::new(FlashbotsClientConfig {
            base_url: SEPOLIA_TEST_URL.to_string(),
            timeout_seconds: 15,
            max_retries: 1,
            retry_delay_seconds: 1,
        })
    }
}
