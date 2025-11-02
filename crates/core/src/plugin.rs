//! Plugin system for NEXUS
//!
//! This module provides secure plugin loading and management capabilities
//! with sandboxing and permission controls.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{info, warn, error};

use crate::agent::{Agent, AgentContext, AgentResult, AgentOutput};
use crate::config::PluginConfig;
use crate::security::SecurityManager;

/// Plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin
    fn initialize(&mut self, config: &PluginConfig) -> Result<()>;
    
    /// Get agents provided by this plugin
    fn agents(&self) -> Vec<Box<dyn Agent>>;
    
    /// Shutdown the plugin
    fn shutdown(&mut self) -> Result<()>;
    
    /// Health check for the plugin
    fn health_check(&self) -> Result<PluginHealth>;
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Required NEXUS version
    pub required_nexus_version: String,
    /// Plugin dependencies
    pub dependencies: Vec<String>,
    /// Digital signature (if signed)
    pub signature: Option<String>,
    /// Plugin permissions
    pub permissions: PluginPermissions,
}

/// Plugin permissions
#[derive(Debug, Clone)]
pub struct PluginPermissions {
    /// Can access filesystem
    pub filesystem_access: bool,
    /// Can access network
    pub network_access: bool,
    /// Can execute system commands
    pub system_commands: bool,
    /// Can access Web3 functions
    pub web3_access: bool,
    /// Can access other plugins
    pub plugin_access: bool,
    /// Can modify NEXUS configuration
    pub config_access: bool,
}

impl Default for PluginPermissions {
    fn default() -> Self {
        Self {
            filesystem_access: false,
            network_access: false,
            system_commands: false,
            web3_access: false,
            plugin_access: false,
            config_access: false,
        }
    }
}

/// Plugin health status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginHealth {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Plugin manager for loading and managing plugins
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    config: PluginConfig,
    security_manager: Option<Arc<SecurityManager>>,
    plugin_agents: HashMap<String, Vec<String>>, // plugin_name -> agent_names
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(config: PluginConfig, security_manager: Option<Arc<SecurityManager>>) -> Self {
        Self {
            plugins: HashMap::new(),
            config,
            security_manager,
            plugin_agents: HashMap::new(),
        }
    }
    
    /// Load plugins from configured directories
    pub async fn load_plugins(&mut self) -> Result<()> {
        info!("Loading plugins from {} directories", self.config.plugin_dirs.len());
        
        for plugin_dir in &self.config.plugin_dirs {
            if plugin_dir.exists() {
                self.load_plugins_from_directory(plugin_dir).await
                    .with_context(|| format!("Failed to load plugins from {:?}", plugin_dir))?;
            } else {
                warn!("Plugin directory does not exist: {:?}", plugin_dir);
            }
        }
        
        info!("Loaded {} plugins", self.plugins.len());
        Ok(())
    }
    
    /// Load plugins from a specific directory
    async fn load_plugins_from_directory(&mut self, dir: &Path) -> Result<()> {
        let entries = std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read plugin directory: {:?}", dir))?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                // Check for plugin libraries (e.g., .so, .dll, .dylib)
                if let Some(extension) = path.extension() {
                    if self.is_plugin_library(extension) {
                        self.load_plugin_from_file(&path).await
                            .unwrap_or_else(|e| {
                                error!("Failed to load plugin from {:?}: {}", path, e);
                            });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if file extension indicates a plugin library
    fn is_plugin_library(&self, extension: &std::ffi::OsStr) -> bool {
        matches!(
            extension.to_str(),
            Some("so") | Some("dll") | Some("dylib")
        )
    }
    
    /// Load a plugin from a file
    async fn load_plugin_from_file(&mut self, path: &Path) -> Result<()> {
        info!("Loading plugin from: {:?}", path);
        
        // Security check: verify plugin signature if required
        if self.config.security_policy.require_signed {
            self.verify_plugin_signature(path).await
                .context("Plugin signature verification failed")?;
        }
        
        // Load the plugin using the dynamic library loader
        let plugin = self.load_dynamic_plugin(path).await
            .context("Failed to load dynamic plugin")?;
        
        let metadata = plugin.metadata().clone();
        
        // Validate plugin permissions
        self.validate_plugin_permissions(&metadata)
            .context("Plugin permission validation failed")?;
        
        // Initialize the plugin
        let mut plugin = plugin;
        plugin.initialize(&self.config)
            .context("Plugin initialization failed")?;
        
        // Register plugin agents
        let agents = plugin.agents();
        let agent_names: Vec<String> = agents.iter().map(|a| a.name().to_string()).collect();
        
        info!("Plugin '{}' provides {} agents: {:?}", 
            metadata.name, agent_names.len(), agent_names);
        
        self.plugin_agents.insert(metadata.name.clone(), agent_names);
        self.plugins.insert(metadata.name.clone(), plugin);
        
        Ok(())
    }
    
    /// Verify plugin signature
    async fn verify_plugin_signature(&self, _path: &Path) -> Result<()> {
        // In a real implementation, this would:
        // 1. Check the plugin's digital signature
        // 2. Verify against trusted certificates
        // 3. Ensure the plugin hasn't been tampered with
        
        // For now, we'll just log that signature verification would happen here
        info!("Plugin signature verification (not implemented yet)");
        Ok(())
    }
    
    /// Load dynamic plugin (placeholder)
    async fn load_dynamic_plugin(&self, _path: &Path) -> Result<Box<dyn Plugin>> {
        // In a real implementation, this would use libloading to:
        // 1. Load the dynamic library
        // 2. Get the plugin factory function
        // 3. Create the plugin instance
        
        // For now, return a mock plugin
        Ok(Box::new(MockPlugin::new()))
    }
    
    /// Validate plugin permissions
    fn validate_plugin_permissions(&self, metadata: &PluginMetadata) -> Result<()> {
        let permissions = &metadata.permissions;
        
        // Check if plugin requires permissions that are not allowed
        match self.config.security_policy.isolation_level {
            crate::config::PluginIsolationLevel::Maximum => {
                if permissions.system_commands || permissions.config_access {
                    return Err(anyhow::anyhow!(
                        "Plugin '{}' requires dangerous permissions not allowed in maximum isolation",
                        metadata.name
                    ));
                }
            }
            crate::config::PluginIsolationLevel::Strict => {
                if permissions.system_commands {
                    warn!("Plugin '{}' requires system command access", metadata.name);
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Get all loaded plugins
    pub fn list_plugins(&self) -> Vec<&PluginMetadata> {
        self.plugins.values().map(|p| p.metadata()).collect()
    }
    
    /// Get plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&Box<dyn Plugin>> {
        self.plugins.get(name)
    }
    
    /// Get agents from all plugins
    pub fn get_all_agents(&self) -> Vec<Box<dyn Agent>> {
        let mut agents = Vec::new();
        
        for plugin in self.plugins.values() {
            agents.extend(plugin.agents());
        }
        
        agents
    }
    
    /// Get agents from a specific plugin
    pub fn get_plugin_agents(&self, plugin_name: &str) -> Option<Vec<Box<dyn Agent>>> {
        self.plugins.get(plugin_name).map(|plugin| plugin.agents())
    }
    
    /// Unload a plugin
    pub async fn unload_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(mut plugin) = self.plugins.remove(name) {
            plugin.shutdown()
                .with_context(|| format!("Failed to shutdown plugin '{}'", name))?;
            
            self.plugin_agents.remove(name);
            info!("Unloaded plugin: {}", name);
        }
        
        Ok(())
    }
    
    /// Reload a plugin
    pub async fn reload_plugin(&mut self, name: &str, path: &Path) -> Result<()> {
        self.unload_plugin(name).await?;
        self.load_plugin_from_file(path).await?;
        Ok(())
    }
    
    /// Check health of all plugins
    pub fn check_plugin_health(&self) -> HashMap<String, PluginHealth> {
        let mut health_status = HashMap::new();
        
        for (name, plugin) in &self.plugins {
            let health = plugin.health_check().unwrap_or(PluginHealth::Unhealthy(
                "Health check failed".to_string()
            ));
            health_status.insert(name.clone(), health);
        }
        
        health_status
    }
    
    /// Shutdown all plugins
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down {} plugins", self.plugins.len());
        
        for (name, mut plugin) in self.plugins.drain() {
            if let Err(e) = plugin.shutdown() {
                error!("Failed to shutdown plugin '{}': {}", name, e);
            }
        }
        
        self.plugin_agents.clear();
        Ok(())
    }
}

/// Mock plugin for testing
struct MockPlugin {
    metadata: PluginMetadata,
}

impl MockPlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "mock-plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Mock plugin for testing".to_string(),
                author: "NEXUS Team".to_string(),
                required_nexus_version: "0.1.0".to_string(),
                dependencies: Vec::new(),
                signature: None,
                permissions: PluginPermissions::default(),
            },
        }
    }
}

