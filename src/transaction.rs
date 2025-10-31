use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use evm_client::{EvmClient, EvmType};
use std::sync::Arc;

use crate::types::{FlashbotsError, FlashbotsResult};
/// Builder for creating and signing Ethereum transactions
pub struct TransactionBuilder {
    evm_client: Arc<EvmClient>,
}

/// Creates a new TransactionBuilder with the given provider
impl TransactionBuilder {
    pub async fn new(evm_type: EvmType) -> FlashbotsResult<Self> {
        Ok(Self {
            evm_client: Arc::new(
                EvmClient::from_type(evm_type)
                    .await
                    .map_err(|e| FlashbotsError::Error(format!("{:?}", e)))?,
            ),
        })
    }

    /// Creates a basic ETH transfer transaction
    ///
    /// # Example
    /// ```
    /// use ethers::types::{Address, U256};
    /// use ethers::providers::Provider;
    /// use std::str::FromStr;
    ///
    /// let builder = TransactionBuilder::new(EvmType::ETHEREUM_MAINNET).await?;
    /// let wallet = create_random_wallet();
    /// let to_address = Address::from_str("0x742d35Cc6634C0532925a3b8Dc9F5a5f6b6b6b6b")?;
    /// let value = U256::from(1000000000000000000u64); // 1 ETH
    /// let tx = builder.create_eth_transfer(wallet, to_address, value).await?;
    /// ```
    pub async fn create_eth_transfer(
        &self,
        from: LocalWallet,
        to: Address,
        value: U256,
    ) -> Result<TransactionRequest, Box<dyn std::error::Error>> {
        let nonce = self
            .evm_client
            .provider
            .get_transaction_count(from.address(), None)
            .await?;
        let gas_price = self.evm_client.provider.get_gas_price().await?;
        Ok(TransactionRequest::new()
            .from(from.address())
            .to(to)
            .value(value)
            .nonce(nonce)
            .gas_price(gas_price)
            .gas(21000))
    }

    /// Creates a contract call transaction
    pub async fn create_contract_call(
        &self,
        from: Address,
        to: Address,
        data: Bytes,
        value: Option<U256>,
    ) -> Result<TransactionRequest, Box<dyn std::error::Error>> {
        let nonce = self
            .evm_client
            .provider
            .get_transaction_count(from, None)
            .await?;
        let gas_price = self.evm_client.provider.get_gas_price().await?;
        let mut tx = TransactionRequest::new()
            .from(from)
            .to(to)
            .data(data)
            .nonce(nonce)
            .gas_price(gas_price);
        if let Some(val) = value {
            tx = tx.value(val);
        }
        Ok(tx)
    }

    /// Signs a transaction with the given wallet
    ///
    /// # Example
    /// ```
    /// let builder = TransactionBuilder::new(EvmType::ETHEREUM_MAINNET).await?;
    /// let wallet = create_random_wallet();
    /// let tx_request = TransactionRequest::new()
    ///     .to("0x742d35Cc6634C0532925a3b8Dc9F5a5f6b6b6b6b".parse()?)
    ///     .value(1000000000000000000u64);
    /// let signed_tx = builder.sign_transaction(wallet, tx_request).await?;
    /// println!("Signed transaction: {}", signed_tx);
    /// ```
    pub async fn sign_transaction(
        &self,
        wallet: LocalWallet,
        tx: TransactionRequest,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let chain_id = self.evm_client.provider.get_chainid().await?.as_u64();
        let mut typed_tx: TypedTransaction = tx.into();
        typed_tx.set_chain_id(chain_id);
        let signature = wallet.sign_transaction(&typed_tx).await?;
        let signed_tx = typed_tx.rlp_signed(&signature);
        let signed_tx_hex = format!("0x{}", hex::encode(signed_tx));
        Ok(signed_tx_hex)
    }

    /// Estimates gas for a transaction
    pub async fn estimate_gas(
        &self,
        tx: &TransactionRequest,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        let typed_tx: TypedTransaction = tx.clone().into();
        let gas_estimate = self
            .evm_client
            .provider
            .estimate_gas(&typed_tx, None)
            .await?;
        Ok(gas_estimate)
    }

    /// Gets current gas price from the network
    pub async fn get_current_gas_price(&self) -> Result<U256, Box<dyn std::error::Error>> {
        let gas_price = self.evm_client.provider.get_gas_price().await?;
        Ok(gas_price)
    }

    /// Gets nonce for an address
    pub async fn get_nonce(&self, address: Address) -> Result<U256, Box<dyn std::error::Error>> {
        let nonce = self
            .evm_client
            .provider
            .get_transaction_count(address, None)
            .await?;
        Ok(nonce)
    }

    /// Creates and signs an ETH transfer in one operation
    pub async fn create_and_sign_eth_transfer(
        &self,
        wallet: LocalWallet,
        to: Address,
        value: U256,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let tx_request = self.create_eth_transfer(wallet.clone(), to, value).await?;
        self.sign_transaction(wallet, tx_request).await
    }

    /// Creates and signs a contract call in one operation
    pub async fn create_and_sign_contract_call(
        &self,
        wallet: LocalWallet,
        to: Address,
        data: Bytes,
        value: Option<U256>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let tx_request = self
            .create_contract_call(wallet.address(), to, data, value)
            .await?;
        self.sign_transaction(wallet, tx_request).await
    }

