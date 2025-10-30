use reqwest::Client;
use solana_network_sdk::Solana;
use std::{collections::HashMap, time::Duration};
use tokio::time;

use crate::{
    global::{DEFAULT_SLIPPAGE_BPS, JUPITER_BASE_URL},
    monitor::{Monitor, TransactionMonitorConfig, TransactionMonitorResult},
    retry::RetryConfig,
    router::RouteAnalysis,
    tool::{is_valid_mint_address, validate_pubkey, validate_slippage_bps},
    types::{
        JupiterError, PriceResponse, QuoteRequest, QuoteResponse, SwapRequest, SwapResponse,
        TokenInfo,
    },
};

pub mod global;
pub mod monitor;
pub mod retry;
pub mod router;
pub mod tool;
pub mod types;

/// Configuration for Jupiter API client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub pool_idle_timeout: Duration,
    pub pool_max_idle_per_host: usize,
    pub user_agent: String,
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub rate_limit_requests_per_second: Option<u32>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: crate::global::JUPITER_BASE_URL.to_string(),
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            pool_idle_timeout: Duration::from_secs(90),
            pool_max_idle_per_host: 10,
            user_agent: format!("jup-sdk/{}", env!("CARGO_PKG_VERSION")),
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
            rate_limit_requests_per_second: Some(10), // Jupiter API 限制
        }
    }
}

/// Main client for interacting with Jupiter API
pub struct JupiterClient {
    client: Client,
    base_url: String,
    config: ClientConfig,
    solana: Solana,
}

impl JupiterClient {
    /// create a client
    /// Creates a new Jupiter client with default configuration
    ///
    /// # Example
    /// ```rust
    /// use jupiter_sdk::JupiterClient;
    /// let client = JupiterClient::new().unwrap();
    /// ```
    pub fn new() -> Result<Self, JupiterError> {
        Ok(Self {
            client: Client::new(),
            base_url: JUPITER_BASE_URL.to_string(),
            config: ClientConfig::default(),
            solana: Solana::new(solana_network_sdk::types::Mode::MAIN)
                .map_err(|e| JupiterError::Error(format!("create solana client error: {:?}", e)))?,
        })
    }

    /// create a client based on the URL, using the default configuration.
    /// Creates a client with custom base URL
    ///
    /// # Example
    /// ```rust
    /// use jupiter_sdk::JupiterClient;
    /// let client = JupiterClient::from_base_url("https://quote-api.jup.ag".to_string()).unwrap();
    /// ```
    pub fn from_base_url(base_url: String) -> Result<Self, JupiterError> {
        Ok(Self {
            client: Client::new(),
            base_url,
            config: ClientConfig::default(),
            solana: Solana::new(solana_network_sdk::types::Mode::MAIN)
                .map_err(|e| JupiterError::Error(format!("create solana client error: {:?}", e)))?,
        })
    }

    /// reate a client based on an existing client, using the default configuration.
    pub fn from_client(client: Client) -> Result<Self, JupiterError> {
        Ok(Self {
            client,
            base_url: JUPITER_BASE_URL.to_string(),
            config: ClientConfig::default(),
            solana: Solana::new(solana_network_sdk::types::Mode::MAIN)
                .map_err(|e| JupiterError::Error(format!("create solana client error: {:?}", e)))?,
        })
    }

