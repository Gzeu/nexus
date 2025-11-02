//! Input validation and sanitization for NEXUS
//!
//! This module provides comprehensive input validation to prevent:
//! - XSS attacks
//! - SQL injection
//! - Path traversal attacks
//! - Command injection
//! - Buffer overflow attempts
//! - Malformed data exploitation

use anyhow::{Result, bail};
use std::collections::HashSet;
use std::path::Path;
use tracing::warn;

use super::ValidationConfig;

/// Input validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Input too long: {length} > {max_length}")]
    InputTooLong { length: usize, max_length: usize },
    
    #[error("Potential XSS attack detected: {pattern}")]
    XssDetected { pattern: String },
    
    #[error("Potential SQL injection detected: {pattern}")]
    SqlInjectionDetected { pattern: String },
    
    #[error("Path traversal attempt detected: {path}")]
    PathTraversalDetected { path: String },
    
    #[error("Command injection attempt detected: {pattern}")]
    CommandInjectionDetected { pattern: String },
    
    #[error("Invalid characters detected: {chars}")]
    InvalidCharacters { chars: String },
    
    #[error("Empty input not allowed for type: {input_type}")]
    EmptyInput { input_type: String },
    
    #[error("Invalid format for type {input_type}: {details}")]
    InvalidFormat { input_type: String, details: String },
}

/// Validate input according to security policies
pub fn validate_input(input: &str, input_type: &str, config: &ValidationConfig) -> Result<()> {
    // Check length limits
    if input.len() > config.max_input_length {
        bail!(ValidationError::InputTooLong {
            length: input.len(),
            max_length: config.max_input_length,
        });
    }

    // Check for empty input where not allowed
    if input.is_empty() && !allows_empty_input(input_type) {
        bail!(ValidationError::EmptyInput {
            input_type: input_type.to_string(),
        });
    }

    // Type-specific validation
    match input_type {
        "email" => validate_email(input)?,
        "url" => validate_url(input)?,
        "filename" => validate_filename(input)?,
        "path" => validate_path(input, config)?,
        "command" => validate_command(input)?,
        "sql" => validate_sql(input, config)?,
        "html" => validate_html(input, config)?,
        "json" => validate_json(input)?,
        "alphanumeric" => validate_alphanumeric(input)?,
        "text" => validate_text(input, config)?,
        _ => validate_general(input, config)?,
    }

    Ok(())
}

/// Check if input type allows empty input
fn allows_empty_input(input_type: &str) -> bool {
    matches!(input_type, "text" | "description" | "comment" | "optional")
}

/// Validate email addresses
fn validate_email(input: &str) -> Result<()> {
    // Basic email validation (not RFC 5322 compliant, but secure)
    let email_pattern = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .map_err(|e| anyhow::anyhow!("Regex compilation failed: {}", e))?;
    
    if !email_pattern.is_match(input) {
        bail!(ValidationError::InvalidFormat {
            input_type: "email".to_string(),
            details: "Invalid email format".to_string(),
        });
    }
    
    Ok(())
}

/// Validate URLs
fn validate_url(input: &str) -> Result<()> {
    // Basic URL validation
    if !input.starts_with("http://") && !input.starts_with("https://") {
        bail!(ValidationError::InvalidFormat {
            input_type: "url".to_string(),
            details: "Must start with http:// or https://".to_string(),
        });
    }
    
    // Check for dangerous characters
    let dangerous_chars: HashSet<char> = ['<', '>', '"', '\'', '`'].iter().copied().collect();
    let found_dangerous: Vec<char> = input.chars().filter(|c| dangerous_chars.contains(c)).collect();
    
    if !found_dangerous.is_empty() {
        bail!(ValidationError::InvalidCharacters {
            chars: found_dangerous.iter().collect::<String>(),
        });
    }
    
    Ok(())
}

