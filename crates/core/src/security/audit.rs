//! Security audit logging (simplified for CI)

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub log_security_events: bool,
    pub log_auth_events: bool,
    pub log_data_access: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_security_events: true,
            log_auth_events: true,
            log_data_access: false,
        }
    }
}

pub struct AuditLogger {
    config: AuditConfig,
}

impl AuditLogger {
    pub fn new(config: &AuditConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    pub fn log_security_event(&self, event_type: &str, details: &str) {
        if self.config.enabled {
            tracing::info!("Security event: {} - {}", event_type, details);
        }
    }
}
