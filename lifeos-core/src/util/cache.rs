//! Simple in-memory cache with TTL

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A cache entry with expiration
struct Entry {
    value: serde_json::Value,
    expires_at: Instant,
}

/// Simple TTL-based cache
pub struct Cache {
    entries: HashMap<String, Entry>,
    ttl: Duration,
}

impl Cache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            entries: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.entries.get(key).and_then(|entry| {
            if Instant::now() < entry.expires_at {
                Some(&entry.value)
            } else {
                None
            }
        })
    }

    pub fn set(&mut self, key: String, value: serde_json::Value) {
        self.entries.insert(key, Entry {
            value,
            expires_at: Instant::now() + self.ttl,
        });
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
