//! Error types for spray operations

use thiserror::Error;

/// Errors that can occur during spray operations
#[derive(Debug, Error)]
pub enum SprayError {
    #[error("Failed to setup test environment: {0}")]
    EnvironmentError(String),

    #[error("Failed to start daemon: {0}")]
    DaemonError(String),

    #[error("Test execution failed: {0}")]
    TestError(String),

    #[error("Contract error: {0}")]
    ContractError(#[from] musk::ContractError),

    #[error("Spend error: {0}")]
    SpendError(#[from] musk::SpendError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("RPC error: {0}")]
    RpcError(String),
}

