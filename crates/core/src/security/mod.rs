//! Security utilities and cryptographic functions for NEXUS
//!
//! This module provides essential security features including:
//! - Cryptographic operations (encryption, hashing, key derivation)
//! - Input validation and sanitization
//! - Secure configuration management
//! - Rate limiting and resource protection
//! - Audit logging for security events

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{error, warn, info};

pub mod crypto;
pub mod validation;
pub mod config;
pub mod audit;
pub mod ratelimit;

/// Security configuration for NEXUS
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable encryption for sensitive data
    pub encryption_enabled: bool,
    /// Minimum password length
    pub min_password_length: usize,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    /// Audit logging configuration
    pub audit_config: AuditConfig,
    /// Input validation settings
    pub validation: ValidationConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_enabled: true,
            min_password_length: 12,
            rate_limit: RateLimitConfig::default(),
            audit_config: AuditConfig::default(),
            validation: ValidationConfig::default(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per time window
    pub max_requests: u32,
    /// Time window for rate limiting
    pub time_window: Duration,
    /// Enable rate limiting
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            time_window: Duration::from_secs(60),
            enabled: true,
        }
    }
}

/// Audit logging configuration
#[derive(Debug, Clone)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,
    /// Log security events
    pub log_security_events: bool,
    /// Log authentication events
    pub log_auth_events: bool,
    /// Log data access events
    pub log_data_access: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_security_events: true,
            log_auth_events: true,
            log_data_access: false, // Can be verbose
        }
    }
}

/// Input validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Maximum input length
    pub max_input_length: usize,
    /// Enable XSS protection
    pub xss_protection: bool,
    /// Enable SQL injection protection
    pub sql_injection_protection: bool,
    /// Enable path traversal protection
    pub path_traversal_protection: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_input_length: 10_000,
            xss_protection: true,
            sql_injection_protection: true,
            path_traversal_protection: true,
        }
    }
}

/// Security manager for NEXUS
pub struct SecurityManager {
    config: SecurityConfig,
    rate_limiters: Arc<Mutex<HashMap<String, RateLimiter>>>,
    audit_logger: audit::AuditLogger,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: SecurityConfig) -> Result<Self> {
        let audit_logger = audit::AuditLogger::new(&config.audit_config)
            .context("Failed to initialize audit logger")?;
        
        Ok(Self {
            config,
            rate_limiters: Arc::new(Mutex::new(HashMap::new())),
            audit_logger,
        })
    }

    /// Check if an operation should be rate limited
    pub fn check_rate_limit(&self, key: &str) -> Result<bool> {
        if !self.config.rate_limit.enabled {
            return Ok(true);
        }

        let mut limiters = self.rate_limiters.lock().unwrap();
        let limiter = limiters
            .entry(key.to_string())
            .or_insert_with(|| RateLimiter::new(self.config.rate_limit.clone()));

        let allowed = limiter.check();
        
        if !allowed {
            warn!("Rate limit exceeded for key: {}", key);
            self.audit_logger.log_security_event(
                "rate_limit_exceeded",
                &format!("Key: {}", key),
            );
        }

        Ok(allowed)
    }

    /// Validate input according to security policies
    pub fn validate_input(&self, input: &str, input_type: &str) -> Result<()> {
        validation::validate_input(input, input_type, &self.config.validation)
            .with_context(|| format!("Input validation failed for type: {}", input_type))
    }

    /// Log a security event
    pub fn log_security_event(&self, event_type: &str, details: &str) {
        self.audit_logger.log_security_event(event_type, details);
    }

    /// Get security metrics
    pub fn get_metrics(&self) -> SecurityMetrics {
        let limiters = self.rate_limiters.lock().unwrap();
        let active_rate_limiters = limiters.len();
        
        SecurityMetrics {
            active_rate_limiters,
            encryption_enabled: self.config.encryption_enabled,
            audit_enabled: self.config.audit_config.enabled,
        }
    }
}

/// Security metrics
#[derive(Debug)]
pub struct SecurityMetrics {
    pub active_rate_limiters: usize,
    pub encryption_enabled: bool,
    pub audit_enabled: bool,
}

/// Simple rate limiter implementation
struct RateLimiter {
    config: RateLimitConfig,
    requests: Vec<Instant>,
}

impl RateLimiter {
    fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            requests: Vec::new(),
        }
    }

    fn check(&mut self) -> bool {
        let now = Instant::now();
        let cutoff = now - self.config.time_window;
        
        // Remove old requests
        self.requests.retain(|&time| time > cutoff);
        
        // Check if we're under the limit
        if self.requests.len() < self.config.max_requests as usize {
            self.requests.push(now);
            true
        } else {
            false
        }
    }
}

/// Initialize security subsystem
pub fn init_security(config: SecurityConfig) -> Result<SecurityManager> {
    info!("Initializing NEXUS security subsystem");
    
    // Log security configuration (without sensitive data)
    info!("Security config - Encryption: {}, Rate limiting: {}, Audit: {}", 
        config.encryption_enabled,
        config.rate_limit.enabled,
        config.audit_config.enabled
    );

    SecurityManager::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_security_manager_creation() {
        let config = SecurityConfig::default();
        let manager = SecurityManager::new(config).unwrap();
        
        let metrics = manager.get_metrics();
        assert!(metrics.encryption_enabled);
        assert!(metrics.audit_enabled);
    }

    #[test]
    fn test_rate_limiting() {
        let mut config = SecurityConfig::default();
        config.rate_limit.max_requests = 2;
        config.rate_limit.time_window = Duration::from_millis(100);
        
        let manager = SecurityManager::new(config).unwrap();
        
        // First two requests should succeed
        assert!(manager.check_rate_limit("test").unwrap());
        assert!(manager.check_rate_limit("test").unwrap());
        
        // Third request should be rate limited
        assert!(!manager.check_rate_limit("test").unwrap());
        
        // Wait for time window to expire
        thread::sleep(Duration::from_millis(150));
        
        // Should work again
        assert!(manager.check_rate_limit("test").unwrap());
    }

    #[test]
    fn test_input_validation() {
        let config = SecurityConfig::default();
        let manager = SecurityManager::new(config).unwrap();
        
        // Valid input
        assert!(manager.validate_input("hello world", "text").is_ok());
        
        // Invalid input (too long)
        let long_input = "a".repeat(20_000);
        assert!(manager.validate_input(&long_input, "text").is_err());
    }
}
