//! Configuration management for NEXUS
//!
//! This module provides configuration loading, validation, and management
//! for the NEXUS system with security-first design.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

use crate::security::SecurityConfig;

/// Main configuration structure for NEXUS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Security configuration
    pub security: SecurityConfig,
    /// Agent configuration
    pub agent: AgentConfig,
    /// Plugin configuration
    pub plugin: PluginConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Web3 configuration
    #[cfg(feature = "web3")]
    pub web3: Web3Config,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            security: SecurityConfig::default(),
            agent: AgentConfig::default(),
            plugin: PluginConfig::default(),
            logging: LoggingConfig::default(),
            #[cfg(feature = "web3")]
            web3: Web3Config::default(),
        }
    }
}

/// Agent system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Maximum number of concurrent agents
    pub max_concurrent_agents: usize,
    /// Default agent timeout in seconds
    pub default_timeout_secs: u64,
    /// Agent data directory
    pub data_dir: PathBuf,
    /// Enable agent sandboxing
    pub enable_sandboxing: bool,
    /// Default resource limits
    pub default_resource_limits: AgentResourceLimits,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_concurrent_agents: 10,
            default_timeout_secs: 300,
            data_dir: PathBuf::from("./data/agents"),
            enable_sandboxing: true,
            default_resource_limits: AgentResourceLimits::default(),
        }
    }
}

/// Agent resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: u64,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: f32,
    /// Maximum file operations per second
    pub max_file_ops_per_sec: u32,
    /// Maximum network requests per minute
    pub max_network_requests_per_min: u32,
}

impl Default for AgentResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 100,
            max_cpu_percent: 50.0,
            max_file_ops_per_sec: 100,
            max_network_requests_per_min: 1000,
        }
    }
}

/// Plugin system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin directories to scan
    pub plugin_dirs: Vec<PathBuf>,
    /// Enable plugin hot reloading
    pub enable_hot_reload: bool,
    /// Plugin security policy
    pub security_policy: PluginSecurityPolicy,
    /// Maximum plugin load time in seconds
    pub max_load_time_secs: u64,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            plugin_dirs: vec![PathBuf::from("./plugins")],
            enable_hot_reload: false, // Disabled by default for security
            security_policy: PluginSecurityPolicy::default(),
            max_load_time_secs: 30,
        }
    }
}

/// Plugin security policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSecurityPolicy {
    /// Require signed plugins
    pub require_signed: bool,
    /// Trusted plugin publishers
    pub trusted_publishers: Vec<String>,
    /// Allow unsigned plugins from local development
    pub allow_local_unsigned: bool,
    /// Plugin isolation level
    pub isolation_level: PluginIsolationLevel,
}

impl Default for PluginSecurityPolicy {
    fn default() -> Self {
        Self {
            require_signed: true,
            trusted_publishers: vec!["nexus-official".to_string()],
            allow_local_unsigned: false, // Secure by default
            isolation_level: PluginIsolationLevel::Strict,
        }
    }
}

/// Plugin isolation levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginIsolationLevel {
    /// No isolation (dangerous)
    None,
    /// Basic isolation
    Basic,
    /// Strict isolation (recommended)
    Strict,
    /// Maximum isolation (may impact performance)
    Maximum,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Log format (json, pretty, compact)
    pub format: String,
    /// Log to file
    pub log_to_file: bool,
    /// Log file path
    pub log_file_path: Option<PathBuf>,
    /// Enable structured logging
    pub structured: bool,
    /// Enable security event logging
    pub security_events: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            log_to_file: false,
            log_file_path: None,
            structured: true,
            security_events: true,
        }
    }
}

/// Web3 configuration
#[cfg(feature = "web3")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3Config {
    /// Default network to connect to
    pub default_network: String,
    /// RPC endpoints
    pub rpc_endpoints: std::collections::HashMap<String, String>,
    /// Enable transaction simulation
    pub enable_simulation: bool,
    /// Gas limit multiplier for safety
    pub gas_limit_multiplier: f64,
    /// Private key storage method
    pub key_storage: KeyStorageConfig,
}

#[cfg(feature = "web3")]
impl Default for Web3Config {
    fn default() -> Self {
        let mut rpc_endpoints = std::collections::HashMap::new();
        rpc_endpoints.insert("ethereum".to_string(), "https://eth.llamarpc.com".to_string());
        rpc_endpoints.insert("polygon".to_string(), "https://polygon.llamarpc.com".to_string());
        rpc_endpoints.insert("base".to_string(), "https://base.llamarpc.com".to_string());
        
        Self {
            default_network: "ethereum".to_string(),
            rpc_endpoints,
            enable_simulation: true,
            gas_limit_multiplier: 1.2,
            key_storage: KeyStorageConfig::default(),
        }
    }
}

