//! Agent system for NEXUS
//!
//! This module provides the core agent traits and implementations
//! for building intelligent, secure, and extensible agents.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};
use tracing::{info, warn, error};

use crate::security::SecurityManager;

/// Agent execution context with security features
#[derive(Debug, Clone)]
pub struct AgentContext {
    /// Agent instance ID
    pub instance_id: String,
    /// User ID if authenticated
    pub user_id: Option<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Working directory
    pub working_dir: PathBuf,
    /// Security manager reference
    pub security_manager: Option<String>, // ID reference to security manager
    /// Agent permissions
    pub permissions: AgentPermissions,
    /// Resource limits
    pub limits: ResourceLimits,
}

/// Agent permissions
#[derive(Debug, Clone)]
pub struct AgentPermissions {
    /// Can read files
    pub can_read_files: bool,
    /// Can write files
    pub can_write_files: bool,
    /// Can execute commands
    pub can_execute_commands: bool,
    /// Can access network
    pub can_access_network: bool,
    /// Can access Web3 functions
    pub can_access_web3: bool,
    /// Allowed file paths (if file access is permitted)
    pub allowed_paths: Vec<PathBuf>,
}

impl Default for AgentPermissions {
    fn default() -> Self {
        Self {
            can_read_files: false,
            can_write_files: false,
            can_execute_commands: false,
            can_access_network: false,
            can_access_web3: false,
            allowed_paths: Vec::new(),
        }
    }
}

/// Resource limits for agent execution
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<u64>,
    /// Maximum execution time in seconds
    pub max_execution_time_secs: Option<u64>,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: Option<f32>,
    /// Maximum file operations per second
    pub max_file_ops_per_sec: Option<u32>,
    /// Maximum network requests per minute
    pub max_network_requests_per_min: Option<u32>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(100 * 1024 * 1024), // 100MB
            max_execution_time_secs: Some(300), // 5 minutes
            max_cpu_percent: Some(50.0), // 50%
            max_file_ops_per_sec: Some(100),
            max_network_requests_per_min: Some(1000),
        }
    }
}

/// Agent execution result
pub type AgentResult<T> = Result<T, AgentError>;

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
    
    #[error("Agent timeout: {0}")]
    Timeout(String),
    
    #[error("Agent security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Agent resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
}

/// Enhanced Agent trait with async support and security
#[async_trait]
pub trait Agent: Send + Sync {
    /// Execute the agent's main logic
    async fn execute(&self, context: &AgentContext) -> AgentResult<AgentOutput>;
    
    /// Get the agent's unique identifier/name
    fn name(&self) -> &str;
    
    /// Get the agent's version
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    /// Get the agent's description
    fn description(&self) -> &str {
        "No description available"
    }
    
    /// Get required permissions for this agent
    fn required_permissions(&self) -> AgentPermissions {
        AgentPermissions::default()
    }
    
    /// Get resource limits for this agent
    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::default()
    }
    
    /// Initialize the agent (called once before execution)
    async fn initialize(&mut self, _context: &AgentContext) -> AgentResult<()> {
        Ok(())
    }
    
    /// Cleanup the agent (called after execution)
    async fn cleanup(&mut self, _context: &AgentContext) -> AgentResult<()> {
        Ok(())
    }
    
    /// Health check for the agent
    async fn health_check(&self) -> AgentResult<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }
    
    /// Validate input before execution
    async fn validate_input(&self, input: &AgentInput) -> AgentResult<()> {
        if input.data.is_empty() {
            return Err(AgentError::ConfigurationInvalid(
                "Input data cannot be empty".to_string()
            ));
        }
        Ok(())
    }
}

/// Agent input data
#[derive(Debug, Clone)]
pub struct AgentInput {
    /// Input data
    pub data: HashMap<String, serde_json::Value>,
    /// Input metadata
    pub metadata: HashMap<String, String>,
    /// Request ID for tracking
    pub request_id: Option<String>,
}

/// Agent output data
#[derive(Debug, Clone)]
pub struct AgentOutput {
    /// Output data
    pub data: HashMap<String, serde_json::Value>,
    /// Output metadata
    pub metadata: HashMap<String, String>,
    /// Success status
    pub success: bool,
    /// Status message
    pub message: String,
    /// Execution metrics
    pub metrics: ExecutionMetrics,
}

