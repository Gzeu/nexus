//! NEXUS Core Engine
//! 
//! Provides the foundational traits and types for the NEXUS terminal platform.

use std::fmt;

// Basic security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub encryption_enabled: bool,
    pub min_password_length: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_enabled: true,
            min_password_length: 12,
        }
    }
}

// Basic security manager
pub struct SecurityManager {
    config: SecurityConfig,
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> anyhow::Result<Self> {
        tracing::info!("Initializing security manager");
        Ok(Self { config })
    }
    
    pub fn validate_input(&self, input: &str, input_type: &str) -> anyhow::Result<()> {
        // Basic validation
        match input_type {
            "email" if !input.contains('@') => {
                return Err(anyhow::anyhow!("Invalid email format"));
            }
            "url" if !input.starts_with("http") => {
                return Err(anyhow::anyhow!("Invalid URL format"));
            }
            _ => {}
        }
        
        Ok(())
    }
}

/// Initialize security subsystem
pub fn init_security(config: SecurityConfig) -> anyhow::Result<SecurityManager> {
    SecurityManager::new(config)
}

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

/// Agent trait defines the core behavior for NEXUS agents
pub trait Agent {
    /// Execute the agent's main logic and return a status message
    fn run(&self) -> String;
    
    /// Get the agent's unique identifier/name
    fn name(&self) -> &str {
        "unnamed-agent"
    }
}

/// Agent execution context
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
    
    #[test]
    fn test_security_manager() {
        let config = SecurityConfig::default();
        let manager = init_security(config).unwrap();
        
        // Test basic validation
        assert!(manager.validate_input("test@example.com", "email").is_ok());
        assert!(manager.validate_input("https://example.com", "url").is_ok());
        assert!(manager.validate_input("invalid-email", "email").is_err());
    }
}
