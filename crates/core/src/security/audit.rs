//! Security audit logging for NEXUS
//!
//! This module provides comprehensive security event logging and audit trails
//! to help detect, investigate, and respond to security incidents.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

use super::AuditConfig;

/// Security event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    /// Authentication events
    Authentication {
        event: AuthEvent,
        user_id: Option<String>,
        ip_address: Option<String>,
    },
    
    /// Authorization events
    Authorization {
        event: AuthzEvent,
        user_id: String,
        resource: String,
        action: String,
    },
    
    /// Data access events
    DataAccess {
        event: DataEvent,
        user_id: String,
        resource_type: String,
        resource_id: String,
    },
    
    /// Security policy violations
    PolicyViolation {
        policy: String,
        violation_type: String,
        details: String,
    },
    
    /// System security events
    SystemSecurity {
        event: SystemEvent,
        component: String,
        details: String,
    },
    
    /// Custom security events
    Custom {
        event_type: String,
        details: HashMap<String, String>,
    },
}

/// Authentication event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthEvent {
    LoginSuccess,
    LoginFailure,
    Logout,
    PasswordChange,
    AccountLocked,
    AccountUnlocked,
    TokenGenerated,
    TokenRevoked,
    TwoFactorEnabled,
    TwoFactorDisabled,
}

/// Authorization event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthzEvent {
    AccessGranted,
    AccessDenied,
    PermissionChanged,
    RoleAssigned,
    RoleRevoked,
}

/// Data access event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataEvent {
    Create,
    Read,
    Update,
    Delete,
    Export,
    Import,
    Backup,
    Restore,
}

/// System security event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    ServiceStarted,
    ServiceStopped,
    ConfigurationChanged,
    SecurityPolicyUpdated,
    CertificateExpiring,
    CertificateRenewed,
    SecurityScanCompleted,
    VulnerabilityDetected,
    IntrusionDetected,
    AnomalyDetected,
}

/// Security audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,
    /// Timestamp in Unix epoch seconds
    pub timestamp: u64,
    /// Event severity level
    pub severity: SecuritySeverity,
    /// Event type and details
    pub event_type: SecurityEventType,
    /// Source component that generated the event
    pub source: String,
    /// Optional correlation ID for related events
    pub correlation_id: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Security event severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecuritySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl SecuritySeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecuritySeverity::Info => "INFO",
            SecuritySeverity::Low => "LOW",
            SecuritySeverity::Medium => "MEDIUM",
            SecuritySeverity::High => "HIGH",
            SecuritySeverity::Critical => "CRITICAL",
        }
    }
}

/// Security audit logger
pub struct AuditLogger {
    config: AuditConfig,
    events: Arc<Mutex<Vec<AuditEvent>>>,
    event_counter: Arc<Mutex<u64>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(config: &AuditConfig) -> Result<Self> {
        info!("Initializing security audit logger");
        
        Ok(Self {
            config: config.clone(),
            events: Arc::new(Mutex::new(Vec::new())),
            event_counter: Arc::new(Mutex::new(0)),
        })
    }

    /// Log a security event
    pub fn log_event(&self, event: AuditEvent) {
        if !self.config.enabled {
            return;
        }

        // Log to tracing based on severity
        match event.severity {
            SecuritySeverity::Info => info!("Security event: {}", serde_json::to_string(&event).unwrap_or_default()),
            SecuritySeverity::Low => info!("Security event: {}", serde_json::to_string(&event).unwrap_or_default()),
            SecuritySeverity::Medium => warn!("Security event: {}", serde_json::to_string(&event).unwrap_or_default()),
            SecuritySeverity::High => warn!("Security event: {}", serde_json::to_string(&event).unwrap_or_default()),
            SecuritySeverity::Critical => error!("Security event: {}", serde_json::to_string(&event).unwrap_or_default()),
        }

        // Store in memory (in production, this would also go to persistent storage)
        let mut events = self.events.lock().unwrap();
        events.push(event);
        
        // Keep only the last 1000 events in memory
        if events.len() > 1000 {
            events.remove(0);
        }
    }

