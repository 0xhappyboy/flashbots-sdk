<h1 align="center">
   ⚡ Flashbots SDK
</h1>
<h4 align="center">
A Rust SDK for interacting with the Flashbots relay, providing functionality for building, signing, sending, and monitoring transaction packages.
</h4>
<p align="center">
  <a href="https://github.com/0xhappyboy/solana-trader/LICENSE"><img src="https://img.shields.io/badge/License-GPL3.0-d1d1f6.svg?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=googledocs&label=license&logoColor=BEC5C9" alt="License"></a>
    <a href="https://crates.io/crates/solana-network-sdk">
<img src="https://img.shields.io/badge/crates-flashbots--sdk-20B2AA.svg?style=flat&labelColor=0F1F2D&color=FFD700&logo=rust&logoColor=FFD700">
</a>
</p>
<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

## Features

- 🚀 Full Flashbots API support
- 🔐 Secure transaction signing and verification
- 📦 Smooth transaction package builder
- ⚡ Asynchronous/wait support
- 🔄 Automatic retry mechanism
- 🧪 Transaction demoting and verification
- 📊 Rich statistics and monitoring functions

## Example

### Create Client

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = FlashbotsClient::new_mainnet();
    let is_healthy = client.get_health().await?;
    println!("Relay healthy: {}", is_healthy);
    Ok(())
}
```

### Building and sending transaction packets

```rust
use flashbots_rs::{FlashbotsClient, BundleBuilder};
use ethers::types::{U64, H256};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = FlashbotsClient::new_mainnet();

    let bundle = BundleBuilder::new()
        .add_transaction("0x02f87401843b9aca00843b9aca0082520894...".to_string())
        .add_transaction("0x02f87401843b9aca00843b9aca0082520894...".to_string())
        .block_number(U64::from(17000000u64))
        .min_timestamp(1625097600)
        .max_timestamp(1625184000)
        .build();

    match client.send_and_wait_for_bundle(bundle, 10).await? {
        Some(receipt) => {
            println!("Bundle included in block: {}", receipt.block_number);
            println!("MEV reward: {:?}", receipt.mev_reward);
        }
        None => println!("Bundle not included within timeout"),
    }

    Ok(())
}
```

### Trading Simulation

```rust
use flashbots_rs::{FlashbotsClient, BundleBuilder, SimulateBundleRequest};
use ethers::types::U64;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = FlashbotsClient::new_mainnet();

    let bundle = BundleBuilder::new()
        .add_transaction("hash...".to_string())
        .block_number(U64::from(17000000u64))
        .build();

    let simulation_request = SimulateBundleRequest {
        bundle: bundle.clone(),
        state_block_number: Some(U64::from(16999999u64)),
        parent_block: None,
        block_number: None,
        timestamp: None,
        gas_limit: None,
        base_fee: None,
        timeout: None,
    };

    let simulation = client.simulate_bundle(simulation_request).await?;

    if simulation.success {
        println!("Simulation successful!");
        println!("Gas used: {}", simulation.gas_used);
        println!("MEV reward: {:?}", simulation.mev_reward);
    } else {
        println!("Simulation failed: {:?}", simulation.error);
    }

    Ok(())
}
```

### Creating and signing transactions

```rust
use flashbots_rs::transaction::TransactionBuilder;
use flashbots_rs::tool::create_random_wallet;
use ethers::types::Address;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tx_builder = TransactionBuilder::new(EvmType::ETHEREUM_MAINNET).await?;

    let wallet = create_random_wallet();

    let to_address = Address::from_str("address")?;
    let value = U256::from(1000000000000000000u64); // 1 ETH

    let signed_tx = tx_builder
        .create_and_sign_eth_transfer(wallet, to_address, value)
        .await?;

    println!("Signed transaction: {}", signed_tx);

    Ok(())
}
```

### Batch transaction processing

```rust
use flashbots_rs::transaction::{TransactionBuilder, BatchTransactionBuilder};
use flashbots_rs::tool::create_random_wallet;
use ethers::types::{Address, TransactionRequest, U256};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wallet = create_random_wallet();
    let to_address = Address::from_str("0x742d35Cc6634C0532925a3b8Dc9F5a5f6b6b6b6b")?;

    let batch_builder = BatchTransactionBuilder::new(EvmType::ETHEREUM_MAINNET).await?;

    let batch = batch_builder
        .add_transaction(
            TransactionRequest::new()
                .to(to_address)
                .value(U256::from(1000000000000000000u64)) // 1 ETH
        )
        .add_transaction(
            TransactionRequest::new()
                .to(to_address)
                .value(U256::from(2000000000000000000u64)) // 2 ETH
        );

    let signed_txs = batch.sign_all(wallet).await?;

    println!("Signed {} transactions", signed_txs.len());
    for (i, tx) in signed_txs.iter().enumerate() {
        println!("Transaction {}: {}", i + 1, tx);
    }

    Ok(())
}
```

### Simulate and send

```rust
use flashbots_rs::{FlashbotsClient, BundleBuilder, SimulateBundleRequest};
use ethers::types::U64;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = FlashbotsClient::new_mainnet();

    let bundle = BundleBuilder::new()
        .add_transaction("0x02f87401843b9aca00843b9aca0082520894...".to_string())
        .block_number(U64::from(17000000u64))
        .build();

    let simulation_params = SimulateBundleRequest {
        bundle: bundle.clone(),
        state_block_number: Some(U64::from(16999999u64)),
        ..Default::default()
    };

    match client.simulate_and_send_bundle(bundle, simulation_params).await? {
        Some(response) => {
            println!("Bundle sent successfully: {:?}", response.bundle_hash);
        }
        None => {
            println!("Bundle simulation failed, not sending");
        }
    }

    Ok(())
}
```

### Wallet management

```rust
use flashbots_rs::tool::{create_wallet_from_private_key, create_wallet_from_mnemonic};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = "private key";
    let wallet1 = create_wallet_from_private_key(private_key)?;
    println!("Wallet address: {:?}", wallet1.address());

    let mnemonic = "test test test test test test test test test test test junk";
    let wallet2 = create_wallet_from_mnemonic(mnemonic, Some("m/44'/60'/0'/0/0"))?;
    println!("Wallet address: {:?}", wallet2.address());

    let wallet3 = create_random_wallet();
    println!("Random wallet address: {:?}", wallet3.address());

    Ok(())
}
```
