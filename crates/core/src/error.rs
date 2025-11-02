//! Error types for NEXUS Core
//!
//! This module defines the error types used throughout the NEXUS system
//! with comprehensive error categorization and context.

use std::fmt;
use thiserror::Error;

/// Main result type for NEXUS operations
pub type Result<T> = std::result::Result<T, NexusError>;

/// Core error types for NEXUS operations
#[derive(Debug, Error)]
pub enum NexusError {
    /// Agent-related errors
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// I/O operation errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Security-related errors
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    /// Plugin-related errors
    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),
    
    /// Network-related errors
    #[error("Network error: {0}")]
    Network(String),
    
    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// Generic internal errors
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// External dependency errors
    #[error("External error: {0}")]
    External(#[from] anyhow::Error),
}

/// Agent-specific errors
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Agent configuration invalid: {0}")]
    ConfigurationInvalid(String),
    
    #[error("Agent resource unavailable: {0}")]
    ResourceUnavailable(String),
    
    #[error("Agent permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Agent timeout: {0}")]
    Timeout(String),
    
    #[error("Agent security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Agent resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Agent not found: {0}")]
    NotFound(String),
    
    #[error("Agent already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Agent initialization failed: {0}")]
    InitializationFailed(String),
}

/// Security-related errors
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),
    
    #[error("Cryptographic operation failed: {0}")]
    CryptographicError(String),
    
    #[error("Invalid input detected: {0}")]
    InvalidInput(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Security policy violation: {0}")]
    PolicyViolation(String),
    
    #[error("Key management error: {0}")]
    KeyManagementError(String),
    
    #[error("Audit log error: {0}")]
    AuditLogError(String),
    
    #[error("Certificate error: {0}")]
    CertificateError(String),
    
    #[error("Encryption/decryption failed: {0}")]
    EncryptionError(String),
}

/// Plugin-related errors
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin loading failed: {0}")]
    LoadingFailed(String),
    
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Plugin already loaded: {0}")]
    AlreadyLoaded(String),
    
    #[error("Plugin dependency error: {0}")]
    DependencyError(String),
    
    #[error("Plugin signature verification failed: {0}")]
    SignatureVerificationFailed(String),
    
    #[error("Plugin permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Plugin version incompatible: {0}")]
    VersionIncompatible(String),
    
    #[error("Plugin execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Plugin configuration invalid: {0}")]
    ConfigurationInvalid(String),
}

/// Web3-related errors
#[cfg(feature = "web3")]
#[derive(Debug, Error)]
pub enum Web3Error {
    #[error("RPC connection failed: {0}")]
    RpcConnectionFailed(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Contract call failed: {0}")]
    ContractCallFailed(String),
    
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    
    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),
    
    #[error("Gas estimation failed: {0}")]
    GasEstimationFailed(String),
    
    #[error("Private key error: {0}")]
    PrivateKeyError(String),
    
    #[error("Network not supported: {0}")]
    NetworkNotSupported(String),
    
    #[error("Wallet connection failed: {0}")]
    WalletConnectionFailed(String),
}

/// Validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Input too long: {length} > {max_length}")]
    InputTooLong { length: usize, max_length: usize },
    
    #[error("Potential XSS attack detected: {pattern}")]
    XssDetected { pattern: String },
    
    #[error("Potential SQL injection detected: {pattern}")]
    SqlInjectionDetected { pattern: String },
    
    #[error("Path traversal attempt detected: {path}")]
    PathTraversalDetected { path: String },
    
    #[error("Command injection attempt detected: {pattern}")]
    CommandInjectionDetected { pattern: String },
    
    #[error("Invalid characters detected: {chars}")]
    InvalidCharacters { chars: String },
    
    #[error("Empty input not allowed for type: {input_type}")]
    EmptyInput { input_type: String },
    
    #[error("Invalid format for type {input_type}: {details}")]
    InvalidFormat { input_type: String, details: String },
}

