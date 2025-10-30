/// Jupiter API base URL - v6 quote API endpoint
pub const JUPITER_BASE_URL: &str = "https://quote-api.jup.ag/v6";
/// Default slippage tolerance in basis points (1 basis point = 0.01%)
/// 50 bps = 0.5% slippage tolerance
pub const DEFAULT_SLIPPAGE_BPS: u16 = 50;
/// Default platform fee in basis points
/// 0 bps = no platform fee by default
pub const DEFAULT_FEE_BPS: u16 = 0;
/// Maximum allowed slippage tolerance in basis points
/// 1000 bps = 10% maximum slippage tolerance for safety
pub const MAX_SLIPPAGE_BPS: u16 = 1000;
/// Default HTTP request timeout in seconds
/// Requests will fail if they take longer than this duration
pub const REQUEST_TIMEOUT_SECONDS: u64 = 30;
/// Maximum number of retry attempts for failed requests
/// Only retriable errors (network issues, rate limits) will be retried
pub const MAX_RETRIES: u32 = 3;
/// Delay between retry attempts in milliseconds
/// Uses exponential backoff: delay increases with each retry attempt
pub const RETRY_DELAY_MS: u64 = 500;