/// Validate filenames
fn validate_filename(input: &str) -> Result<()> {
    // Check for path traversal
    if input.contains("../") || input.contains("..\\") {
        bail!(ValidationError::PathTraversalDetected {
            path: input.to_string(),
        });
    }
    
    // Check for dangerous characters
    let dangerous_chars: HashSet<char> = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'].iter().copied().collect();
    let found_dangerous: Vec<char> = input.chars().filter(|c| dangerous_chars.contains(c)).collect();
    
    if !found_dangerous.is_empty() {
        bail!(ValidationError::InvalidCharacters {
            chars: found_dangerous.iter().collect::<String>(),
        });
    }
    
    Ok(())
}

/// Validate file paths
fn validate_path(input: &str, config: &ValidationConfig) -> Result<()> {
    if config.path_traversal_protection {
        // Check for path traversal attempts
        if input.contains("../") || input.contains("..\\") {
            bail!(ValidationError::PathTraversalDetected {
                path: input.to_string(),
            });
        }
        
        // Normalize and check the path
        let path = Path::new(input);
        if let Ok(canonical) = path.canonicalize() {
            let canonical_str = canonical.to_string_lossy();
            if canonical_str.contains("..") {
                bail!(ValidationError::PathTraversalDetected {
                    path: input.to_string(),
                });
            }
        }
    }
    
    Ok(())
}

/// Validate commands for injection attempts
fn validate_command(input: &str) -> Result<()> {
    // Check for command injection patterns
    let dangerous_patterns = [
        ";", "||", "&&", "`", "$(", "${"  // Command chaining and substitution
    ];
    
    for pattern in &dangerous_patterns {
        if input.contains(pattern) {
            bail!(ValidationError::CommandInjectionDetected {
                pattern: pattern.to_string(),
            });
        }
    }
    
    Ok(())
}

/// Validate SQL input for injection attempts
fn validate_sql(input: &str, config: &ValidationConfig) -> Result<()> {
    if !config.sql_injection_protection {
        return Ok(());
    }
    
    let input_lower = input.to_lowercase();
    
    // Check for SQL injection patterns
    let dangerous_patterns = [
        "' or '1'='1",
        "' or 1=1",
        "union select",
        "drop table",
        "delete from",
        "insert into",
        "update set",
        "exec(",
        "execute(",
        "sp_",
        "xp_",
        ";", // Statement terminator
    ];
    
    for pattern in &dangerous_patterns {
        if input_lower.contains(pattern) {
            warn!("Potential SQL injection detected: {}", pattern);
            bail!(ValidationError::SqlInjectionDetected {
                pattern: pattern.to_string(),
            });
        }
    }
    
    Ok(())
}

/// Validate HTML input for XSS attempts
fn validate_html(input: &str, config: &ValidationConfig) -> Result<()> {
    if !config.xss_protection {
        return Ok(());
    }
    
    let input_lower = input.to_lowercase();
    
    // Check for XSS patterns
    let dangerous_patterns = [
        "<script",
        "javascript:",
        "vbscript:",
        "onload=",
        "onerror=",
        "onclick=",
        "onmouseover=",
        "onfocus=",
        "onblur=",
        "onchange=",
        "onsubmit=",
        "<iframe",
        "<object",
        "<embed",
        "<link",
        "<meta",
        "<style",
        "expression(",
        "url(",
        "@import",
    ];
    
    for pattern in &dangerous_patterns {
        if input_lower.contains(pattern) {
            warn!("Potential XSS detected: {}", pattern);
            bail!(ValidationError::XssDetected {
                pattern: pattern.to_string(),
            });
        }
    }
    
    Ok(())
}

/// Validate JSON input
fn validate_json(input: &str) -> Result<()> {
    // Try to parse as JSON to ensure it's valid
    serde_json::from_str::<serde_json::Value>(input)
        .map_err(|e| ValidationError::InvalidFormat {
            input_type: "json".to_string(),
            details: format!("JSON parse error: {}", e),
        })?;
    
    Ok(())
}

/// Validate alphanumeric input
fn validate_alphanumeric(input: &str) -> Result<()> {
    if !input.chars().all(|c| c.is_alphanumeric()) {
        let invalid_chars: Vec<char> = input.chars().filter(|c| !c.is_alphanumeric()).collect();
        bail!(ValidationError::InvalidCharacters {
            chars: invalid_chars.iter().collect::<String>(),
        });
    }
    
    Ok(())
}

