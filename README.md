<h1 align="center">
    Jupiter SDK
</h1>
<h4 align="center">
A Rust SDK for interacting with Jupiter, implementing full token swapping, price lookup, and transaction monitoring functionality.
</h4>
<p align="center">
  <a href="https://github.com/0xhappyboy/jup-sdk/LICENSE"><img src="https://img.shields.io/badge/License-GPL3.0-d1d1f6.svg?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=googledocs&label=license&logoColor=BEC5C9" alt="License"></a>
</p>
<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

# Features

## 🚀 Complete Jupiter API integration - supports all major endpoints

## 💰 Token Exchange - Get quotes and create exchange transactions

## 📊 Price Inquiry - Real-time Token Prices and Batch Inquiry

## 🛣️ Route Analysis - Intelligent Route Selection and Optimization

## 🔄 Transaction monitoring - Real-time transaction status tracking

## 🔄 Retry Mechanism - Intelligent Error Handling and Retry

# Example

## Basic usage

```rust
use jup-sdk::{JupiterClient, QuoteRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = JupiterClient::new()?;
    let tokens = client.get_tokens().await?;
    println!("Support {} tokens", tokens.len());
    let sol_price = client.get_token_price("So11111111111111111111111111111111111111112").await?;
    println!("SOL Price: {:?}", sol_price);
    Ok(())
}
```

## 获取交换报价

```rust
use jup-sdk::{JupiterClient, QuoteRequest};

async fn get_swap_quote() -> Result<(), Box<dyn std::error::Error>> {
    let client = JupiterClient::new()?;

    let request = QuoteRequest {
        input_mint: "So11111111111111111111111111111111111111112".to_string(), // SOL
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
        amount: 1000000, // 0.001 SOL
        slippage_bps: 50, // 0.5% Slippage
        fee_bps: None,
        only_direct_routes: None,
        as_legacy_transaction: None,
        restrict_middle_tokens: None,
    };

    let quote = client.get_quote(&request).await?;
    println!("Input amount: {}", quote.in_amount);
    println!("Output amount: {}", quote.out_amount);
    println!("Price impact: {}%", quote.price_impact_pct);

    Ok(())
}
```

## Simplify quote retrieval

```rust
use jup-sdk::JupiterClient;

async fn simple_quote() -> Result<(), Box<dyn std::error::Error>> {
    let client = JupiterClient::new()?;

    let input_mint = "So11111111111111111111111111111111111111112"; // SOL
    let output_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC
    let amount = 1000000; // 0.001 SOL
    let quote = client.simple_swap_quote(input_mint, output_mint, amount, Some(50)).await?;
    println!("You can receive {} USDC", quote.out_amount);

    Ok(())
}
```

## Create exchange transaction

```rust
use jup-sdk::{JupiterClient, QuoteRequest};

async fn create_swap_transaction() -> Result<(), Box<dyn std::error::Error>> {
    let client = JupiterClient::new()?;

    let request = QuoteRequest {
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        amount: 1000000,
        slippage_bps: 50,
        fee_bps: None,
        only_direct_routes: None,
        as_legacy_transaction: None,
        restrict_middle_tokens: None,
    };
    let quote = client.get_quote(&request).await?;
    let user_public_key = "YourPublicKeyHere123456789012345678901234567890123";
    let swap_response = client.create_swap_transaction(quote, user_public_key, Some(true)).await?;
    println!("Transaction data: {}", swap_response.swap_transaction);
    println!("Last valid block height: {}", swap_response.last_valid_block_height);

    Ok(())
}
```

## Bulk price inquiry

```rust
use jup-sdk::JupiterClient;
use std::collections::HashMap;

async fn batch_prices() -> Result<(), Box<dyn std::error::Error>> {
    let client = JupiterClient::new()?;
    let token_pairs = &[
        ("So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // SOL/USDC
        ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "So11111111111111111111111111111111111111112"), // USDC/SOL
        ("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // USDT/USDC
    ];
    let prices: HashMap<String, f64> = client.get_prices_batch(token_pairs).await?;
    for (pair, price) in prices {
        println!("{}: {}", pair, price);
    }
    Ok(())
}
```

## Route analysis

```rust
use jup-sdk::JupiterClient;

async fn analyze_routes() -> Result<(), Box<dyn std::error::Error>> {
    let client = JupiterClient::new()?;

    let input_mint = "So11111111111111111111111111111111111111112";
    let output_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let amount = 1000000;

    let analysis = client.analyze_routes(input_mint, output_mint, amount, Some(3)).await?;

   println!("Best route output: {}", analysis.best_route.out_amount);
   println!("Confidence score: {:.2}", analysis.confidence_score);
   println!("Number of alternative routes: {}", analysis.alternative_routes.len());

    for (i, route) in analysis.alternative_routes.iter().enumerate() {
        println!("Alternative routes {}: {}", i + 1, route.out_amount);
    }

    Ok(())
}
```

## Transaction monitoring

```rust
use jup-sdk::{JupiterClient, Solana};

async fn monitor_transaction() -> Result<(), Box<dyn std::error::Error>> {
    let client = JupiterClient::new()?;
    let solana = Solana::new(solana_network_sdk::types::Mode::MAIN)?;

    let signature = "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYvC7j45R5...";

    let result = client.monitor_transaction(signature, &solana, None).await?;

    println!("Transaction status: {:?}", result.status);
    println!("Block: {}", result.slot);
    if let Some(error) = result.error {
        println!("Error: {}", error);
    }

    Ok(())
}
```

## Token search and filtering

```rust
use jup-sdk::JupiterClient;

async fn token_operations() -> Result<(), Box<dyn std::error::Error>> {
    let client = JupiterClient::new()?;

    if let Some(sol_token) = client.get_token_by_symbol("SOL").await? {
        println!("SOL address: {}", sol_token.address);
        println!("SOL decimal places: {}", sol_token.decimals);
    }

    let usdc_address = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    if let Some(usdc_token) = client.get_token_by_address(usdc_address).await? {
        println!("USDC symbol: {}", usdc_token.symbol);
    }

    let stablecoins = client.get_tokens_by_tag("stablecoin").await?;
    println!("Found {} stablecoins", stablecoins.len());

    let tokens_page = client.get_tokens_paginated(Some(1), Some(50)).await?;
    println!("Number of tokens on page 1: {}", tokens_page.len());

    Ok(())
}
```
