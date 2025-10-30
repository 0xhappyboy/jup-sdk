use crate::types::{QuoteResponse, TokenInfo};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

/// Validates a Solana public key string and converts it to a Pubkey
///
/// # Arguments
/// address - A string slice representing the public key
///
/// # Returns
/// Result<Pubkey, String> - Ok(Pubkey) if valid, Err(String) if invalid
///
/// # Example
/// ```rust
/// let pubkey_str = "So11111111111111111111111111111111111111112";
/// match validate_pubkey(pubkey_str) {
///     Ok(pubkey) => println!("Valid pubkey: {}", pubkey),
///     Err(e) => println!("Invalid pubkey: {}", e),
/// }
/// ```
pub fn validate_pubkey(address: &str) -> Result<Pubkey, String> {
    Pubkey::from_str(address).map_err(|e| e.to_string())
}

/// Calculates the minimum amount after applying slippage
///
/// # Arguments
/// amount - The original amount
/// slippage_bps - Slippage in basis points (1 basis point = 0.01%)
///
/// # Returns
/// u64 - The amount after slippage deduction
///
/// # Example
/// ```rust
/// let amount = 1000000;
/// let slippage_bps = 50; // 0.5%
/// let min_amount = cal_slippage_amount(amount, slippage_bps);
/// println!("Minimum amount after slippage: {}", min_amount);
/// ```
pub fn cal_slippage_amount(amount: u64, slippage_bps: u16) -> u64 {
    let slippage_percent = slippage_bps as f64 / 10000.0;
    (amount as f64 * (1.0 - slippage_percent)) as u64
}

/// Formats a raw token amount to human-readable format with decimals
///
/// # Arguments
/// amount - The raw token amount
/// decimals - Number of decimal places for the token
///
/// # Returns
/// String - Formatted amount string
///
/// # Example
/// ```rust
/// let raw_amount = 1234567890;
/// let decimals = 9;
/// let formatted = format_amount(raw_amount, decimals);
/// println!("Formatted amount: {}", formatted); // "1.234567890"
/// ```
pub fn format_amount(amount: u64, decimals: u8) -> String {
    let factor = 10u64.pow(decimals as u32);
    let whole = amount / factor;
    let fractional = amount % factor;

    if fractional == 0 {
        format!("{}", whole)
    } else {
        format!(
            "{}.{:0>width$}",
            whole,
            fractional,
            width = decimals as usize
        )
    }
}

/// Parses a human-readable amount string into raw token amount
///
/// # Arguments
/// amount_str - String representation of the amount
/// decimals - Number of decimal places for the token
///
/// # Returns
/// Result<u64, String> - Raw amount if successful, error message if failed
///
/// # Example
/// ```rust
/// let amount_str = "1.5";
/// let decimals = 9;
/// match parse_amount(amount_str, decimals) {
///     Ok(raw_amount) => println!("Raw amount: {}", raw_amount), // 1500000000
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
pub fn parse_amount(amount_str: &str, decimals: u8) -> Result<u64, String> {
    let parts: Vec<&str> = amount_str.split('.').collect();

    match parts.len() {
        1 => {
            let whole = parts[0]
                .parse::<u64>()
                .map_err(|e| format!("Invalid amount: {}", e))?;
            Ok(whole * 10u64.pow(decimals as u32))
        }
        2 => {
            let whole = parts[0]
                .parse::<u64>()
                .map_err(|e| format!("Invalid amount: {}", e))?;
            let fractional = parts[1];

            // 确保小数部分不超过精度
            if fractional.len() > decimals as usize {
                return Err(format!("Too many decimal places. Maximum is {}", decimals));
            }

            let fractional_padded = format!("{:0<width$}", fractional, width = decimals as usize);
            let fractional_value = fractional_padded
                .parse::<u64>()
                .map_err(|e| format!("Invalid fractional part: {}", e))?;

            Ok(whole * 10u64.pow(decimals as u32) + fractional_value)
        }
        _ => Err("Invalid amount format".to_string()),
    }
}