/// Validate general text input
fn validate_text(input: &str, config: &ValidationConfig) -> Result<()> {
    // Check for null bytes
    if input.contains('\0') {
        bail!(ValidationError::InvalidCharacters {
            chars: "\0".to_string(),
        });
    }
    
    // Basic XSS protection for text input
    if config.xss_protection {
        validate_html(input, config)?;
    }
    
    Ok(())
}

/// General validation for unknown input types
fn validate_general(input: &str, config: &ValidationConfig) -> Result<()> {
    // Apply basic protections
    validate_text(input, config)?;
    
    Ok(())
}

/// Sanitize input by removing/escaping dangerous characters
pub fn sanitize_input(input: &str, input_type: &str) -> String {
    match input_type {
        "html" => sanitize_html(input),
        "filename" => sanitize_filename(input),
        "path" => sanitize_path(input),
        _ => sanitize_general(input),
    }
}

/// Sanitize HTML by escaping dangerous characters
fn sanitize_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('/', "&#x2F;")
}

/// Sanitize filename by removing dangerous characters
fn sanitize_filename(input: &str) -> String {
    input
        .chars()
        .filter(|&c| !['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'].contains(&c))
        .collect()
}

/// Sanitize path by removing path traversal attempts
fn sanitize_path(input: &str) -> String {
    input
        .replace("../", "")
        .replace("..\\", "")
        .replace("\0", "")
}

/// General sanitization
fn sanitize_general(input: &str) -> String {
    // Remove null bytes and control characters
    input
        .chars()
        .filter(|&c| c >= ' ' || c == '\n' || c == '\r' || c == '\t')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ValidationConfig {
        ValidationConfig::default()
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name+tag@domain.co.uk").is_ok());
        assert!(validate_email("invalid-email").is_err());
        assert!(validate_email("@domain.com").is_err());
        assert!(validate_email("").is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://localhost:8080").is_ok());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("https://example.com<script>").is_err());
    }

    #[test]
    fn test_validate_filename() {
        assert!(validate_filename("document.txt").is_ok());
        assert!(validate_filename("my-file_v2.pdf").is_ok());
        assert!(validate_filename("../../../etc/passwd").is_err());
        assert!(validate_filename("file|with|pipes.txt").is_err());
    }

    #[test]
    fn test_validate_path() {
        let config = test_config();
        assert!(validate_path("/home/user/document.txt", &config).is_ok());
        assert!(validate_path("../../../etc/passwd", &config).is_err());
        assert!(validate_path("folder/../secret", &config).is_err());
    }

    #[test]
    fn test_validate_sql() {
        let config = test_config();
        assert!(validate_sql("SELECT name FROM users WHERE id = 1", &config).is_ok());
        assert!(validate_sql("' OR '1'='1", &config).is_err());
        assert!(validate_sql("UNION SELECT password FROM admin", &config).is_err());
        assert!(validate_sql("DROP TABLE users", &config).is_err());
    }

    #[test]
    fn test_validate_html() {
        let config = test_config();
        assert!(validate_html("<p>Hello world</p>", &config).is_ok());
        assert!(validate_html("<script>alert('xss')</script>", &config).is_err());
        assert!(validate_html("<img src=x onerror=alert(1)>", &config).is_err());
        assert!(validate_html("javascript:alert(1)", &config).is_err());
    }

    #[test]
    fn test_sanitize_html() {
        let input = "<script>alert('xss')</script>";
        let sanitized = sanitize_html(input);
        assert!(!sanitized.contains("<script>"));
        assert!(sanitized.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_sanitize_filename() {
        let input = "../../../file|name*.txt";
        let sanitized = sanitize_filename(input);
        assert!(!sanitized.contains("../"));
        assert!(!sanitized.contains("|"));
        assert!(!sanitized.contains("*"));
    }

    #[test]
    fn test_input_length_validation() {
        let mut config = test_config();
        config.max_input_length = 10;
        
        assert!(validate_input("short", "text", &config).is_ok());
        assert!(validate_input("this is too long", "text", &config).is_err());
    }
}
