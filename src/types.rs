use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

/// Represents token information including metadata and extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub chain_id: u64,
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
    pub logo_uri: String,
    pub tags: Vec<String>,
    pub extensions: Option<TokenExtensions>,
}

/// Request structure for getting swap quotes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteRequest {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    pub slippage_bps: u16,
    pub fee_bps: Option<u16>,
    pub only_direct_routes: Option<bool>,
    pub as_legacy_transaction: Option<bool>,
    pub restrict_middle_tokens: Option<bool>,
}

/// Response structure containing swap quote details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub other_amount_threshold: String,
    pub swap_mode: String,
    pub slippage_bps: u16,
    pub platform_fee: Option<PlatformFee>,
    pub price_impact_pct: String,
    pub route_plan: Vec<RoutePlan>,
    pub context_slot: u64,
    pub time_taken: f64,
}

/// Platform fee information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformFee {
    pub amount: String,
    pub fee_bps: u16,
}

/// Individual route information within a swap route plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePlan {
    pub swap_info: SwapInfo,
    pub percent: u8,
}

/// Swap information for a specific route step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapInfo {
    pub amm_key: String,
    pub label: String,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub fee_amount: String,
    pub fee_mint: String,
}

/// Request structure for executing a swap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRequest {
    pub quote_response: QuoteResponse,
    pub user_public_key: String,
    pub wrap_and_unwrap_sol: Option<bool>,
    pub compute_unit_price: Option<u64>,
    pub prioritization_fee_lamports: Option<u64>,
}

/// Response structure containing swap transaction details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResponse {
    pub swap_transaction: String,
    pub last_valid_block_height: u64,
    pub prioritization_fee_lamports: Option<u64>,
}

/// Price information response for a token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceResponse {
    pub id: String,
    pub mint_symbol: String,
    pub vs_token: String,
    pub vs_token_symbol: String,
    pub price: f64,
}

/// Token extension metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenExtensions {
    pub coingecko_id: Option<String>,
    pub website: Option<String>,
}

/// Response containing indexed route map data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedRouteMapResponse {
    pub indexed_route_map: IndexedRouteMap,
}

/// Indexed route map structure for efficient route lookup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedRouteMap {
    pub mint_keys: Vec<String>,
    pub indexed_route_map: HashMap<String, Vec<usize>>,
}

/// Error types for Jupiter operations
#[derive(Debug, Clone)]
pub enum JupiterError {
    RequestFailed(String),
    InvalidInput(String),
    NetworkError(String),
    ValidationError(String),
    RateLimitExceeded(String),
    TransactionFailed(String),
    ParseError(String),
    Error(String),
}

impl JupiterError {
    /// Determines if the error is retriable
    pub fn is_retriable(&self) -> bool {
        match self {
            JupiterError::NetworkError(_) => true,
            JupiterError::RequestFailed(msg) => {
                // Retry on 5xx server errors
                msg.contains("500") || msg.contains("502") || msg.contains("503")
            }
            JupiterError::RateLimitExceeded(_) => true,
            JupiterError::InvalidInput(_) => false,
            JupiterError::ParseError(_) => false,
            JupiterError::TransactionFailed(_) => false,
            JupiterError::Error(_) => false,
            JupiterError::ValidationError(_) => false,
        }
    }
}

impl std::fmt::Display for JupiterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JupiterError::RequestFailed(msg) => write!(f, "Request failed: {}", msg),
            JupiterError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            JupiterError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            JupiterError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            JupiterError::Error(msg) => write!(f, "Parse error: {}", msg),
            JupiterError::ValidationError(msg) => write!(f, "Parse error: {}", msg),
            JupiterError::RateLimitExceeded(msg) => write!(f, "Parse error: {}", msg),
            JupiterError::TransactionFailed(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for JupiterError {}

/// Rate limiter for API requests
#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests_per_second: u32,
    // Can be implemented using governor or tower::limit::RateLimit
}

impl RateLimiter {
    /// Creates a new rate limiter with specified requests per second
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            requests_per_second,
        }
    }

    /// Acquires permission to make a request, waiting if necessary
    pub fn acquire(&self) -> impl std::future::Future<Output = ()> {
        // Simplified rate limiting implementation
        // In practice, use governor crate for more precise rate limiting
        async {
            tokio::time::sleep(Duration::from_millis(
                1000 / self.requests_per_second as u64,
            ))
            .await;
        }
    }
}

/// Transaction status types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatusType {
    Pending,
    Confirmed,
    Finalized,
    Failed,
    Timeout,
}

/// Transaction status monitoring - used for tracking transaction confirmation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    /// Transaction signature
    pub signature: String,
    /// Transaction status
    pub status: TransactionStatusType,
    /// Block slot
    pub slot: u64,
    /// Confirmation status
    pub confirmation_status: Option<String>,
    /// Error information (if any)
    pub err: Option<serde_json::Value>,
}

/// Advanced swap configuration - for fine-grained swap control
#[derive(Debug, Clone)]
pub struct AdvancedSwapConfig {
    /// Maximum slippage tolerance (basis points)
    pub max_slippage_bps: u16,
    /// Preferred AMM list
    pub preferred_amms: Vec<String>,
    /// Excluded AMM list
    pub excluded_amms: Vec<String>,
    /// Maximum price impact tolerance (basis points)
    pub max_price_impact_bps: u16,
    /// Whether to use versioned transactions
    pub use_versioned_transaction: bool,
}

impl Default for AdvancedSwapConfig {
    fn default() -> Self {
        Self {
            max_slippage_bps: 50,
            preferred_amms: Vec::new(),
            excluded_amms: Vec::new(),
            max_price_impact_bps: 500, // 5%
            use_versioned_transaction: true,
        }
    }
}

/// Batch quote request - for getting multiple swap quotes in one request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchQuoteRequest {
    /// Multiple quote requests
    pub requests: Vec<QuoteRequest>,
}

/// Batch quote response - contains multiple quote results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchQuoteResponse {
    /// List of quote results
    pub quotes: Vec<QuoteResponse>,
}

/// Swap execution result - encapsulates complete swap operation result
#[derive(Debug, Clone)]
pub struct SwapExecutionResult {
    /// Quote used for the swap
    pub quote: QuoteResponse,
    /// Swap transaction response
    pub swap_response: crate::types::SwapResponse,
    /// Configuration used
    pub config: AdvancedSwapConfig,
}

impl SwapExecutionResult {
    /// Gets the expected output amount
    pub fn get_expected_output(&self) -> u64 {
        self.quote.out_amount.parse().unwrap_or(0)
    }

    /// Gets the minimum output amount considering slippage
    pub fn get_minimum_output(&self) -> u64 {
        let out_amount: u64 = self.quote.out_amount.parse().unwrap_or(0);
        crate::tool::calculate_slippage_amount(out_amount, self.quote.slippage_bps)
    }

    /// Calculates price impact percentage
    pub fn get_price_impact(&self) -> f64 {
        self.quote.price_impact_pct.parse().unwrap_or(0.0)
    }
}
