//! Natural Language Command Interpreter for NEXUS
//!
//! This module provides AI-powered natural language understanding for CLI commands,
//! leveraging Rust 2024 async closures and modern LLM integration.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents a parsed command from natural language input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCommand {
    /// The action to perform (e.g., "deploy", "analyze", "bridge")
    pub action: String,
    /// Target resource (e.g., "contract", "wallet", "transaction")
    pub target: String,
    /// Chain or network identifier
    pub chain: Option<String>,
    /// Additional parameters extracted from NL
    pub parameters: serde_json::Value,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Original user input
    pub original_input: String,
}

/// AI-powered natural language command interpreter
#[derive(Clone)]
pub struct NLPCommandInterpreter {
    /// LLM client (OpenAI, Anthropic, local, etc.)
    llm_client: Arc<RwLock<Box<dyn LLMClient + Send + Sync>>>,
    /// Command history for context-aware parsing
    command_history: Arc<RwLock<Vec<ParsedCommand>>>,
}

/// Trait for LLM client implementations
#[async_trait::async_trait]
pub trait LLMClient {
    /// Parse natural language into structured command
    async fn parse_command(&self, input: &str, context: &[ParsedCommand]) -> Result<ParsedCommand>;
    
    /// Generate command suggestions based on input
    async fn suggest_commands(&self, partial_input: &str) -> Result<Vec<String>>;
    
    /// Explain what a command will do in natural language
    async fn explain_command(&self, command: &ParsedCommand) -> Result<String>;
}

impl NLPCommandInterpreter {
    /// Create a new NLP command interpreter
    pub fn new(llm_client: Box<dyn LLMClient + Send + Sync>) -> Self {
        Self {
            llm_client: Arc::new(RwLock::new(llm_client)),
            command_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Parse natural language input into a structured command
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// let interpreter = NLPCommandInterpreter::new(client);
    /// let cmd = interpreter.parse("deploy an NFT contract on Base with royalties").await?;
    /// ```
    pub async fn parse(&self, input: &str) -> Result<ParsedCommand> {
        let client = self.llm_client.read().await;
        let history = self.command_history.read().await;
        
        let command = client
            .parse_command(input, &history)
            .await
            .context("Failed to parse natural language command")?;
        
        // Store in history for context
        drop(history);
        self.command_history.write().await.push(command.clone());
        
        Ok(command)
    }

    /// Get command suggestions as user types (using Rust 2024 async closures)
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// let suggestions = interpreter.suggest("deploy").await?;
    /// ```
    pub async fn suggest(&self, partial_input: &str) -> Result<Vec<String>> {
        let client = self.llm_client.read().await;
        client.suggest_commands(partial_input).await
    }

    /// Explain what a parsed command will do before execution
    pub async fn explain(&self, command: &ParsedCommand) -> Result<String> {
        let client = self.llm_client.read().await;
        client.explain_command(command).await
    }

    /// Process command with async callback (Rust 2024 feature)
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// interpreter.process_with_callback(
    ///     "analyze gas usage",
    ///     async |cmd| {
    ///         println!("Executing: {:?}", cmd);
    ///         execute_command(cmd).await
    ///     }
    /// ).await?;
    /// ```
    pub async fn process_with_callback<F, Fut>(
        &self,
        input: &str,
        callback: F,
    ) -> Result<()>
    where
        F: FnOnce(ParsedCommand) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let command = self.parse(input).await?;
        callback(command).await
    }

    /// Batch process multiple commands in parallel (leveraging async closures)
    pub async fn batch_process<F, Fut>(
        &self,
        inputs: Vec<&str>,
        handler: F,
    ) -> Result<Vec<Result<()>>>
    where
        F: Fn(ParsedCommand) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        use futures::future::join_all;

        let parse_tasks: Vec<_> = inputs
            .into_iter()
            .map(|input| async move { self.parse(input).await })
            .collect();

        let commands: Vec<_> = join_all(parse_tasks).await;

        let execute_tasks: Vec<_> = commands
            .into_iter()
            .filter_map(|cmd_result| cmd_result.ok())
            .map(|cmd| handler(cmd))
            .collect();

        Ok(join_all(execute_tasks).await)
    }

    /// Clear command history
    pub async fn clear_history(&self) {
        self.command_history.write().await.clear();
    }

