//! NEXUS Core Engine
//! 
//! Provides the foundational traits and types for the NEXUS terminal platform.

use std::fmt;

pub mod security {
    pub use super::SecurityManager;
    pub use super::SecurityConfig;
    
    pub mod crypto;
    pub mod validation;
    pub mod audit;
    pub mod config;
    pub mod ratelimit;
    
    use anyhow::Result;
    
    #[derive(Debug, Clone)]
    pub struct SecurityConfig {
        pub encryption_enabled: bool,
        pub min_password_length: usize,
        pub rate_limit: ratelimit::RateLimitConfig,
        pub audit_config: audit::AuditConfig,
        pub validation: validation::ValidationConfig,
    }
    
    impl Default for SecurityConfig {
        fn default() -> Self {
            Self {
                encryption_enabled: true,
                min_password_length: 12,
                rate_limit: ratelimit::RateLimitConfig::default(),
                audit_config: audit::AuditConfig::default(),
                validation: validation::ValidationConfig::default(),
            }
        }
    }
    
    pub struct SecurityManager {
        config: SecurityConfig,
    }
    
    impl SecurityManager {
        pub fn new(config: SecurityConfig) -> Result<Self> {
            Ok(Self { config })
        }
    }
    
    pub fn init_security(config: SecurityConfig) -> Result<SecurityManager> {
        SecurityManager::new(config)
    }
}

// Re-exports for backward compatibility
pub use security::{SecurityManager, SecurityConfig};

/// Agent trait defines the core behavior for NEXUS agents
/// 
/// Agents are autonomous components that can perform tasks, respond to events,
/// and interact with the terminal environment.
pub trait Agent {
    /// Execute the agent's main logic and return a status message
    fn run(&self) -> String;
    
    /// Get the agent's unique identifier/name
    fn name(&self) -> &str {
        "unnamed-agent"
    }
}

/// Agent execution context - simplified version
#[derive(Debug, Clone)]
pub struct AgentContext {
    pub instance_id: String,
}

/// Agent execution result
pub type AgentResult<T> = std::result::Result<T, AgentError>;

/// Agent-specific errors
#[derive(Debug)]
pub struct AgentError {
    pub message: String,
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Agent error: {}", self.message)
    }
}

impl std::error::Error for AgentError {}

/// Simple Config struct
#[derive(Debug, Clone)]
pub struct Config {
    pub security: SecurityConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            security: SecurityConfig::default(),
        }
    }
}

/// Result type for NEXUS operations
pub type Result<T> = std::result::Result<T, NexusError>;

/// Core error types for NEXUS operations
#[derive(Debug, Clone)]
pub enum NexusError {
    /// Agent execution failed
    AgentError(String),
    /// Configuration error
    ConfigError(String),
    /// IO operation failed
    IoError(String),
    /// Security error
    SecurityError(String),
}

impl fmt::Display for NexusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NexusError::AgentError(msg) => write!(f, "Agent error: {}", msg),
            NexusError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            NexusError::IoError(msg) => write!(f, "IO error: {}", msg),
            NexusError::SecurityError(msg) => write!(f, "Security error: {}", msg),
        }
    }
}

impl std::error::Error for NexusError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dummy agent for testing
    struct DummyAgent {
        name: String,
    }

    impl DummyAgent {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl Agent for DummyAgent {
        fn run(&self) -> String {
            "dummy".into()
        }
        
        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn dummy_agent_works() {
        let agent = DummyAgent::new("test-agent");
        assert_eq!(agent.run(), "dummy");
        assert_eq!(agent.name(), "test-agent");
    }

    #[test]
    fn agent_trait_default_name() {
        struct NoNameAgent;
        impl Agent for NoNameAgent {
            fn run(&self) -> String {
                "no-name".into()
            }
        }
        
        let agent = NoNameAgent;
        assert_eq!(agent.name(), "unnamed-agent");
        assert_eq!(agent.run(), "no-name");
    }

    #[test]
    fn nexus_error_display() {
        let error = NexusError::AgentError("test failure".to_string());
        assert_eq!(error.to_string(), "Agent error: test failure");
        
        let error = NexusError::ConfigError("bad config".to_string());
        assert_eq!(error.to_string(), "Config error: bad config");
        
        let error = NexusError::IoError("file not found".to_string());
        assert_eq!(error.to_string(), "IO error: file not found");
    }
}
