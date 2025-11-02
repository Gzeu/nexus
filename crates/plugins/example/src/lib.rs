//! Example plugin for NEXUS

use nexus_core::{Agent, Result};

/// Example agent implementation
pub struct ExampleAgent {
    name: String,
}

impl ExampleAgent {
    pub fn new() -> Self {
        Self {
            name: "example".to_string(),
        }
    }
}

impl Default for ExampleAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for ExampleAgent {
    fn run(&self) -> String {
        "Example agent executed successfully!".to_string()
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Plugin entry point
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Agent {
    let agent = Box::new(ExampleAgent::new());
    Box::into_raw(agent)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example_agent() {
        let agent = ExampleAgent::new();
        assert_eq!(agent.name(), "example");
        assert!(!agent.run().is_empty());
    }
}