    /// Get recent command history
    pub async fn get_history(&self, limit: usize) -> Vec<ParsedCommand> {
        let history = self.command_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }
}

/// OpenAI GPT-4 implementation of LLM client
pub struct OpenAIClient {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "gpt-4o".to_string(),
            client: reqwest::Client::new(),
        }
    }

    fn build_system_prompt() -> &'static str {
        r#"You are a NEXUS CLI command interpreter. Parse natural language into JSON commands.
        
        Available actions: deploy, analyze, bridge, swap, monitor, generate, audit
        Available targets: contract, wallet, transaction, token, nft, defi
        Supported chains: ethereum, solana, polygon, arbitrum, base, optimism, near, aptos, sui
        
        Return JSON: {"action": "...", "target": "...", "chain": "...", "parameters": {...}, "confidence": 0.0-1.0}
        "#
    }
}

#[async_trait::async_trait]
impl LLMClient for OpenAIClient {
    async fn parse_command(&self, input: &str, context: &[ParsedCommand]) -> Result<ParsedCommand> {
        let context_str = if !context.is_empty() {
            format!(
                "Previous commands: {}",
                context
                    .iter()
                    .rev()
                    .take(3)
                    .map(|c| &c.original_input)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            String::new()
        };

        let prompt = format!(
            "{}\n\nContext: {}\n\nUser input: {}\n\nParse into JSON command:",
            Self::build_system_prompt(),
            context_str,
            input
        );

        // Make API call (simplified - in production use proper OpenAI SDK)
        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "messages": [
                    {"role": "system", "content": Self::build_system_prompt()},
                    {"role": "user", "content": prompt}
                ],
                "temperature": 0.3,
                "response_format": {"type": "json_object"}
            }))
            .send()
            .await?
            .error_for_status()?;

        let json: serde_json::Value = response.json().await?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .context("Missing content in response")?;

        let mut parsed: ParsedCommand = serde_json::from_str(content)
            .context("Failed to parse LLM response as command")?;
        
        parsed.original_input = input.to_string();
        Ok(parsed)
    }

    async fn suggest_commands(&self, partial_input: &str) -> Result<Vec<String>> {
        let prompt = format!(
            "Suggest 5 complete NEXUS CLI commands starting with: '{}'",
            partial_input
        );

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "messages": [{"role": "user", "content": prompt}],
                "temperature": 0.7,
                "max_tokens": 200
            }))
            .send()
            .await?
            .error_for_status()?;

        let json: serde_json::Value = response.json().await?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("");

        Ok(content.lines().map(|s| s.trim().to_string()).collect())
    }

    async fn explain_command(&self, command: &ParsedCommand) -> Result<String> {
        let prompt = format!(
            "Explain what this command will do in one sentence: {:?}",
            command
        );

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "messages": [{"role": "user", "content": prompt}],
                "temperature": 0.3,
                "max_tokens": 100
            }))
            .send()
            .await?
            .error_for_status()?;

        let json: serde_json::Value = response.json().await?;
        Ok(json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("Unable to explain command")
            .to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nlp_command_parsing() {
        // Mock implementation for testing
        struct MockClient;

        #[async_trait::async_trait]
        impl LLMClient for MockClient {
            async fn parse_command(
                &self,
                input: &str,
                _context: &[ParsedCommand],
            ) -> Result<ParsedCommand> {
                Ok(ParsedCommand {
                    action: "deploy".to_string(),
                    target: "contract".to_string(),
                    chain: Some("ethereum".to_string()),
                    parameters: serde_json::json!({"type": "ERC721"}),
                    confidence: 0.95,
                    original_input: input.to_string(),
                })
            }

            async fn suggest_commands(&self, _partial: &str) -> Result<Vec<String>> {
                Ok(vec!["deploy contract".to_string()])
            }

            async fn explain_command(&self, _cmd: &ParsedCommand) -> Result<String> {
                Ok("This will deploy a contract".to_string())
            }
        }

        let interpreter = NLPCommandInterpreter::new(Box::new(MockClient));
        let result = interpreter.parse("deploy nft contract").await.unwrap();
        assert_eq!(result.action, "deploy");
        assert_eq!(result.confidence, 0.95);
    }
}