/// Key storage configuration
#[cfg(feature = "web3")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStorageConfig {
    /// Storage type
    pub storage_type: KeyStorageType,
    /// Encryption enabled
    pub encrypted: bool,
    /// Key derivation function
    pub kdf: String,
}

#[cfg(feature = "web3")]
impl Default for KeyStorageConfig {
    fn default() -> Self {
        Self {
            storage_type: KeyStorageType::Keychain,
            encrypted: true,
            kdf: "argon2".to_string(),
        }
    }
}

/// Key storage types
#[cfg(feature = "web3")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyStorageType {
    /// OS keychain/keyring
    Keychain,
    /// Encrypted file
    EncryptedFile,
    /// Hardware wallet
    Hardware,
    /// Environment variables (not recommended for production)
    Environment,
}

/// Configuration loader
pub struct ConfigLoader {
    search_paths: Vec<PathBuf>,
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Self {
        Self {
            search_paths: vec![
                PathBuf::from("./nexus.toml"),
                PathBuf::from("./config/nexus.toml"),
                PathBuf::from("~/.config/nexus/config.toml"),
                PathBuf::from("/etc/nexus/config.toml"),
            ],
        }
    }
    
    /// Add a search path for configuration files
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }
    
    /// Load configuration from file or use defaults
    pub fn load(&self) -> Result<Config> {
        // Try to find and load configuration file
        for path in &self.search_paths {
            if path.exists() {
                info!("Loading configuration from: {:?}", path);
                return self.load_from_file(path);
            }
        }
        
        warn!("No configuration file found, using defaults");
        Ok(Config::default())
    }
    
    /// Load configuration from a specific file
    pub fn load_from_file(&self, path: &PathBuf) -> Result<Config> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;
        
        self.validate_config(&config)?;
        
        Ok(config)
    }
    
    /// Load configuration from string
    pub fn load_from_string(&self, content: &str) -> Result<Config> {
        let config: Config = toml::from_str(content)
            .context("Failed to parse configuration string")?;
        
        self.validate_config(&config)?;
        
        Ok(config)
    }
    
    /// Validate configuration
    fn validate_config(&self, config: &Config) -> Result<()> {
        // Validate security configuration
        if config.security.rate_limit.max_requests == 0 {
            return Err(anyhow::anyhow!("Rate limit max_requests must be greater than 0"));
        }
        
        // Validate agent configuration
        if config.agent.max_concurrent_agents == 0 {
            return Err(anyhow::anyhow!("max_concurrent_agents must be greater than 0"));
        }
        
        if config.agent.default_timeout_secs == 0 {
            return Err(anyhow::anyhow!("default_timeout_secs must be greater than 0"));
        }
        
        // Validate plugin configuration
        if config.plugin.plugin_dirs.is_empty() {
            warn!("No plugin directories configured");
        }
        
        // Validate logging configuration
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&config.logging.level.as_str()) {
            return Err(anyhow::anyhow!(
                "Invalid log level '{}', must be one of: {}", 
                config.logging.level,
                valid_levels.join(", ")
            ));
        }
        
        Ok(())
    }
    
    /// Save configuration to file
    pub fn save_to_file(&self, config: &Config, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(config)
            .context("Failed to serialize configuration")?;
        
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;
        
        info!("Configuration saved to: {:?}", path);
        Ok(())
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.security.encryption_enabled);
        assert_eq!(config.agent.max_concurrent_agents, 10);
        assert_eq!(config.logging.level, "info");
    }
    
    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.security.encryption_enabled, deserialized.security.encryption_enabled);
        assert_eq!(config.agent.max_concurrent_agents, deserialized.agent.max_concurrent_agents);
    }
    
    #[test]
    fn test_config_loader() {
        let loader = ConfigLoader::new();
        
        // Test loading default config when no file exists
        let config = loader.load().unwrap();
        assert!(config.security.encryption_enabled);
    }
    
    #[test]
    fn test_config_file_loading() {
        let config_content = r#"
[security]
encryption_enabled = false
min_password_length = 8

[agent]
max_concurrent_agents = 5
default_timeout_secs = 60

[logging]
level = "debug"
format = "json"
        "#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", config_content).unwrap();
        
        let loader = ConfigLoader::new();
        let config = loader.load_from_file(&temp_file.path().to_path_buf()).unwrap();
        
        assert!(!config.security.encryption_enabled);
        assert_eq!(config.security.min_password_length, 8);
        assert_eq!(config.agent.max_concurrent_agents, 5);
        assert_eq!(config.agent.default_timeout_secs, 60);
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.logging.format, "json");
    }
    
    #[test]
    fn test_config_validation() {
        let loader = ConfigLoader::new();
        
        // Test invalid log level
        let invalid_config = r#"
[logging]
level = "invalid"
        "#;
        
        let result = loader.load_from_string(invalid_config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid log level"));
    }
}