/// Error context for better debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Component where the error occurred
    pub component: String,
    /// Operation that was being performed
    pub operation: String,
    /// Additional context information
    pub details: std::collections::HashMap<String, String>,
    /// Timestamp when the error occurred
    pub timestamp: std::time::SystemTime,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(component: &str, operation: &str) -> Self {
        Self {
            component: component.to_string(),
            operation: operation.to_string(),
            details: std::collections::HashMap::new(),
            timestamp: std::time::SystemTime::now(),
        }
    }
    
    /// Add detail to the error context
    pub fn with_detail(mut self, key: &str, value: &str) -> Self {
        self.details.insert(key.to_string(), value.to_string());
        self
    }
}

/// Error with context information
#[derive(Debug)]
pub struct ContextualError {
    pub error: NexusError,
    pub context: ErrorContext,
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}::{}] {}", self.context.component, self.context.operation, self.error)?;
        
        if !self.context.details.is_empty() {
            write!(f, " (")?;
            for (i, (key, value)) in self.context.details.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}: {}", key, value)?;
            }
            write!(f, ")")?;
        }
        
        Ok(())
    }
}

impl std::error::Error for ContextualError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Trait for adding context to errors
pub trait ErrorExt<T> {
    /// Add context to an error
    fn with_context(self, context: ErrorContext) -> std::result::Result<T, ContextualError>;
    
    /// Add simple context to an error
    fn with_simple_context(self, component: &str, operation: &str) -> std::result::Result<T, ContextualError>;
}

impl<T> ErrorExt<T> for Result<T> {
    fn with_context(self, context: ErrorContext) -> std::result::Result<T, ContextualError> {
        self.map_err(|error| ContextualError { error, context })
    }
    
    fn with_simple_context(self, component: &str, operation: &str) -> std::result::Result<T, ContextualError> {
        self.with_context(ErrorContext::new(component, operation))
    }
}

/// Convert from validation error to security error
impl From<ValidationError> for SecurityError {
    fn from(err: ValidationError) -> Self {
        SecurityError::InvalidInput(err.to_string())
    }
}

/// Convert from validation error to NEXUS error
impl From<ValidationError> for NexusError {
    fn from(err: ValidationError) -> Self {
        NexusError::Security(SecurityError::from(err))
    }
}

#[cfg(feature = "web3")]
impl From<Web3Error> for NexusError {
    fn from(err: Web3Error) -> Self {
        NexusError::Network(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let error = NexusError::Config("Invalid configuration".to_string());
        assert_eq!(error.to_string(), "Configuration error: Invalid configuration");
        
        let agent_error = AgentError::ExecutionFailed("Test failure".to_string());
        let nexus_error = NexusError::Agent(agent_error);
        assert_eq!(nexus_error.to_string(), "Agent error: Agent execution failed: Test failure");
    }
    
    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test-component", "test-operation")
            .with_detail("user_id", "123")
            .with_detail("action", "create");
        
        assert_eq!(context.component, "test-component");
        assert_eq!(context.operation, "test-operation");
        assert_eq!(context.details.len(), 2);
    }
    
    #[test]
    fn test_contextual_error() {
        let error = NexusError::Config("Test error".to_string());
        let context = ErrorContext::new("config", "load")
            .with_detail("file", "test.toml");
        
        let contextual_error = ContextualError { error, context };
        let error_string = contextual_error.to_string();
        
        assert!(error_string.contains("config::load"));
        assert!(error_string.contains("Configuration error: Test error"));
        assert!(error_string.contains("file: test.toml"));
    }
    
    #[test]
    fn test_error_ext_trait() {
        let result: Result<()> = Err(NexusError::Config("Test".to_string()));
        let contextual_result = result.with_simple_context("test", "operation");
        
        assert!(contextual_result.is_err());
        let err = contextual_result.unwrap_err();
        assert_eq!(err.context.component, "test");
        assert_eq!(err.context.operation, "operation");
    }
    
    #[test]
    fn test_validation_error_conversion() {
        let validation_error = ValidationError::InputTooLong {
            length: 100,
            max_length: 50,
        };
        
        let security_error: SecurityError = validation_error.into();
        assert!(matches!(security_error, SecurityError::InvalidInput(_)));
        
        let nexus_error: NexusError = ValidationError::XssDetected {
            pattern: "<script>".to_string(),
        }.into();
        assert!(matches!(nexus_error, NexusError::Security(_)));
    }
}
