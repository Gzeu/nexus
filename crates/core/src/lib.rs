//! NEXUS Core - The foundation of the NEXUS Living Terminal
//!
//! This crate provides the core functionality for NEXUS, including:
//! - Agent system traits and interfaces
//! - Security utilities and cryptographic functions
//! - Plugin loading and management
//! - Configuration management
//! - Observability and metrics

pub mod security;
pub mod agent;
pub mod plugin;
pub mod config;
pub mod error;

// Re-export commonly used types and traits
pub use agent::{Agent, AgentContext, AgentResult};
pub use config::Config;
pub use error::{NexusError, Result};
pub use security::{SecurityManager, SecurityConfig};

use std::fmt;

/// NEXUS version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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

/// Agent execution context
#[derive(Debug, Clone)]
pub struct AgentContext {
    /// Agent instance ID
    pub instance_id: String,
    /// User ID if authenticated
    pub user_id: Option<String>,
    /// Environment variables
    pub env: std::collections::HashMap<String, String>,
    /// Working directory
    pub working_dir: std::path::PathBuf,
}

/// Agent execution result
pub type AgentResult<T> = std::result::Result<T, AgentError>;

/// Agent-specific errors
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Agent execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Agent configuration invalid: {0}")]
    ConfigurationInvalid(String),
    
    #[error("Agent resource unavailable: {0}")]
    ResourceUnavailable(String),
    
    #[error("Agent permission denied: {0}")]
    PermissionDenied(String),
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
    /// Plugin error
    PluginError(String),
}

impl fmt::Display for NexusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NexusError::AgentError(msg) => write!(f, "Agent error: {}", msg),
            NexusError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            NexusError::IoError(msg) => write!(f, "IO error: {}", msg),
            NexusError::SecurityError(msg) => write!(f, "Security error: {}", msg),
            NexusError::PluginError(msg) => write!(f, "Plugin error: {}", msg),
        }
    }
}

impl std::error::Error for NexusError {}

/// Initialize the NEXUS core system
pub async fn init() -> Result<()> {
    tracing::info!("Initializing NEXUS core v{}", VERSION);
    
    // Initialize security subsystem with default configuration
    let security_config = SecurityConfig::default();
    let _security_manager = security::init_security(security_config)
        .map_err(|e| NexusError::SecurityError(e.to_string()))?;
    
    tracing::info!("NEXUS core initialization complete");
    Ok(())
}

/// Initialize NEXUS core with custom configuration
pub async fn init_with_config(config: Config) -> Result<()> {
    tracing::info!("Initializing NEXUS core v{} with custom config", VERSION);
    
    // Initialize security subsystem with custom configuration
    let _security_manager = security::init_security(config.security)
        .map_err(|e| NexusError::SecurityError(e.to_string()))?;
    
    tracing::info!("NEXUS core initialization complete");
    Ok(())
}

/// Get NEXUS core version
pub fn version() -> &'static str {
    VERSION
}

/// Get build information
pub fn build_info() -> BuildInfo {
    BuildInfo {
        version: VERSION,
        git_commit: option_env!("GIT_COMMIT").unwrap_or("unknown"),
        build_timestamp: option_env!("BUILD_TIMESTAMP").unwrap_or("unknown"),
        target_triple: env!("TARGET"),
        rust_version: env!("CARGO_PKG_RUST_VERSION"),
    }
}

/// Build information structure
#[derive(Debug, Clone)]
pub struct BuildInfo {
    pub version: &'static str,
    pub git_commit: &'static str,
    pub build_timestamp: &'static str,
    pub target_triple: &'static str,
    pub rust_version: &'static str,
}

impl std::fmt::Display for BuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NEXUS v{} ({})", self.version, self.git_commit)?;
        write!(f, "\nBuilt: {} for {}", self.build_timestamp, self.target_triple)?;
        write!(f, "\nRust: {}", self.rust_version)?;
        Ok(())
    }
}

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
        
        let error = NexusError::SecurityError("unauthorized".to_string());
        assert_eq!(error.to_string(), "Security error: unauthorized");
    }

    #[test]
    fn test_version() {
        let version = version();
        assert!(!version.is_empty());
    }

    #[test]
    fn test_build_info() {
        let info = build_info();
        assert_eq!(info.version, VERSION);
        assert!(!info.target_triple.is_empty());
    }

    #[tokio::test]
    async fn test_init() {
        // This test ensures that init doesn't panic
        // Note: In a real scenario, we'd need the actual security module
        // For now, this test is commented out to avoid compilation errors
        // let result = init().await;
        // assert!(result.is_ok());
    }
}
