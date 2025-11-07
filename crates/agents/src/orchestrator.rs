//! Agent Orchestration System for NEXUS
//!
//! Leverages Rust 2024 async closures for seamless multi-agent coordination

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Represents an autonomous agent in the NEXUS system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub capabilities: Vec<String>,
    pub memory: AgentMemory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    CodeAnalyzer,
    DefiTrader,
    ContractAuditor,
    DataCollector,
    TaskExecutor,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Idle,
    Running,
    Waiting,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentMemory {
    /// Short-term memory for current task
    pub working_memory: HashMap<String, serde_json::Value>,
    /// Long-term memory persisted across sessions
    pub knowledge_base: HashMap<String, serde_json::Value>,
    /// Conversation history
    pub conversation: Vec<String>,
}

/// Task to be executed by an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: Uuid,
    pub description: String,
    pub priority: u8,
    pub dependencies: Vec<Uuid>,
    pub data: serde_json::Value,
}

/// Result from agent task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub task_id: Uuid,
    pub agent_id: Uuid,
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// Agent orchestration engine using Rust 2024 async patterns
pub struct AgentOrchestrator {
    agents: Arc<RwLock<HashMap<Uuid, Agent>>>,
    task_queue: Arc<RwLock<Vec<AgentTask>>>,
    result_sender: mpsc::UnboundedSender<AgentResult>,
    result_receiver: Arc<RwLock<mpsc::UnboundedReceiver<AgentResult>>>,
}

