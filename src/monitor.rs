use crate::types::JupiterError;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_commitment_config::CommitmentConfig;
use solana_network_sdk::Solana;
use solana_sdk::signature::Signature;
use std::str::FromStr;
use std::time::Duration;
use tokio::time;

/// Configuration for transaction monitoring
#[derive(Debug, Clone)]
pub struct TransactionMonitorConfig {
    pub timeout: Duration,
    pub poll_interval: Duration,
    pub commitment: CommitmentConfig,
    pub confirmations_required: u8,
}

impl Default for TransactionMonitorConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            poll_interval: Duration::from_secs(2),
            commitment: CommitmentConfig::confirmed(),
            confirmations_required: 1,
        }
    }
}

/// Transaction status
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Finalized,
    Failed,
    Timeout,
}

/// Transaction monitoring result
#[derive(Debug, Clone)]
pub struct TransactionMonitorResult {
    pub signature: String,
    pub status: TransactionStatus,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub confirmations: Option<u8>,
    pub logs: Vec<String>,
    pub error: Option<String>,
}

/// Transaction monitor for tracking Solana transaction status
pub struct Monitor;

impl Monitor {
    /// Monitors a transaction until it reaches final state or timeout
    ///
    /// # Params
    /// signature - Transaction signature string
    /// solana - Solana client instance
    /// config - Optional monitoring configuration
    ///
    /// # Example
    /// ```rust
    /// use solana_network_sdk::Solana;
    /// use monitor::Monitor;
    /// use std::time::Duration;
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let solana = Solana::new(Mode::MAIN);
    /// let monitor = Monitor;
    /// let signature = "........";
    ///
    /// let result = monitor.monitor_transaction_status(signature, &solana, None).await?;
    /// println!("Transaction status: {:?}", result.status);
    /// Ok(())
    /// }
    /// ```
    pub async fn monitor_transaction_status(
        &self,
        signature: &str,
        solana: &Solana,
        config: Option<TransactionMonitorConfig>,
    ) -> Result<TransactionMonitorResult, JupiterError> {
        let config = config.unwrap_or_default();
        let signature = Signature::from_str(signature)
            .map_err(|e| JupiterError::InvalidInput(e.to_string()))?;
        let start = std::time::Instant::now();
        while start.elapsed() < config.timeout {
            match self
                .check_transaction_status(&signature, solana, &config)
                .await
            {
                Ok(Some(result)) => {
                    if result.status == TransactionStatus::Confirmed
                        || result.status == TransactionStatus::Finalized
                    {
                        return Ok(result);
                    } else if result.status == TransactionStatus::Failed {
                        return Ok(result);
                    }
                    // Continue to wait for confirmation
                }
                Ok(None) => {
                    // The transaction has not yet been seen online; please continue to wait.
                }
                Err(e) => {
                    // Log the error but continue to retry.
                    eprintln!("Error checking transaction status: {}", e);
                }
            }
            time::sleep(config.poll_interval).await;
        }
        // timeout
        Ok(TransactionMonitorResult {
            signature: signature.to_string(),
            status: TransactionStatus::Timeout,
            slot: 0,
            block_time: None,
            confirmations: None,
            logs: Vec::new(),
            error: Some("Transaction monitoring timeout".to_string()),
        })
    }

    /// Check the status of a single transaction
    async fn check_transaction_status(
        &self,
        signature: &Signature,
        solana: &Solana,
        config: &TransactionMonitorConfig,
    ) -> Result<Option<TransactionMonitorResult>, JupiterError> {
        let statuses = solana
            .client
            .clone()
            .ok_or(JupiterError::Error("solana client error".to_string()))?
            .get_signature_statuses(&[*signature])
            .await
            .map_err(|e| JupiterError::NetworkError(e.to_string()))?;
        if let Some(status) = statuses.value.get(0).and_then(|s| s.as_ref()) {
            let slot = status.slot;
            // get transcation lgos
            let logs = self
                .get_transaction_logs(signature, solana)
                .await
                .map_err(|e| JupiterError::Error(format!("get transcation logs error:{:?}", e)))?
                .unwrap();
            // Determine transaction status
            let transaction_status = if status.err.is_some() {
                TransactionStatus::Failed
            } else if status.confirmations.is_none() {
                // No confirmation number indicates final confirmation.
                TransactionStatus::Finalized
            } else if status
                .confirmations
                .map(|c| c >= config.confirmations_required.into())
                .unwrap_or(false)
            {
                TransactionStatus::Confirmed
            } else {
                TransactionStatus::Pending
            };
            // get block time
            let block_time = if slot > 0 {
                solana
                    .client
                    .clone()
                    .ok_or(JupiterError::Error("get block time error".to_string()))?
                    .get_block_time(slot)
                    .await
                    .map_err(|e| JupiterError::Error(format!("get block time error:{:?}", e)))?
            } else {
                0
            };
            let result = TransactionMonitorResult {
                signature: signature.to_string(),
                status: transaction_status,
                slot,
                block_time: Some(block_time),
                confirmations: status.confirmations.map(|c| c as u8),
                logs: logs,
                error: status.err.clone().map(|e| e.to_string()),
            };

            return Ok(Some(result));
        }
        if let Ok(Some(result)) = self.check_via_transaction(signature, solana, config).await {
            return Ok(Some(result));
        }
        Ok(None)
    }

