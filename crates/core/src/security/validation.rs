//! Input validation (simplified for CI)

use anyhow::{Result, bail};

#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub max_input_length: usize,
    pub xss_protection: bool,
    pub sql_injection_protection: bool,
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

pub fn validate_input(input: &str, input_type: &str, config: &ValidationConfig) -> Result<()> {
    // Check length limits
    if input.len() > config.max_input_length {
        bail!("Input too long: {} > {}", input.len(), config.max_input_length);
    }

    // Basic validation based on type
    match input_type {
        "email" => validate_email(input)?,
        "url" => validate_url(input)?,
        "filename" => validate_filename(input)?,
        _ => {}, // Accept other types for now
    }

    Ok(())
}

fn validate_email(input: &str) -> Result<()> {
    if !input.contains('@') || !input.contains('.') {
        bail!("Invalid email format");
    }
    Ok(())
}

fn validate_url(input: &str) -> Result<()> {
    if !input.starts_with("http://") && !input.starts_with("https://") {
        bail!("URL must start with http:// or https://");
    }
    Ok(())
}

fn validate_filename(input: &str) -> Result<()> {
    if input.contains("../") || input.contains("..\\") {
        bail!("Path traversal detected in filename");
    }
    Ok(())
}