    /// Log a security event with automatic ID generation
    pub fn log_security_event(&self, event_type: &str, details: &str) {
        let event = AuditEvent {
            id: self.generate_event_id(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            severity: SecuritySeverity::Medium,
            event_type: SecurityEventType::Custom {
                event_type: event_type.to_string(),
                details: [("details".to_string(), details.to_string())].iter().cloned().collect(),
            },
            source: "nexus-core".to_string(),
            correlation_id: None,
            metadata: HashMap::new(),
        };
        
        self.log_event(event);
    }

    /// Log an authentication event
    pub fn log_auth_event(
        &self,
        auth_event: AuthEvent,
        user_id: Option<String>,
        ip_address: Option<String>,
        severity: SecuritySeverity,
    ) {
        if !self.config.log_auth_events {
            return;
        }

        let event = AuditEvent {
            id: self.generate_event_id(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            severity,
            event_type: SecurityEventType::Authentication {
                event: auth_event,
                user_id,
                ip_address,
            },
            source: "nexus-auth".to_string(),
            correlation_id: None,
            metadata: HashMap::new(),
        };
        
        self.log_event(event);
    }

    /// Log an authorization event
    pub fn log_authz_event(
        &self,
        authz_event: AuthzEvent,
        user_id: String,
        resource: String,
        action: String,
        severity: SecuritySeverity,
    ) {
        let event = AuditEvent {
            id: self.generate_event_id(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            severity,
            event_type: SecurityEventType::Authorization {
                event: authz_event,
                user_id,
                resource,
                action,
            },
            source: "nexus-authz".to_string(),
            correlation_id: None,
            metadata: HashMap::new(),
        };
        
        self.log_event(event);
    }

    /// Log a data access event
    pub fn log_data_event(
        &self,
        data_event: DataEvent,
        user_id: String,
        resource_type: String,
        resource_id: String,
    ) {
        if !self.config.log_data_access {
            return;
        }

        let event = AuditEvent {
            id: self.generate_event_id(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            severity: SecuritySeverity::Info,
            event_type: SecurityEventType::DataAccess {
                event: data_event,
                user_id,
                resource_type,
                resource_id,
            },
            source: "nexus-data".to_string(),
            correlation_id: None,
            metadata: HashMap::new(),
        };
        
        self.log_event(event);
    }

    /// Log a system security event
    pub fn log_system_event(
        &self,
        system_event: SystemEvent,
        component: String,
        details: String,
        severity: SecuritySeverity,
    ) {
        let event = AuditEvent {
            id: self.generate_event_id(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            severity,
            event_type: SecurityEventType::SystemSecurity {
                event: system_event,
                component,
                details,
            },
            source: "nexus-system".to_string(),
            correlation_id: None,
            metadata: HashMap::new(),
        };
        
        self.log_event(event);
    }

    /// Get recent security events
    pub fn get_recent_events(&self, count: usize) -> Vec<AuditEvent> {
        let events = self.events.lock().unwrap();
        events.iter().rev().take(count).cloned().collect()
    }

    /// Get events by severity
    pub fn get_events_by_severity(&self, severity: SecuritySeverity) -> Vec<AuditEvent> {
        let events = self.events.lock().unwrap();
        events.iter().filter(|e| e.severity >= severity).cloned().collect()
    }

    /// Get events in time range
    pub fn get_events_in_range(&self, start_timestamp: u64, end_timestamp: u64) -> Vec<AuditEvent> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.timestamp >= start_timestamp && e.timestamp <= end_timestamp)
            .cloned()
            .collect()
    }

    /// Generate audit statistics
    pub fn get_audit_stats(&self) -> AuditStats {
        let events = self.events.lock().unwrap();
        let total_events = events.len();
        
        let mut severity_counts = HashMap::new();
        let mut event_type_counts = HashMap::new();
        
        for event in events.iter() {
            *severity_counts.entry(event.severity.as_str().to_string()).or_insert(0) += 1;
            
            let event_type_name = match &event.event_type {
                SecurityEventType::Authentication { .. } => "Authentication",
                SecurityEventType::Authorization { .. } => "Authorization",
                SecurityEventType::DataAccess { .. } => "DataAccess",
                SecurityEventType::PolicyViolation { .. } => "PolicyViolation",
                SecurityEventType::SystemSecurity { .. } => "SystemSecurity",
                SecurityEventType::Custom { .. } => "Custom",
            };
            
            *event_type_counts.entry(event_type_name.to_string()).or_insert(0) += 1;
        }
        
        AuditStats {
            total_events,
            severity_counts,
            event_type_counts,
        }
    }

    /// Generate a unique event ID
    fn generate_event_id(&self) -> String {
        let mut counter = self.event_counter.lock().unwrap();
        *counter += 1;
        format!("audit-{}-{}", 
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(),
            *counter
        )
    }

    /// Export events as JSON
    pub fn export_events_json(&self) -> Result<String> {
        let events = self.events.lock().unwrap();
        serde_json::to_string_pretty(&*events)
            .map_err(|e| anyhow::anyhow!("Failed to serialize events: {}", e))
    }
}

/// Audit statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_events: usize,
    pub severity_counts: HashMap<String, usize>,
    pub event_type_counts: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_config() -> AuditConfig {
        AuditConfig {
            enabled: true,
            log_security_events: true,
            log_auth_events: true,
            log_data_access: true,
        }
    }

    #[test]
    fn test_audit_logger_creation() {
        let config = test_config();
        let logger = AuditLogger::new(&config).unwrap();
        
        let stats = logger.get_audit_stats();
        assert_eq!(stats.total_events, 0);
    }

    #[test]
    fn test_log_security_event() {
        let config = test_config();
        let logger = AuditLogger::new(&config).unwrap();
        
        logger.log_security_event("test_event", "Test details");
        
        let events = logger.get_recent_events(10);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].severity, SecuritySeverity::Medium);
    }