/// Validates that slippage is within acceptable limits
///
/// # Arguments
/// slippage_bps - Slippage in basis points
///
/// # Returns
/// Result<(), String> - Ok(()) if valid, Err if exceeds maximum
///
/// # Example
/// ```rust
/// let slippage_bps = 500; // 5%
/// if let Err(e) = validate_slippage_bps(slippage_bps) {
///     println!("Slippage validation failed: {}", e);
/// }
/// ```
pub fn validate_slippage_bps(slippage_bps: u16) -> Result<(), String> {
    if slippage_bps > 1000 {
        Err("Slippage must be <= 1000 (10%)".to_string())
    } else {
        Ok(())
    }
}

/// Calculates the minimum output amount considering slippage
///
/// # Arguments
/// out_amount - The expected output amount
/// slippage_bps - Slippage in basis points
///
/// # Returns
/// u64 - Minimum acceptable output amount
///
/// # Example
/// ```rust
/// let expected_output = 1000000;
/// let slippage_bps = 100; // 1%
/// let min_output = cal_minimum_out_amount(expected_output, slippage_bps);
/// println!("Minimum output: {}", min_output);
/// ```
pub fn cal_minimum_out_amount(out_amount: u64, slippage_bps: u16) -> u64 {
    cal_slippage_amount(out_amount, slippage_bps)
}

/// Checks if a string is a valid mint address
///
/// # Arguments
/// address - String to validate as a mint address
///
/// # Returns
/// bool - True if valid mint address
///
/// # Example
/// ```rust
/// let address = "So11111111111111111111111111111111111111112";
/// if is_valid_mint_address(address) {
///     println!("Valid mint address");
/// }
/// ```
pub fn is_valid_mint_address(address: &str) -> bool {
    validate_pubkey(address).is_ok()
}

/// Generates a unique nonce based on current time
///
/// # Returns
/// u64 - Unique nonce value
///
/// # Example
/// ```rust
/// let nonce = generate_nonce();
/// println!("Generated nonce: {}", nonce);
/// ```
pub fn generate_nonce() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

/// Calculates price impact percentage for a trade
///
/// # Arguments
/// input_amount - Amount of input token
/// output_amount - Amount of output token
/// spot_price - Current spot price of input token in output token terms
///
/// # Returns
/// f64 - Price impact percentage
///
/// # Example
/// ```rust
/// let input_amount = 1000000;
/// let output_amount = 500000;
/// let spot_price = 0.6;
/// let impact = cal_price_impact(input_amount, output_amount, spot_price);
/// println!("Price impact: {:.2}%", impact);
/// ```
pub fn cal_price_impact(input_amount: u64, output_amount: u64, spot_price: f64) -> f64 {
    let expected_output = (input_amount as f64) * spot_price;
    if expected_output == 0.0 {
        return 0.0;
    }
    (expected_output - output_amount as f64) / expected_output * 100.0
}

/// Validates transaction signature format
///
/// # Arguments
/// signature - Transaction signature string
///
/// # Returns
/// bool - True if signature has valid format
///
/// # Example
/// ```rust
/// let sig = "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYvC7j45R5...";
/// if validate_transaction_signature(sig) {
///     println!("Valid signature format");
/// }
/// ```
pub fn validate_transaction_signature(signature: &str) -> bool {
    signature.len() == 88 && signature.chars().all(|c| c.is_ascii_hexdigit())
}

// ============================

