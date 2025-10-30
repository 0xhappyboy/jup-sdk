<h1 align="center">
     Jupiter SDK
</h1>
<h4 align="center">
一个用于与 Jupiter DEX 交互的 Rust SDK, 实现了完整的代币交换、价格查询和交易监控功能.
</h4>
<p align="center">
  <a href="https://github.com/0xhappyboy/jup-sdk/LICENSE"><img src="https://img.shields.io/badge/License-GPL3.0-d1d1f6.svg?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=googledocs&label=license&logoColor=BEC5C9" alt="License"></a>
</p>
<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

# 功能特性

## 🚀 完整的 Jupiter API 集成 - 支持所有主要端点

## 💰 代币交换 - 获取报价、创建交换交易

## 📊 价格查询 - 实时代币价格和批量查询

## 🛣️ 路由分析 - 智能路由选择和优化

## 🔄 交易监控 - 实时交易状态跟踪

## 🔄 重试机制 - 智能错误处理和重试

# Example

## 基础使用

```rust
use jup-sdk::{JupiterClient, QuoteRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端
    let client = JupiterClient::new()?;

    // 获取代币列表
    let tokens = client.get_tokens().await?;
    println!("支持 {} 个代币", tokens.len());

    // 获取 SOL 价格
    let sol_price = client.get_token_price("So11111111111111111111111111111111111111112").await?;
    println!("SOL 价格: {:?}", sol_price);

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
        slippage_bps: 50, // 0.5% 滑点
        fee_bps: None,
        only_direct_routes: None,
        as_legacy_transaction: None,
        restrict_middle_tokens: None,
    };

    let quote = client.get_quote(&request).await?;
    println!("输入金额: {}", quote.in_amount);
    println!("输出金额: {}", quote.out_amount);
    println!("价格影响: {}%", quote.price_impact_pct);

    Ok(())
}
```

## 简化报价获取

```rust
use jup-sdk::JupiterClient;

async fn simple_quote() -> Result<(), Box<dyn std::error::Error>> {
    let client = JupiterClient::new()?;

    let input_mint = "So11111111111111111111111111111111111111112"; // SOL
    let output_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC
    let amount = 1000000; // 0.001 SOL

    let quote = client.simple_swap_quote(input_mint, output_mint, amount, Some(50)).await?;
    println!("可收到 {} USDC", quote.out_amount);

    Ok(())
}
```

## 创建交换交易

```rust
use jup-sdk::{JupiterClient, QuoteRequest};

async fn create_swap_transaction() -> Result<(), Box<dyn std::error::Error>> {
    let client = JupiterClient::new()?;

    // 首先获取报价
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

    // 创建交换交易
    let user_public_key = "YourPublicKeyHere123456789012345678901234567890123";
    let swap_response = client.create_swap_transaction(quote, user_public_key, Some(true)).await?;

    println!("交易数据: {}", swap_response.swap_transaction);
    println!("最后有效区块高度: {}", swap_response.last_valid_block_height);

    Ok(())
}
```

## 批量价格查询

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

## 路由分析

```rust
use jup-sdk::JupiterClient;

async fn analyze_routes() -> Result<(), Box<dyn std::error::Error>> {
    let client = JupiterClient::new()?;

    let input_mint = "So11111111111111111111111111111111111111112";
    let output_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let amount = 1000000;

    let analysis = client.analyze_routes(input_mint, output_mint, amount, Some(3)).await?;

    println!("最佳路由输出: {}", analysis.best_route.out_amount);
    println!("置信度评分: {:.2}", analysis.confidence_score);
    println!("备选路由数量: {}", analysis.alternative_routes.len());

    for (i, route) in analysis.alternative_routes.iter().enumerate() {
        println!("备选路由 {}: {}", i + 1, route.out_amount);
    }

    Ok(())
}
```

## 交易监控

```rust
use jup-sdk::{JupiterClient, Solana};

async fn monitor_transaction() -> Result<(), Box<dyn std::error::Error>> {
    let client = JupiterClient::new()?;
    let solana = Solana::new(solana_network_sdk::types::Mode::MAIN)?;

    let signature = "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYvC7j45R5...";

    let result = client.monitor_transaction(signature, &solana, None).await?;

    println!("交易状态: {:?}", result.status);
    println!("区块: {}", result.slot);
    if let Some(error) = result.error {
        println!("错误: {}", error);
    }

    Ok(())
}
```

## 代币搜索和过滤

```rust
use jup-sdk::JupiterClient;

async fn token_operations() -> Result<(), Box<dyn std::error::Error>> {
    let client = JupiterClient::new()?;

    // 按符号查找代币
    if let Some(sol_token) = client.get_token_by_symbol("SOL").await? {
        println!("SOL 地址: {}", sol_token.address);
        println!("SOL 小数位: {}", sol_token.decimals);
    }

    // 按地址查找代币
    let usdc_address = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    if let Some(usdc_token) = client.get_token_by_address(usdc_address).await? {
        println!("USDC 符号: {}", usdc_token.symbol);
    }

    // 按标签过滤代币
    let stablecoins = client.get_tokens_by_tag("stablecoin").await?;
    println!("找到 {} 个稳定币", stablecoins.len());

    // 分页获取代币
    let tokens_page = client.get_tokens_paginated(Some(1), Some(50)).await?;
    println!("第 1 页代币数量: {}", tokens_page.len());

    Ok(())
}
```