/// Agent execution metrics
#[derive(Debug, Clone, Default)]
pub struct ExecutionMetrics {
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Memory used in bytes
    pub memory_used_bytes: u64,
    /// CPU usage percentage
    pub cpu_usage_percent: f32,
    /// Number of file operations
    pub file_operations: u32,
    /// Number of network requests
    pub network_requests: u32,
}

/// Agent health status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Agent manager for orchestrating multiple agents
pub struct AgentManager {
    agents: HashMap<String, Box<dyn Agent>>,
    security_manager: Option<SecurityManager>,
    command_tx: mpsc::UnboundedSender<AgentCommand>,
    command_rx: Option<mpsc::UnboundedReceiver<AgentCommand>>,
}

/// Agent management commands
#[derive(Debug)]
pub enum AgentCommand {
    Execute {
        agent_name: String,
        input: AgentInput,
        context: AgentContext,
        response_tx: oneshot::Sender<AgentResult<AgentOutput>>,
    },
    HealthCheck {
        agent_name: String,
        response_tx: oneshot::Sender<AgentResult<HealthStatus>>,
    },
    Shutdown,
}

impl AgentManager {
    /// Create a new agent manager
    pub fn new(security_manager: Option<SecurityManager>) -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        
        Self {
            agents: HashMap::new(),
            security_manager,
            command_tx,
            command_rx: Some(command_rx),
        }
    }
    
    /// Register an agent
    pub fn register_agent(&mut self, agent: Box<dyn Agent>) -> Result<()> {
        let name = agent.name().to_string();
        
        if self.agents.contains_key(&name) {
            return Err(anyhow::anyhow!("Agent '{}' is already registered", name));
        }
        
        info!("Registering agent: {}", name);
        self.agents.insert(name, agent);
        Ok(())
    }
    
    /// Execute an agent
    pub async fn execute_agent(
        &self,
        agent_name: &str,
        input: AgentInput,
        context: AgentContext,
    ) -> AgentResult<AgentOutput> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.command_tx.send(AgentCommand::Execute {
            agent_name: agent_name.to_string(),
            input,
            context,
            response_tx,
        }).map_err(|e| AgentError::ExecutionFailed(format!("Command send failed: {}", e)))?;
        
        response_rx.await
            .map_err(|e| AgentError::ExecutionFailed(format!("Response receive failed: {}", e)))?
    }
    
    /// Get agent health status
    pub async fn get_agent_health(&self, agent_name: &str) -> AgentResult<HealthStatus> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.command_tx.send(AgentCommand::HealthCheck {
            agent_name: agent_name.to_string(),
            response_tx,
        }).map_err(|e| AgentError::ExecutionFailed(format!("Command send failed: {}", e)))?;
        
        response_rx.await
            .map_err(|e| AgentError::ExecutionFailed(format!("Response receive failed: {}", e)))?
    }
    
    /// Start the agent manager event loop
    pub async fn start(&mut self) -> Result<()> {
        let mut command_rx = self.command_rx.take()
            .ok_or_else(|| anyhow::anyhow!("Agent manager already started"))?;
        
        info!("Starting agent manager with {} agents", self.agents.len());
        
        while let Some(command) = command_rx.recv().await {
            match command {
                AgentCommand::Execute { agent_name, input, context, response_tx } => {
                    let result = self.handle_execute(&agent_name, input, context).await;
                    let _ = response_tx.send(result);
                }
                AgentCommand::HealthCheck { agent_name, response_tx } => {
                    let result = self.handle_health_check(&agent_name).await;
                    let _ = response_tx.send(result);
                }
                AgentCommand::Shutdown => {
                    info!("Agent manager shutting down");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle agent execution
    async fn handle_execute(
        &self,
        agent_name: &str,
        input: AgentInput,
        context: AgentContext,
    ) -> AgentResult<AgentOutput> {
        let agent = self.agents.get(agent_name)
            .ok_or_else(|| AgentError::ResourceUnavailable(format!("Agent '{}' not found", agent_name)))?;
        
        // Validate permissions
        self.validate_permissions(&context, agent.as_ref()).await?;
        
        // Validate input
        agent.validate_input(&input).await?;
        
        // Execute with timeout and resource monitoring
        let start_time = std::time::Instant::now();
        
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(context.limits.max_execution_time_secs.unwrap_or(300)),
            agent.execute(&context)
        ).await;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        match result {
            Ok(Ok(mut output)) => {
                output.metrics.duration_ms = duration_ms;
                info!("Agent '{}' executed successfully in {}ms", agent_name, duration_ms);
                Ok(output)
            }
            Ok(Err(e)) => {
                error!("Agent '{}' execution failed: {}", agent_name, e);
                Err(e)
            }
            Err(_) => {
                error!("Agent '{}' execution timed out", agent_name);
                Err(AgentError::Timeout(format!("Agent '{}' execution timed out", agent_name)))
            }
        }
    }
    
    /// Handle health check
    async fn handle_health_check(&self, agent_name: &str) -> AgentResult<HealthStatus> {
        let agent = self.agents.get(agent_name)
            .ok_or_else(|| AgentError::ResourceUnavailable(format!("Agent '{}' not found", agent_name)))?;
        
        agent.health_check().await
    }
    
    /// Validate agent permissions
    async fn validate_permissions(&self, context: &AgentContext, agent: &dyn Agent) -> AgentResult<()> {
        let required = agent.required_permissions();
        let granted = &context.permissions;
        
        if required.can_read_files && !granted.can_read_files {
            return Err(AgentError::PermissionDenied("File read permission required".to_string()));
        }
        
        if required.can_write_files && !granted.can_write_files {
            return Err(AgentError::PermissionDenied("File write permission required".to_string()));
        }
        
        if required.can_execute_commands && !granted.can_execute_commands {
            return Err(AgentError::PermissionDenied("Command execution permission required".to_string()));
        }
        
        if required.can_access_network && !granted.can_access_network {
            return Err(AgentError::PermissionDenied("Network access permission required".to_string()));
        }
        
        if required.can_access_web3 && !granted.can_access_web3 {
            return Err(AgentError::PermissionDenied("Web3 access permission required".to_string()));
        }
        
        Ok(())
    }
    
    /// List registered agents
    pub fn list_agents(&self) -> Vec<AgentInfo> {
        self.agents.iter().map(|(name, agent)| {
            AgentInfo {
                name: name.clone(),
                version: agent.version().to_string(),
                description: agent.description().to_string(),
                permissions: agent.required_permissions(),
                limits: agent.resource_limits(),
            }
        }).collect()
    }
    
    /// Shutdown the agent manager
    pub async fn shutdown(&self) -> Result<()> {
        self.command_tx.send(AgentCommand::Shutdown)
            .map_err(|e| anyhow::anyhow!("Shutdown command failed: {}", e))?;
        Ok(())
    }
}

/// Agent information
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub permissions: AgentPermissions,
    pub limits: ResourceLimits,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;
    
    struct TestAgent {
        name: String,
    }
    
    impl TestAgent {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }
    
    #[async_trait]
    impl Agent for TestAgent {
        async fn execute(&self, _context: &AgentContext) -> AgentResult<AgentOutput> {
            Ok(AgentOutput {
                data: [("result".to_string(), serde_json::Value::String("success".to_string()))]
                    .iter().cloned().collect(),
                metadata: HashMap::new(),
                success: true,
                message: "Test agent executed successfully".to_string(),
                metrics: ExecutionMetrics::default(),
            })
        }
        
        fn name(&self) -> &str {
            &self.name
        }
        
        fn description(&self) -> &str {
            "Test agent for unit testing"
        }
    }
    
    #[tokio::test]
    async fn test_agent_manager() {
        let mut manager = AgentManager::new(None);
        let agent = Box::new(TestAgent::new("test-agent"));
        
        manager.register_agent(agent).unwrap();
        
        let agents = manager.list_agents();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "test-agent");
    }
    
    #[tokio::test]
    async fn test_agent_execution() {
        let test_agent = TestAgent::new("test");
        let context = AgentContext {
            instance_id: "test-instance".to_string(),
            user_id: None,
            env: HashMap::new(),
            working_dir: PathBuf::from("/tmp"),
            security_manager: None,
            permissions: AgentPermissions::default(),
            limits: ResourceLimits::default(),
        };
        
        let result = test_agent.execute(&context).await;
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.data.get("result").unwrap(), "success");
    }
}