    /// Creates an EIP-1559 type transaction
    ///
    /// # Example
    /// ```
    /// let builder = TransactionBuilder::new(EvmType::ETHEREUM_MAINNET).await?;
    /// let wallet = create_random_wallet();
    /// let to_address = "0x742d35Cc6634C0532925a3b8Dc9F5a5f6b6b6b6b".parse()?;
    /// let value = U256::from(1000000000000000000u64);
    /// let tx = builder.create_eip1559_transaction(wallet, to_address, value).await?;
    /// ```
    pub async fn create_eip1559_transaction(
        &self,
        from: LocalWallet,
        to: Address,
        value: U256,
    ) -> Result<Eip1559TransactionRequest, Box<dyn std::error::Error>> {
        let nonce = self
            .evm_client
            .provider
            .get_transaction_count(from.address(), None)
            .await?;
        let block = self
            .evm_client
            .provider
            .get_block(BlockNumber::Latest)
            .await?;
        let base_fee = block
            .and_then(|b| b.base_fee_per_gas)
            .unwrap_or_else(|| U256::from(1000000000)); // 默认 1 gwei
        let max_priority_fee_per_gas = U256::from(1500000000); // 1.5 gwei
        Ok(Eip1559TransactionRequest::new()
            .from(from.address())
            .to(to)
            .value(value)
            .nonce(nonce)
            .max_fee_per_gas(base_fee + max_priority_fee_per_gas)
            .max_priority_fee_per_gas(max_priority_fee_per_gas)
            .gas(21000))
    }

    /// Signs an EIP-1559 transaction
    pub async fn sign_eip1559_transaction(
        &self,
        wallet: LocalWallet,
        tx: Eip1559TransactionRequest,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let chain_id = self.evm_client.provider.get_chainid().await?.as_u64();
        let mut typed_tx: TypedTransaction = tx.into();
        typed_tx.set_chain_id(chain_id);
        let signature = wallet.sign_transaction(&typed_tx).await?;
        let signed_tx = typed_tx.rlp_signed(&signature);
        let signed_tx_hex = format!("0x{}", hex::encode(signed_tx));
        Ok(signed_tx_hex)
    }

    /// Signs raw transaction data
    pub async fn sign_raw_transaction(
        &self,
        wallet: LocalWallet,
        tx_data: Bytes,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let chain_id = self.evm_client.provider.get_chainid().await?.as_u64();
        let tx_request = TransactionRequest::new().data(tx_data);
        let mut typed_tx: TypedTransaction = tx_request.into();
        typed_tx.set_chain_id(chain_id);
        let signature = wallet.sign_transaction(&typed_tx).await?;
        let signed_tx = typed_tx.rlp_signed(&signature);
        let signed_tx_hex = format!("0x{}", hex::encode(signed_tx));
        Ok(signed_tx_hex)
    }
}

/// Builder for creating multiple transactions in batch
pub struct BatchTransactionBuilder {
    evm_client: Arc<EvmClient>,
    transactions: Vec<TransactionRequest>,
}

impl BatchTransactionBuilder {
    /// Creates a new BatchTransactionBuilder
    pub async fn new(evm_type: EvmType) -> FlashbotsResult<Self> {
        Ok(Self {
            evm_client: Arc::new(
                EvmClient::from_type(evm_type)
                    .await
                    .map_err(|e| FlashbotsError::Error(format!("{:?}", e)))?,
            ),
            transactions: Vec::new(),
        })
    }
    /// Adds a transaction to the batch
    pub fn add_transaction(mut self, tx: TransactionRequest) -> Self {
        self.transactions.push(tx);
        self
    }

    /// Estimates gas for all transactions in the batch
    pub async fn estimate_all_gas(&self) -> Result<Vec<U256>, Box<dyn std::error::Error>> {
        let mut estimates = Vec::new();
        for tx in &self.transactions {
            let typed_tx: TypedTransaction = tx.clone().into();
            let estimate = self
                .evm_client
                .provider
                .estimate_gas(&typed_tx, None)
                .await?;
            estimates.push(estimate);
        }
        Ok(estimates)
    }

    /// Signs all transactions in the batch with the same wallet
    ///
    /// # Example
    /// ```
    /// let builder = BatchTransactionBuilder::new(EvmType::ETHEREUM_MAINNET).await?
    /// let wallet = create_random_wallet();
    /// let batch = builder
    ///     .add_transaction(TransactionRequest::new().value(1000000000000000000u64))
    ///     .add_transaction(TransactionRequest::new().value(2000000000000000000u64));
    /// let signed_txs = batch.sign_all(wallet).await?;
    /// println!("Signed {} transactions", signed_txs.len());
    /// ```
    pub async fn sign_all(&self, wallet: LocalWallet) -> FlashbotsResult<Vec<String>> {
        let mut signed_txs = Vec::new();
        // Create a TransactionBuilder with the same evm_client
        let tx_builder = TransactionBuilder {
            evm_client: Arc::clone(&self.evm_client),
        };
        for tx in &self.transactions {
            let signed_tx = tx_builder
                .sign_transaction(wallet.clone(), tx.clone())
                .await
                .map_err(|e| FlashbotsError::Error(format!("{:?}", e)))?;
            signed_txs.push(signed_tx);
        }
        Ok(signed_txs)
    }
}
