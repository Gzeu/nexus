//! Example NEXUS Plugin
//! 
//! Demonstrates how to create a plugin for the NEXUS platform

use nexus_core::Agent;

/// Example echo agent that simply returns input
pub struct EchoAgent {
    name: String,
}

impl EchoAgent {
    pub fn new() -> Self {
        Self {
            name: "echo-agent".to_string(),
        }
    }
}

impl Default for EchoAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for EchoAgent {
    fn run(&self) -> String {
        "Echo agent running successfully!".to_string()
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo_agent_works() {
        let agent = EchoAgent::new();
        assert_eq!(agent.name(), "echo-agent");
        assert_eq!(agent.run(), "Echo agent running successfully!");
    }

    #[test]
    fn echo_agent_default() {
        let agent = EchoAgent::default();
        assert_eq!(agent.name(), "echo-agent");
        assert_eq!(agent.run(), "Echo agent running successfully!");
    }
}