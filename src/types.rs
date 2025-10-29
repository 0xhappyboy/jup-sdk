use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub chain_id: u64,
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
    pub logo_uri: String,
    pub tags: Vec<String>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformFee {
    pub amount: String,
    pub fee_bps: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePlan {
    pub swap_info: SwapInfo,
    pub percent: u8,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRequest {
    pub quote_response: QuoteResponse,
    pub user_public_key: String,
    pub wrap_and_unwrap_sol: Option<bool>,
    pub compute_unit_price: Option<u64>,
    pub prioritization_fee_lamports: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResponse {
    pub swap_transaction: String,
    pub last_valid_block_height: u64,
    pub prioritization_fee_lamports: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceResponse {
    pub id: String,
    pub mint_symbol: String,
    pub vs_token: String,
    pub vs_token_symbol: String,
    pub price: f64,
}
