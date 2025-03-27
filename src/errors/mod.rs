//! Common error types for the Runar system
//! 
//! This module defines domain-specific error types for different subsystems
//! in the Runar platform. It aims to provide a consistent error handling
//! approach across the codebase.

use thiserror::Error;

/// Core errors related to service operations
#[derive(Error, Debug)]
pub enum ServiceError {
    /// Service not found
    #[error("Service '{0}' not found")]
    ServiceNotFound(String),
    
    /// Operation not supported by service
    #[error("Operation '{0}' not supported by service '{1}'")]
    UnsupportedOperation(String, String),
    
    /// Authorization failed for operation
    #[error("Authorization failed for operation '{0}' on service '{1}'")]
    AuthorizationFailed(String, String),
    
    /// Invalid request parameters
    #[error("Invalid request parameters: {0}")]
    InvalidRequest(String),
    
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    /// Network error
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    /// Internal service error
    #[error("Internal service error: {0}")]
    Internal(String),
    
    /// Error from another service
    #[error("Error from service '{0}': {1}")]
    ServiceError(String, String),
}

/// Database-related errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// Connection error
    #[error("Database connection error: {0}")]
    ConnectionError(String),
    
    /// Query execution error
    #[error("Query error: {0}")]
    QueryError(String),
    
    /// Migration error
    #[error("Migration error: {0}")]
    MigrationError(String),
    
    /// Record not found
    #[error("Record not found: {0}")]
    NotFound(String),
    
    /// Constraint violation
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    
    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),
}

/// Network-related errors
#[derive(Error, Debug)]
pub enum NetworkError {
    /// Connection error
    #[error("Network connection error: {0}")]
    ConnectionError(String),
    
    /// Timeout error
    #[error("Network timeout: {0}")]
    Timeout(String),
    
    /// Peer not found
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    
    /// Protocol error
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    /// Message delivery error
    #[error("Message delivery error: {0}")]
    MessageDeliveryError(String),
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Missing configuration value
    #[error("Missing configuration value: {0}")]
    MissingValue(String),
    
    /// Invalid configuration value
    #[error("Invalid configuration value for '{0}': {1}")]
    InvalidValue(String, String),
    
    /// Configuration file error
    #[error("Configuration file error: {0}")]
    FileError(String),
}

/// Type conversion errors
#[derive(Error, Debug)]
pub enum ConversionError {
    /// Type conversion failed
    #[error("Failed to convert '{0}' to '{1}': {2}")]
    TypeConversionFailed(String, String, String),
    
    /// Missing field in conversion
    #[error("Missing field '{0}' in conversion")]
    MissingField(String),
    
    /// Invalid format for conversion
    #[error("Invalid format for field '{0}': {1}")]
    InvalidFormat(String, String),
}

/// Helper functions for working with errors
pub mod utils {
    use super::*;
    use crate::types::ValueType;
    
    /// Convert any error to a ServiceError::Internal
    pub fn to_internal_error<E: std::fmt::Display>(err: E) -> ServiceError {
        ServiceError::Internal(err.to_string())
    }
    
    /// Create an error response with proper logging
    pub fn error_response<E: std::fmt::Display>(
        error: E,
        status_code: Option<i32>,
    ) -> (String, ValueType) {
        let error_message = error.to_string();
        let mut error_map = std::collections::HashMap::new();
        
        error_map.insert("message".to_string(), ValueType::String(error_message.clone()));
        
        if let Some(code) = status_code {
            error_map.insert("code".to_string(), ValueType::Number(code as f64));
        }
        
        (error_message, ValueType::Map(error_map))
    }
}

// Re-export common error types
pub use self::utils::{to_internal_error, error_response}; 