impl AgentOrchestrator {
    /// Create a new agent orchestrator
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(Vec::new())),
            result_sender: tx,
            result_receiver: Arc::new(RwLock::new(rx)),
        }
    }

    /// Register a new agent
    pub async fn register_agent(&self, agent: Agent) -> Result<Uuid> {
        let id = agent.id;
        self.agents.write().await.insert(id, agent);
        Ok(id)
    }

    /// Submit a task to the orchestrator
    pub async fn submit_task(&self, task: AgentTask) -> Result<Uuid> {
        let id = task.id;
        self.task_queue.write().await.push(task);
        Ok(id)
    }

    /// Execute agent with async callback (Rust 2024 feature!)
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// orchestrator.execute_with_callback(
    ///     agent_id,
    ///     task,
    ///     async |result| {
    ///         println!("Task completed: {:?}", result);
    ///         notify_user(&result).await?;
    ///         Ok(())
    ///     }
    /// ).await?;
    /// ```
    pub async fn execute_with_callback<F, Fut>(
        &self,
        agent_id: Uuid,
        task: AgentTask,
        callback: F,
    ) -> Result<()>
    where
        F: FnOnce(AgentResult) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let result = self.execute_task(agent_id, task).await?;
        callback(result).await
    }

    /// Execute task on specific agent
    async fn execute_task(&self, agent_id: Uuid, task: AgentTask) -> Result<AgentResult> {
        let start = std::time::Instant::now();
        
        // Update agent status
        {
            let mut agents = self.agents.write().await;
            if let Some(agent) = agents.get_mut(&agent_id) {
                agent.status = AgentStatus::Running;
            }
        }

        // Simulate task execution (in real implementation, call actual agent logic)
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let result = AgentResult {
            task_id: task.id,
            agent_id,
            success: true,
            output: serde_json::json!({"message": "Task completed"}),
            error: None,
            execution_time_ms: start.elapsed().as_millis() as u64,
        };

        // Update agent status
        {
            let mut agents = self.agents.write().await;
            if let Some(agent) = agents.get_mut(&agent_id) {
                agent.status = AgentStatus::Completed;
            }
        }

        self.result_sender.send(result.clone()).ok();
        Ok(result)
    }

    /// Run multiple agents in parallel with async closures (Rust 2024)
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// orchestrator.parallel_execute(
    ///     vec![task1, task2, task3],
    ///     async |result| {
    ///         process_result(result).await
    ///     }
    /// ).await?;
    /// ```
    pub async fn parallel_execute<F, Fut>(
        &self,
        tasks: Vec<AgentTask>,
        handler: F,
    ) -> Result<Vec<Result<()>>>
    where
        F: Fn(AgentResult) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        use futures::future::join_all;

        let agents = self.agents.read().await;
        let available_agents: Vec<Uuid> = agents
            .iter()
            .filter(|(_, a)| a.status == AgentStatus::Idle)
            .map(|(id, _)| *id)
            .collect();
        drop(agents);

        let execute_tasks: Vec<_> = tasks
            .into_iter()
            .zip(available_agents.iter().cycle())
            .map(|(task, agent_id)| {
                let agent_id = *agent_id;
                async move {
                    let result = self.execute_task(agent_id, task).await?;
                    handler(result).await
                }
            })
            .collect();

        Ok(join_all(execute_tasks).await)
    }

    /// Chain multiple agent operations (Rust 2024 async closure composition)
    pub async fn chain_agents<F1, F2, Fut1, Fut2>(
        &self,
        agent1_id: Uuid,
        agent2_id: Uuid,
        task: AgentTask,
        transform: F1,
        finalize: F2,
    ) -> Result<serde_json::Value>
    where
        F1: FnOnce(AgentResult) -> Fut1,
        F2: FnOnce(serde_json::Value) -> Fut2,
        Fut1: std::future::Future<Output = Result<AgentTask>>,
        Fut2: std::future::Future<Output = Result<serde_json::Value>>,
    {
        // Execute first agent
        let result1 = self.execute_task(agent1_id, task).await?;
        
        // Transform result into new task
        let task2 = transform(result1).await?;
        
        // Execute second agent
        let result2 = self.execute_task(agent2_id, task2).await?;
        
        // Finalize and return
        finalize(result2.output).await
    }

    /// Get all agent statuses
    pub async fn get_agent_statuses(&self) -> HashMap<Uuid, AgentStatus> {
        self.agents
            .read()
            .await
            .iter()
            .map(|(id, agent)| (*id, agent.status.clone()))
            .collect()
    }

    /// Wait for all agents to complete
    pub async fn wait_for_completion(&self, timeout_secs: u64) -> Result<()> {
        let timeout = tokio::time::Duration::from_secs(timeout_secs);
        let start = std::time::Instant::now();

        loop {
            let agents = self.agents.read().await;
            let all_done = agents.values().all(|a| {
                matches!(
                    a.status,
                    AgentStatus::Idle | AgentStatus::Completed | AgentStatus::Failed(_)
                )
            });

            if all_done {
                return Ok(());
            }

            if start.elapsed() > timeout {
                anyhow::bail!("Timeout waiting for agents to complete");
            }

            drop(agents);
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// Get results stream (returns all pending results)
    pub async fn get_results(&self) -> Vec<AgentResult> {
        let mut results = Vec::new();
        let mut rx = self.result_receiver.write().await;
        
        while let Ok(result) = rx.try_recv() {
            results.push(result);
        }
        
        results
    }
}

impl Default for AgentOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for creating agents
pub struct AgentBuilder {
    name: String,
    agent_type: AgentType,
    capabilities: Vec<String>,
}

impl AgentBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            agent_type: AgentType::TaskExecutor,
            capabilities: Vec::new(),
        }
    }

    pub fn agent_type(mut self, agent_type: AgentType) -> Self {
        self.agent_type = agent_type;
        self
    }

    pub fn capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    pub fn build(self) -> Agent {
        Agent {
            id: Uuid::new_v4(),
            name: self.name,
            agent_type: self.agent_type,
            status: AgentStatus::Idle,
            capabilities: self.capabilities,
            memory: AgentMemory::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_orchestration() {
        let orchestrator = AgentOrchestrator::new();

        // Create and register agents
        let agent = AgentBuilder::new("TestAgent")
            .agent_type(AgentType::TaskExecutor)
            .capability("analyze")
            .build();

        let agent_id = orchestrator.register_agent(agent).await.unwrap();

        // Create task
        let task = AgentTask {
            id: Uuid::new_v4(),
            description: "Test task".to_string(),
            priority: 1,
            dependencies: vec![],
            data: serde_json::json!({"test": true}),
        };

        // Execute with callback (Rust 2024 async closure)
        orchestrator
            .execute_with_callback(agent_id, task, async |result| {
                assert!(result.success);
                Ok(())
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        let orchestrator = AgentOrchestrator::new();

        // Register multiple agents
        for i in 0..3 {
            let agent = AgentBuilder::new(format!("Agent{}", i)).build();
            orchestrator.register_agent(agent).await.unwrap();
        }

        // Create tasks
        let tasks: Vec<_> = (0..3)
            .map(|i| AgentTask {
                id: Uuid::new_v4(),
                description: format!("Task {}", i),
                priority: 1,
                dependencies: vec![],
                data: serde_json::json!({"index": i}),
            })
            .collect();

        // Execute in parallel with async closure handler
        let results = orchestrator
            .parallel_execute(tasks, async |result| {
                assert!(result.success);
                Ok(())
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 3);
    }
}