/// Calculates net output amount after deducting fees
///
/// # Arguments
/// quote - Quote response from swap
/// input_token - Input token information
/// output_token - Output token information
/// additional_fees_bps - Additional fees in basis points
///
/// # Returns
/// Result<u64, String> - Net output amount after fees
///
/// # Example
/// ```rust
/// let net_output = cal_net_output(
///     &quote,
///     &input_token,
///     &output_token,
///     10, // 0.1% additional fee
/// )?;
/// println!("Net output after fees: {}", net_output);
/// ```
pub fn cal_net_output(
    quote: &QuoteResponse,
    input_token: &TokenInfo,
    output_token: &TokenInfo,
    additional_fees_bps: u16,
) -> Result<u64, String> {
    let out_amount: u64 = quote.out_amount.parse().map_err(|e| format!("{:?}", e))?;

    // 考虑平台手续费
    let platform_fee = if let Some(fee) = &quote.platform_fee {
        fee.amount.parse().unwrap_or(0)
    } else {
        0
    };

    // 考虑额外手续费
    let additional_fee = (out_amount as f64 * additional_fees_bps as f64 / 10000.0) as u64;

    Ok(out_amount
        .saturating_sub(platform_fee)
        .saturating_sub(additional_fee))
}

/// Estimates annual percentage yield for a trade
///
/// # Arguments
/// input_amount - Amount of input token
/// output_amount - Amount of output token
/// input_token - Input token information
/// output_token - Output token information
/// time_frame_hours - Time frame in hours for the trade
///
/// # Returns
/// f64 - Estimated APY percentage
///
/// # Example
/// ```
/// let apy = estimate_apy(
///     input_amount,
///     output_amount,
///     &input_token,
///     &output_token,
///     24.0, // 24-hour timeframe
/// );
/// println!("Estimated APY: {:.2}%", apy * 100.0);
/// ```
pub fn estimate_apy(
    input_amount: u64,
    output_amount: u64,
    input_token: &TokenInfo,
    output_token: &TokenInfo,
    time_frame_hours: f64,
) -> f64 {
    let input_value = input_amount as f64 / 10f64.powi(input_token.decimals as i32);
    let output_value = output_amount as f64 / 10f64.powi(output_token.decimals as i32);

    if input_value == 0.0 || output_value <= input_value {
        return 0.0;
    }

    let profit_ratio = (output_value - input_value) / input_value;
    let periods_per_year = 365.0 * 24.0 / time_frame_hours;

    (1.0 + profit_ratio).powf(periods_per_year) - 1.0
}

/// Builds a HashMap of token addresses to token information
///
/// # Arguments
/// tokens - Slice of TokenInfo references
///
/// # Returns
/// HashMap<String, &TokenInfo> - Mapping of address to token info
///
/// # Example
/// ```
/// let token_map = TokenUtils::build_token_map(&tokens);
/// if let Some(token) = token_map.get("So111...11112") {
///     println!("Found token: {}", token.symbol);
/// }
/// ```
pub fn build_token_map(tokens: &[TokenInfo]) -> HashMap<String, &TokenInfo> {
    tokens
        .iter()
        .map(|token| (token.address.clone(), token))
        .collect()
}

/// Finds tokens by symbol using fuzzy matching
///
/// # Arguments
/// tokens - Slice of TokenInfo references
/// symbol - Symbol to search for
/// threshold - Similarity threshold (0.0 to 1.0)
///
/// # Returns
/// Vec<&TokenInfo> - Vector of matching tokens
///
/// # Example
/// ```rust
/// let matches = TokenUtils::find_tokens_by_symbol_fuzzy(
///     &tokens,
///     "SOL",
///     0.7, // 70% similarity threshold
/// );
/// for token in matches {
///     println!("Found: {} - {}", token.symbol, token.name);
/// }
/// ```
pub fn find_tokens_by_symbol_fuzzy<'a>(
    tokens: &'a [TokenInfo],
    symbol: &str,
    threshold: f64,
) -> Vec<&'a TokenInfo> {
    tokens
        .iter()
        .filter(|token| {
            let similarity = cal_similarity(&token.symbol.to_lowercase(), &symbol.to_lowercase());
            similarity >= threshold
        })
        .collect()
}

fn cal_similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 {
        return 1.0;
    }
    let common_chars = s1.chars().filter(|c| s2.contains(*c)).count();
    if common_chars == 0 {
        return 0.0;
    }
    common_chars as f64 / s1.len().max(s2.len()) as f64
}