impl Plugin for MockPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn initialize(&mut self, _config: &PluginConfig) -> Result<()> {
        Ok(())
    }
    
    fn agents(&self) -> Vec<Box<dyn Agent>> {
        Vec::new() // No agents for mock plugin
    }
    
    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn health_check(&self) -> Result<PluginHealth> {
        Ok(PluginHealth::Healthy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PluginSecurityPolicy;
    use tempfile::TempDir;
    
    fn test_plugin_config() -> PluginConfig {
        PluginConfig {
            plugin_dirs: vec![PathBuf::from("./test_plugins")],
            enable_hot_reload: false,
            security_policy: PluginSecurityPolicy::default(),
            max_load_time_secs: 30,
        }
    }
    
    #[tokio::test]
    async fn test_plugin_manager_creation() {
        let config = test_plugin_config();
        let manager = PluginManager::new(config, None);
        
        assert_eq!(manager.plugins.len(), 0);
    }
    
    #[tokio::test]
    async fn test_load_plugins_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let config = PluginConfig {
            plugin_dirs: vec![temp_dir.path().to_path_buf()],
            enable_hot_reload: false,
            security_policy: PluginSecurityPolicy::default(),
            max_load_time_secs: 30,
        };
        
        let mut manager = PluginManager::new(config, None);
        let result = manager.load_plugins().await;
        
        assert!(result.is_ok());
        assert_eq!(manager.plugins.len(), 0);
    }
    
    #[test]
    fn test_plugin_permissions() {
        let permissions = PluginPermissions::default();
        
        assert!(!permissions.filesystem_access);
        assert!(!permissions.network_access);
        assert!(!permissions.system_commands);
        assert!(!permissions.web3_access);
        assert!(!permissions.plugin_access);
        assert!(!permissions.config_access);
    }
    
    #[test]
    fn test_mock_plugin() {
        let plugin = MockPlugin::new();
        let metadata = plugin.metadata();
        
        assert_eq!(metadata.name, "mock-plugin");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(plugin.agents().len(), 0);
        
        let health = plugin.health_check().unwrap();
        assert_eq!(health, PluginHealth::Healthy);
    }
}
