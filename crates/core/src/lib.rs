//! NEXUS Core Engine
//! 
//! Provides the foundational traits and types for the NEXUS terminal platform.

use std::fmt;

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
}

impl fmt::Display for NexusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NexusError::AgentError(msg) => write!(f, "Agent error: {}", msg),
            NexusError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            NexusError::IoError(msg) => write!(f, "IO error: {}", msg),
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