    /// create a client using configuration
    pub fn from_config(config: ClientConfig) -> Result<Self, crate::types::JupiterError> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .pool_idle_timeout(config.pool_idle_timeout)
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| crate::types::JupiterError::NetworkError(e.to_string()))?;
        Ok(Self {
            client,
            base_url: config.base_url.clone(),
            config: config,
            solana: Solana::new(solana_network_sdk::types::Mode::MAIN)
                .map_err(|e| JupiterError::Error(format!("create solana client error: {:?}", e)))?,
        })
    }

    /// create a client with rate limiting
    pub fn with_rate_limit(requests_per_second: u32) -> Result<Self, crate::types::JupiterError> {
        let mut config = ClientConfig::default();
        config.rate_limit_requests_per_second = Some(requests_per_second);
        Self::from_config(config)
    }

    /// Monitors transaction status
    ///
    /// # Example
    /// ```rust
    /// use jupiter_sdk::{JupiterClient, Solana};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = JupiterClient::new()?;
    /// let solana = Solana::new(solana_network_sdk::types::Mode::MAIN)?;
    /// let signature = "5verv...";
    /// let result = client.monitor_transaction(signature, &solana, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn monitor_transaction(
        &self,
        signature: &str,
        solana: &Solana,
        config: Option<TransactionMonitorConfig>,
    ) -> Result<TransactionMonitorResult, JupiterError> {
        let monitor = Monitor;
        monitor
            .monitor_transaction_status(signature, solana, config)
            .await
    }

    /// Monitors multiple transactions in batch
    pub async fn monitor_transactions_batch(
        &self,
        signatures: &[String],
        solana: &Solana,
        config: Option<TransactionMonitorConfig>,
    ) -> Result<Vec<TransactionMonitorResult>, JupiterError> {
        let monitor = Monitor;
        monitor
            .monitor_transactions_batch(signatures, solana, config)
            .await
    }

    /// Gets a quote for token swap
    ///
    /// # Example
    /// ```rust
    /// use jupiter_sdk::{JupiterClient, QuoteRequest};
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = JupiterClient::new()?;
    /// let request = QuoteRequest {
    ///     input_mint: "So11111111111111111111111111111111111111112".to_string(),
    ///     output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
    ///     amount: 1000000,
    ///     slippage_bps: 50,
    ///     fee_bps: None,
    ///     only_direct_routes: None,
    ///     as_legacy_transaction: None,
    ///     restrict_middle_tokens: None,
    /// };
    /// let quote = client.get_quote(&request).await?;
    /// Ok(())
    /// }
    /// ```
    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<QuoteResponse, JupiterError> {
        self.validate_quote_request(request)?;
        let url = format!("{}/quote", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&request)
            .send()
            .await
            .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
            return Err(JupiterError::RequestFailed(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }
        let quote: QuoteResponse = response
            .json()
            .await
            .map_err(|e| JupiterError::ParseError(e.to_string()))?;
        Ok(quote)
    }

    /// Gets swap transaction data
    ///
    /// # Example
    /// ```rust
    /// use jupiter_sdk::{JupiterClient, SwapRequest, QuoteResponse};
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = JupiterClient::new()?;
    /// let quote = QuoteResponse { /* ... */ };
    /// let request = SwapRequest {
    ///     quote_response: quote,
    ///     user_public_key: "YourPublicKeyHere".to_string(),
    ///     wrap_and_unwrap_sol: Some(true),
    ///     compute_unit_price: None,
    ///     prioritization_fee_lamports: None,
    /// };
    /// let swap_response = client.get_swap_transaction(&request).await?;
    /// Ok(())
    /// }
    /// ```
    pub async fn get_swap_transaction_data(
        &self,
        request: &SwapRequest,
    ) -> Result<SwapResponse, JupiterError> {
        self.validate_swap_request(request)?;
        let url = format!("{}/swap", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
            return Err(JupiterError::RequestFailed(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }
        let swap_response: SwapResponse = response
            .json()
            .await
            .map_err(|e| JupiterError::ParseError(e.to_string()))?;
        Ok(swap_response)
    }

    /// Gets list of all supported tokens
    pub async fn get_tokens(&self) -> Result<Vec<TokenInfo>, JupiterError> {
        let url = format!("{}/tokens", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
            return Err(JupiterError::RequestFailed(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }
        let tokens: Vec<TokenInfo> = response
            .json()
            .await
            .map_err(|e| JupiterError::ParseError(e.to_string()))?;
        Ok(tokens)
    }

    /// Gets prices for multiple tokens
    pub async fn get_price(
        &self,
        ids: &[String],
    ) -> Result<HashMap<String, PriceResponse>, JupiterError> {
        if ids.is_empty() {
            return Err(JupiterError::InvalidInput(
                "No token IDs provided".to_string(),
            ));
        }
        let url = format!("{}/price", self.base_url);
        let mut params = HashMap::new();
        params.insert("ids", ids.join(","));
        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
            return Err(JupiterError::RequestFailed(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }
        let prices: HashMap<String, PriceResponse> = response
            .json()
            .await
            .map_err(|e| JupiterError::ParseError(e.to_string()))?;
        Ok(prices)
    }

    /// Gets multiple routes for token swap
    pub async fn get_routes(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<Vec<QuoteResponse>, JupiterError> {
        self.validate_mint_address(input_mint)?;
        self.validate_mint_address(output_mint)?;
        validate_slippage_bps(slippage_bps).map_err(|e| JupiterError::Error(format!("{:?}", e)))?;
        let url = format!("{}/quote", self.base_url);
        let params = [
            ("inputMint", input_mint),
            ("outputMint", output_mint),
            ("amount", &amount.to_string()),
            ("slippageBps", &slippage_bps.to_string()),
        ];
        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
            return Err(JupiterError::RequestFailed(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }
        let routes: Vec<QuoteResponse> = response
            .json()
            .await
            .map_err(|e| JupiterError::ParseError(e.to_string()))?;
        Ok(routes)
    }

    /// Simple method to get swap quote
    ///
    /// # Example
    /// ```rust
    /// use jupiter_sdk::JupiterClient;
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = JupiterClient::new()?;
    /// let input_mint = "So11111111111111111111111111111111111111112";
    /// let output_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    /// let amount = 1000000;
    /// let quote = client.simple_swap_quote(input_mint, output_mint, amount, Some(50)).await?;
    /// Ok(())
    /// }
    /// ```
    pub async fn simple_swap_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: Option<u16>,
    ) -> Result<QuoteResponse, JupiterError> {
        let slippage = slippage_bps.unwrap_or(DEFAULT_SLIPPAGE_BPS);
        let request = QuoteRequest {
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            amount,
            slippage_bps: slippage,
            fee_bps: None,
            only_direct_routes: None,
            as_legacy_transaction: None,
            restrict_middle_tokens: None,
        };
        self.get_quote(&request).await
    }

    /// Finds token by symbol
    pub async fn get_token_by_symbol(
        &self,
        symbol: &str,
    ) -> Result<Option<TokenInfo>, JupiterError> {
        let tokens = self.get_tokens().await?;
        let token = tokens
            .into_iter()
            .find(|token| token.symbol.to_lowercase() == symbol.to_lowercase());
        Ok(token)
    }

    /// Finds token by address
    pub async fn get_token_by_address(
        &self,
        address: &str,
    ) -> Result<Option<TokenInfo>, JupiterError> {
        self.validate_mint_address(address)?;
        let tokens = self.get_tokens().await?;
        let token = tokens.into_iter().find(|token| token.address == address);
        Ok(token)
    }

    /// Gets price for a single token
    pub async fn get_token_price(&self, mint_address: &str) -> Result<Option<f64>, JupiterError> {
        self.validate_mint_address(mint_address)?;
        let prices = self.get_price(&[mint_address.to_string()]).await?;
        Ok(prices.get(mint_address).map(|price| price.price))
    }

    /// Creates swap transaction from quote
    pub async fn create_swap_transaction(
        &self,
        quote: QuoteResponse,
        user_public_key: &str,
        wrap_and_unwrap_sol: Option<bool>,
    ) -> Result<SwapResponse, JupiterError> {
        self.validate_pubkey(user_public_key)?;
        let request = SwapRequest {
            quote_response: quote,
            user_public_key: user_public_key.to_string(),
            wrap_and_unwrap_sol,
            compute_unit_price: None,
            prioritization_fee_lamports: None,
        };
        self.get_swap_transaction_data(&request).await
    }

    pub async fn get_quotes_batch(
        &self,
        requests: &[QuoteRequest],
    ) -> Result<Vec<Result<QuoteResponse, JupiterError>>, JupiterError> {
        let mut results = Vec::new();
        for request in requests {
            let result = self.get_quote(request).await;
            results.push(result);
        }
        Ok(results)
    }

    pub async fn get_quote_with_retry(
        &self,
        request: &QuoteRequest,
        max_retries: u32,
    ) -> Result<QuoteResponse, JupiterError> {
        for attempt in 0..=max_retries {
            match self.get_quote(request).await {
                Ok(quote) => return Ok(quote),
                Err(e) if attempt == max_retries => return Err(e),
                Err(e) if e.is_retriable() => {
                    let delay_ms = 200 * (attempt + 1) as u64;
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    /// Get Route Map - Used to understand all available transaction paths
    /// Gets all token pairs and routing information supported by Jupiter
    pub async fn get_indexed_route_map(
        &self,
    ) -> Result<crate::types::IndexedRouteMapResponse, JupiterError> {
        let url = format!("{}/indexed-route-map", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
            return Err(JupiterError::RequestFailed(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }
        let route_map: crate::types::IndexedRouteMapResponse = response
            .json()
            .await
            .map_err(|e| JupiterError::ParseError(e.to_string()))?;
        Ok(route_map)
    }

    /// Get a list of program IDs - used to verify the programs involved in a transaction
    /// Get all Solana program IDs involved in a Jupiter exchange
    pub async fn get_program_ids(&self) -> Result<Vec<String>, JupiterError> {
        let url = format!("{}/program-ids", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
            return Err(JupiterError::RequestFailed(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }
        let program_ids: Vec<String> = response
            .json()
            .await
            .map_err(|e| JupiterError::ParseError(e.to_string()))?;
        Ok(program_ids)
    }

    pub async fn health(&self) -> Result<bool, JupiterError> {
        let url = format!("{}/health", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
        Ok(response.status().is_success())
    }

    /// Batch Price Retrieval - Retrieve prices of multiple tokens at once
    /// Efficiently retrieve price information for multiple tokens, reducing the number of API calls.
    pub async fn get_prices_batch(
        &self,
        token_pairs: &[(&str, &str)], // (mint_address, vs_token)
    ) -> Result<HashMap<String, f64>, JupiterError> {
        if token_pairs.is_empty() {
            return Ok(HashMap::new());
        }
        let ids: Vec<String> = token_pairs
            .iter()
            .map(|(mint, vs)| format!("{}:{}", mint, vs))
            .collect();
        let mut params = HashMap::new();
        params.insert("ids", ids.join(","));
        let url = format!("{}/price", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
            return Err(JupiterError::RequestFailed(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }
        let prices: HashMap<String, crate::types::PriceResponse> = response
            .json()
            .await
            .map_err(|e| JupiterError::ParseError(e.to_string()))?;
        let result = prices
            .into_iter()
            .map(|(id, price)| (id, price.price))
            .collect();
        Ok(result)
    }

    /// Advanced Route Analysis - Compare multiple routes and select the optimal one
    //  Analyze metrics such as price impact, slippage, and execution time of different routes.
    pub async fn analyze_routes(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        max_routes: Option<usize>,
    ) -> Result<RouteAnalysis, JupiterError> {
        let routes = self.get_routes(input_mint, output_mint, amount, 50).await?;
        if routes.is_empty() {
            return Err(JupiterError::RequestFailed("No routes found".to_string()));
        }
        let best_route = routes.first().unwrap().clone();
        let mut analysis = RouteAnalysis::new(best_route);
        if routes.len() > 1 {
            let max_alt = max_routes.unwrap_or(3).min(routes.len() - 1);
            analysis.alternative_routes = routes[1..=max_alt].to_vec();
        }
        if let Ok(price_impact) = analysis.best_route.price_impact_pct.parse::<f64>() {
            analysis.confidence_score = (100.0 - price_impact.max(0.0)) / 100.0;
            analysis.confidence_score = analysis.confidence_score.max(0.1).min(1.0);
        }
        Ok(analysis)
    }

    /// Paginated token list - Use pagination when retrieving a large number of tokens
    /// Supports paginated retrieval of token lists to avoid loading too much data at once.
    pub async fn get_tokens_paginated(
        &self,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<TokenInfo>, JupiterError> {
        let url = format!("{}/tokens", self.base_url);
        let mut request_builder = self.client.get(&url);
        if let Some(page) = page {
            request_builder = request_builder.query(&[("page", page)]);
        }
        if let Some(page_size) = page_size {
            request_builder = request_builder.query(&[("pageSize", page_size)]);
        }
        let response = request_builder
            .send()
            .await
            .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
            return Err(JupiterError::RequestFailed(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }
        let tokens: Vec<TokenInfo> = response
            .json()
            .await
            .map_err(|e| JupiterError::ParseError(e.to_string()))?;
        Ok(tokens)
    }

    /// Filter tokens by tag - Get tokens categorized by purpose
    /// Filter tokens by tag (e.g., stablecoin, defi, etc.)
    pub async fn get_tokens_by_tag(&self, tag: &str) -> Result<Vec<TokenInfo>, JupiterError> {
        let all_tokens = self.get_tokens().await?;
        let filtered: Vec<TokenInfo> = all_tokens
            .into_iter()
            .filter(|token| token.tags.iter().any(|t| t == tag))
            .collect();
        Ok(filtered)
    }

    /// Calculate transaction fees - Estimate transaction execution costs
    /// Estimate transaction fees based on transaction complexity and current network status
    pub async fn estimate_transaction_fee(
        &self,
        quote: &QuoteResponse,
        priority_fee: Option<u64>,
    ) -> Result<u64, JupiterError> {
        // Base compute unit fee in micro-lamports per CU
        let base_fee = 5000; // micro-lamports per CU
        // Estimate compute units based on route complexity
        let compute_units = match quote.route_plan.len() {
            1 => 100_000, // Simple swap
            2 => 150_000, // Medium complexity
            _ => 200_000, // Complex route
        };
        let total_fee = base_fee * compute_units / 1_000_000; // 转换为 lamports
        let priority_fee = priority_fee.unwrap_or(0);
        Ok(total_fee + priority_fee)
    }

    /// Exchange transaction creation with retries
    pub async fn get_swap_transaction_with_retry(
        &self,
        request: &crate::types::SwapRequest,
        config: &RetryConfig,
    ) -> Result<crate::types::SwapResponse, JupiterError> {
        self.execute_with_retry(|| self.get_swap_transaction_data(request), config)
            .await
    }

    async fn execute_with_retry<F, T, Fut>(
        &self,
        operation: F,
        config: &RetryConfig,
    ) -> Result<T, JupiterError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, JupiterError>>,
    {
        let mut last_error = None;

        for attempt in 0..=config.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e.clone());
                    if attempt < config.max_retries && e.is_retriable() {
                        let delay = Self::cal_delay(attempt, config);
                        time::sleep(delay).await;
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
        Err(last_error
            .unwrap_or_else(|| JupiterError::Error("Unknown error after retries".to_string())))
    }

    /// Calculate backoff delay
    fn cal_delay(attempt: u32, config: &RetryConfig) -> Duration {
        let delay = config.initial_delay.as_millis() as f64
            * config.backoff_multiplier.powi(attempt as i32);
        let delay = delay.min(config.max_delay.as_millis() as f64);
        Duration::from_millis(delay as u64)
    }

    fn validate_quote_request(&self, request: &QuoteRequest) -> Result<(), JupiterError> {
        self.validate_mint_address(&request.input_mint)
            .map_err(|e| JupiterError::Error(format!("{:?}", e)))?;
        self.validate_mint_address(&request.output_mint)
            .map_err(|e| JupiterError::Error(format!("{:?}", e)))?;
        validate_slippage_bps(request.slippage_bps)
            .map_err(|e| JupiterError::Error(format!("{:?}", e)))?;
        if request.amount == 0 {
            return Err(JupiterError::InvalidInput(
                "Amount must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_swap_request(&self, request: &SwapRequest) -> Result<(), JupiterError> {
        self.validate_pubkey(&request.user_public_key)?;
        Ok(())
    }

    fn validate_mint_address(&self, address: &str) -> Result<(), JupiterError> {
        if !is_valid_mint_address(address) {
            return Err(JupiterError::InvalidInput(format!(
                "Invalid mint address: {}",
                address
            )));
        }
        Ok(())
    }

    fn validate_pubkey(&self, pubkey: &str) -> Result<(), JupiterError> {
        validate_pubkey(pubkey)
            .map_err(|e| JupiterError::InvalidInput(format!("Invalid public key: {}", e)))?;
        Ok(())
    }
}
