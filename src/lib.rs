use reqwest::Client;
use std::collections::HashMap;

use crate::{
    global::JUPITER_BASE_URL,
    types::{PriceResponse, QuoteRequest, QuoteResponse, SwapRequest, SwapResponse, TokenInfo},
};
pub mod global;
pub mod tool;
pub mod types;

#[derive(Clone)]
pub struct JupiterClient {
    client: Client,
    base_url: String,
}

impl JupiterClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: JUPITER_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<QuoteResponse, String> {
        let url = format!("{}/quote", self.base_url);

        let response = self
            .client
            .get(&url)
            .query(&request)
            .send()
            .await
            .map_err(|e| format!("{:?}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.map_err(|e| format!("{:?}", e))?;
            return Err(format!("{:?}", "get quote error".to_string()));
        }

        let quote: QuoteResponse = response.json().await.map_err(|e| format!("{:?}", e))?;
        Ok(quote)
    }

    pub async fn get_swap_transaction(
        &self,
        request: &SwapRequest,
    ) -> Result<SwapResponse, String> {
        let url = format!("{}/swap", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("{:?}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.map_err(|e| format!("{:?}", e))?;
            return Err(format!("{:?}", "get_swap_transaction".to_string()));
        }

        let swap_response: SwapResponse = response.json().await.map_err(|e| format!("{:?}", e))?;
        Ok(swap_response)
    }

    pub async fn get_tokens(&self) -> Result<Vec<TokenInfo>, String> {
        let url = format!("{}/tokens", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("{:?}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.map_err(|e| format!("{:?}", e))?;
            return Err(format!("{:?}", "get_swap_transaction".to_string()));
        }

        let tokens: Vec<TokenInfo> = response.json().await.map_err(|e| format!("{:?}", e))?;
        Ok(tokens)
    }

    pub async fn get_price(
        &self,
        ids: &[String],
    ) -> Result<HashMap<String, PriceResponse>, String> {
        let url = format!("{}/price", self.base_url);

        let mut params = HashMap::new();
        params.insert("ids", ids.join(","));

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| format!("{:?}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.map_err(|e| format!("{:?}", e))?;
            return Err(format!("{:?}", "get_swap_transaction".to_string()));
        }

        let prices: HashMap<String, PriceResponse> =
            response.json().await.map_err(|e| format!("{:?}", e))?;
        Ok(prices)
    }

    pub async fn get_routes(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<Vec<QuoteResponse>, String> {
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
            .map_err(|e| format!("{:?}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.map_err(|e| format!("{:?}", e))?;
            return Err(format!("{:?}", "get_swap_transaction".to_string()));
        }

        let routes: Vec<QuoteResponse> = response.json().await.map_err(|e| format!("{:?}", e))?;
        Ok(routes)
    }
}

impl Default for JupiterClient {
    fn default() -> Self {
        Self::new()
    }
}