    /// Check the transaction status using get_transaction
    async fn check_via_transaction(
        &self,
        signature: &Signature,
        solana: &Solana,
        config: &TransactionMonitorConfig,
    ) -> Result<Option<TransactionMonitorResult>, JupiterError> {
        let transaction_config = RpcTransactionConfig {
            encoding: None,
            commitment: Some(config.commitment),
            max_supported_transaction_version: Some(0),
        };
        match solana
            .client
            .clone()
            .ok_or(JupiterError::Error("get block time error".to_string()))?
            .get_transaction_with_config(signature, transaction_config)
            .await
        {
            Ok(transaction) => {
                let slot = transaction.slot;
                let block_time = transaction.block_time;
                let logs = transaction
                    .transaction
                    .meta
                    .and_then(|meta| match meta.log_messages {
                        solana_transaction_status::option_serializer::OptionSerializer::Some(
                            logs,
                        ) => Some(logs),
                        _ => None,
                    })
                    .unwrap_or_default();
                let result = TransactionMonitorResult {
                    signature: signature.to_string(),
                    status: TransactionStatus::Confirmed, // 如果能获取到交易，认为是已确认
                    slot,
                    block_time,
                    confirmations: Some(config.confirmations_required),
                    logs,
                    error: None,
                };
                Ok(Some(result))
            }
            Err(_) => Ok(None),
        }
    }

    /// get transca
    async fn get_transaction_logs(
        &self,
        signature: &Signature,
        solana: &Solana,
    ) -> Result<Option<Vec<String>>, JupiterError> {
        let transaction_config = RpcTransactionConfig {
            encoding: None,
            commitment: None,
            max_supported_transaction_version: Some(0),
        };
        match solana
            .client
            .clone()
            .ok_or(JupiterError::Error("solana client error".to_string()))?
            .get_transaction_with_config(signature, transaction_config)
            .await
        {
            Ok(transaction) => {
                Ok(transaction
                    .transaction
                    .meta
                    .and_then(|meta| match meta.log_messages {
                        solana_transaction_status::option_serializer::OptionSerializer::Some(
                            logs,
                        ) => Some(logs),
                        _ => None,
                    }))
            }
            Err(_) => Err(JupiterError::Error(
                "transaction does not exist".to_string(),
            )),
        }
    }

    /// Monitors multiple transactions concurrently
    ///
    /// # Params
    /// signatures - Slice of transaction signature strings
    /// solana - Solana client instance
    /// config - Optional monitoring configuration
    ///
    /// # Example
    /// ```rust
    /// use solana_network_sdk::Solana;
    /// use monitor::Monitor;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let solana = Solana::new(Mode::MAIN);
    /// let monitor = Monitor;
    /// let signatures = vec![
    ///     "...".to_string(),
    ///     ".....".to_string(),
    /// ];
    ///
    /// let results = monitor.monitor_transactions_batch(&signatures, &solana, None).await?;
    /// for result in results {
    ///     println!("Signature: {}, Status: {:?}", result.signature, result.status);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn monitor_transactions_batch(
        &self,
        signatures: &[String],
        solana: &Solana,
        config: Option<TransactionMonitorConfig>,
    ) -> Result<Vec<TransactionMonitorResult>, JupiterError> {
        let mut results = Vec::new();
        let config = config.unwrap_or_default();
        for signature in signatures {
            match self
                .monitor_transaction_status(signature, solana, Some(config.clone()))
                .await
            {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(TransactionMonitorResult {
                        signature: signature.clone(),
                        status: TransactionStatus::Failed,
                        slot: 0,
                        block_time: None,
                        confirmations: None,
                        logs: Vec::new(),
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        Ok(results)
    }
}
