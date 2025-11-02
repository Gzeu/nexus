//! Rate limiting for NEXUS security

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct RateLimiter {
    limits: Arc<Mutex<HashMap<String, TokenBucket>>>,
    config: RateLimitConfig,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub time_window: Duration,
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

struct TokenBucket {
    tokens: u32,
    last_refill: Instant,
    max_tokens: u32,
}

impl TokenBucket {
    fn new(max_tokens: u32) -> Self {
        Self {
            tokens: max_tokens,
            last_refill: Instant::now(),
            max_tokens,
        }
    }

    fn try_consume(&mut self) -> bool {
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limits: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    pub fn check(&self, key: &str) -> bool {
        if !self.config.enabled {
            return true;
        }

        let mut limits = self.limits.lock().unwrap();
        let bucket = limits.entry(key.to_string()).or_insert_with(|| {
            TokenBucket::new(self.config.max_requests)
        });

        bucket.try_consume()
    }
}
