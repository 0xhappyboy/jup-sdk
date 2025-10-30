/// Client-side retry module.
/// Provides intelligent retry, error classification, and recovery strategies.
use crate::types::JupiterError;
use std::time::Duration;
use tokio::time;

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }
    }
}

/// Trait defining retry strategy behavior.
pub trait RetryStrategy {
    /// Determines if a retry should be attempted based on the error and attempt count.
    ///
    /// # Params
    /// error - The error that occurred
    /// attempt - The current attempt number (1-based)
    ///
    /// # Example
    /// ```
    /// # use crate::JupiterError;
    /// # struct MyStrategy;
    /// # impl RetryStrategy for MyStrategy {
    /// #     fn should_retry(&self, error: &JupiterError, attempt: u32) -> bool {
    /// #         attempt < 3 && !error.is_fatal()
    /// #     }
    /// #     fn get_delay(&self, attempt: u32) -> Duration {
    /// #         Duration::from_millis(100 * attempt as u64)
    /// #     }
    /// # }
    /// let strategy = MyStrategy;
    /// let error = JupiterError::NetworkError;
    /// assert!(strategy.should_retry(&error, 1));
    /// ```
    fn should_retry(&self, error: &JupiterError, attempt: u32) -> bool;

    /// Calculates the delay before the next retry attempt.
    ///
    /// # Params
    /// attempt - The current attempt number (1-based)
    ///
    /// # Example
    /// ```
    /// # use std::time::Duration;
    /// # struct MyStrategy;
    /// # impl RetryStrategy for MyStrategy {
    /// #     fn should_retry(&self, _error: &JupiterError, _attempt: u32) -> bool { true }
    /// #     fn get_delay(&self, attempt: u32) -> Duration {
    /// #         Duration::from_millis(100 * attempt as u64)
    /// #     }
    /// # }
    /// let strategy = MyStrategy;
    /// assert_eq!(strategy.get_delay(1), Duration::from_millis(100));
    /// assert_eq!(strategy.get_delay(2), Duration::from_millis(200));
    /// ```
    fn get_delay(&self, attempt: u32) -> Duration;
}

/// Categorizes errors for appropriate handling.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    /// Network connectivity issues
    Network,
    /// Server errors (5xx status codes)
    Server,
    /// Rate limiting exceeded
    RateLimit,
    /// Client errors (4xx status codes)
    Client,
    /// Transaction-related errors
    Transaction,
    /// Unknown or unclassified errors
    Unknown,
}
