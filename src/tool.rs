use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub fn validate_pubkey(address: &str) -> Result<Pubkey, String> {
    Pubkey::from_str(address).map_err(|e| e.to_string())
}

pub fn calculate_slippage_amount(amount: u64, slippage_bps: u16) -> u64 {
    let slippage_percent = slippage_bps as f64 / 10000.0;
    (amount as f64 * (1.0 - slippage_percent)) as u64
}

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