    #[test]
    fn test_log_auth_event() {
        let config = test_config();
        let logger = AuditLogger::new(&config).unwrap();
        
        logger.log_auth_event(
            AuthEvent::LoginSuccess,
            Some("user123".to_string()),
            Some("192.168.1.1".to_string()),
            SecuritySeverity::Info,
        );
        
        let events = logger.get_recent_events(10);
        assert_eq!(events.len(), 1);
        
        if let SecurityEventType::Authentication { event, user_id, .. } = &events[0].event_type {
            assert!(matches!(event, AuthEvent::LoginSuccess));
            assert_eq!(user_id.as_ref().unwrap(), "user123");
        } else {
            panic!("Expected Authentication event");
        }
    }

    #[test]
    fn test_get_events_by_severity() {
        let config = test_config();
        let logger = AuditLogger::new(&config).unwrap();
        
        logger.log_auth_event(
            AuthEvent::LoginSuccess,
            None,
            None,
            SecuritySeverity::Info,
        );
        
        logger.log_auth_event(
            AuthEvent::LoginFailure,
            None,
            None,
            SecuritySeverity::High,
        );
        
        let high_events = logger.get_events_by_severity(SecuritySeverity::High);
        assert_eq!(high_events.len(), 1);
        
        let all_events = logger.get_events_by_severity(SecuritySeverity::Info);
        assert_eq!(all_events.len(), 2);
    }

    #[test]
    fn test_audit_stats() {
        let config = test_config();
        let logger = AuditLogger::new(&config).unwrap();
        
        logger.log_security_event("event1", "details1");
        logger.log_security_event("event2", "details2");
        
        let stats = logger.get_audit_stats();
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.severity_counts.get("MEDIUM"), Some(&2));
        assert_eq!(stats.event_type_counts.get("Custom"), Some(&2));
    }

    #[test]
    fn test_event_serialization() {
        let event = AuditEvent {
            id: "test-123".to_string(),
            timestamp: 1234567890,
            severity: SecuritySeverity::High,
            event_type: SecurityEventType::Authentication {
                event: AuthEvent::LoginFailure,
                user_id: Some("user123".to_string()),
                ip_address: Some("192.168.1.1".to_string()),
            },
            source: "test".to_string(),
            correlation_id: None,
            metadata: HashMap::new(),
        };
        
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: AuditEvent = serde_json::from_str(&json).unwrap();
        
        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.severity, deserialized.severity);
    }
}
