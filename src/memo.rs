//! Memoization and caching for OUROCHRONOS execution.
//!
//! Provides epoch-level memoization to speed up fixed-point search
//! by caching the results of previously computed states.

use std::collections::HashMap;
use crate::core_types::{Memory, Value};
use crate::vm::EpochStatus;

/// The result of a memoized epoch execution.
#[derive(Debug, Clone)]
pub struct MemoizedResult {
    /// The resulting present state.
    pub present: Memory,
    /// Output produced.
    pub output: Vec<Value>,
    /// Status of execution.
    pub status: EpochStatus,
}

/// Cache for memoizing epoch results.
/// 
/// Keys are state hashes, values are the computed results.
#[derive(Debug, Default)]
pub struct EpochCache {
    cache: HashMap<u64, MemoizedResult>,
    hits: usize,
    misses: usize,
}

impl EpochCache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a cache with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(capacity),
            hits: 0,
            misses: 0,
        }
    }
    
    /// Look up a cached result by state hash.
    pub fn get(&mut self, state_hash: u64) -> Option<&MemoizedResult> {
        if let Some(result) = self.cache.get(&state_hash) {
            self.hits += 1;
            Some(result)
        } else {
            self.misses += 1;
            None
        }
    }
    
    /// Store a result in the cache.
    pub fn insert(&mut self, state_hash: u64, result: MemoizedResult) {
        self.cache.insert(state_hash, result);
    }
    
    /// Check if a state is cached.
    pub fn contains(&self, state_hash: u64) -> bool {
        self.cache.contains_key(&state_hash)
    }
    
    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache.len(),
            hits: self.hits,
            misses: self.misses,
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f64 / (self.hits + self.misses) as f64
            } else {
                0.0
            },
        }
    }
    
    /// Clear the cache.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
    }
}

/// Statistics about cache performance.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of entries in cache.
    pub size: usize,
    /// Number of cache hits.
    pub hits: usize,
    /// Number of cache misses.
    pub misses: usize,
    /// Hit rate (0.0 to 1.0).
    pub hit_rate: f64,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cache: {} entries, {}/{} hits ({:.1}%)", 
            self.size, self.hits, self.hits + self.misses, self.hit_rate * 100.0)
    }
}

/// Incremental computation helper.
/// 
/// Detects which memory cells have changed between epochs
/// and enables delta-based execution (future optimization).
#[derive(Debug, Default)]
pub struct DeltaTracker {
    previous_nonzero: Vec<u16>,
}

impl DeltaTracker {
    /// Create a new delta tracker.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Compute the delta between two memory states.
    /// Returns the addresses that changed.
    pub fn compute_delta(&mut self, old: &Memory, new: &Memory) -> Vec<u16> {
        let mut changed = Vec::new();
        
        // Check all addresses that were non-zero in old
        for &addr in &self.previous_nonzero {
            if old.read(addr) != new.read(addr) {
                changed.push(addr);
            }
        }
        
        // Update tracking for next iteration
        self.previous_nonzero = new.non_zero_cells().into_iter().map(|(addr, _)| addr).collect();
        
        // Also check new non-zero addresses
        for &addr in &self.previous_nonzero {
            if !changed.contains(&addr) && old.read(addr) != new.read(addr) {
                changed.push(addr);
            }
        }
        
        changed
    }
    
    /// Check if states are identical (no delta).
    pub fn is_unchanged(old: &Memory, new: &Memory) -> bool {
        old.values_equal(new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core_types::Value;
    
    #[test]
    fn test_epoch_cache() {
        let mut cache = EpochCache::new();
        
        let result = MemoizedResult {
            present: Memory::new(),
            output: vec![Value::new(42)],
            status: EpochStatus::Finished,
        };
        
        cache.insert(12345, result.clone());
        
        assert!(cache.contains(12345));
        assert!(!cache.contains(99999));
        
        let retrieved = cache.get(12345);
        assert!(retrieved.is_some());
        
        let stats = cache.stats();
        assert_eq!(stats.size, 1);
        assert_eq!(stats.hits, 1);
    }
    
    #[test]
    fn test_delta_tracker() {
        let mut tracker = DeltaTracker::new();
        
        let mut old = Memory::new();
        old.write(0, Value::new(10));
        old.write(1, Value::new(20));
        
        let mut new = Memory::new();
        new.write(0, Value::new(10));  // Same
        new.write(1, Value::new(30));  // Changed
        new.write(2, Value::new(40));  // New
        
        let delta = tracker.compute_delta(&old, &new);
        
        // Should detect changes
        assert!(!delta.is_empty());
    }
}
