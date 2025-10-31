use crate::global::{GOERLI_TEST_URL, MAIN_URL, SEPOLIA_TEST_URL};
use crate::types::{
    Bundle, BundlePriceResponse, BundleReceipt, BundleStats, FlashbotsError,
    SimulateBundleResponse, UserStats,
};
use crate::types::{FlashbotsResult, SendBundleResponse, SimulateBundleRequest};
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
