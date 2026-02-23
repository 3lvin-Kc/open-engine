//! Concurrency control primitives

use std::sync::Arc;

use parking_lot::RwLock;
use dashmap::DashMap;

/// Read-write lock wrapper for entity-level locking
pub struct EntityLock<K: Eq + std::hash::Hash + Clone> {
    locks: Arc<DashMap<K, RwLock<()>>>,
}

impl<K: Eq + std::hash::Hash + Clone> EntityLock<K> {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(DashMap::new()),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            locks: Arc::new(DashMap::with_capacity(capacity)),
        }
    }

    pub fn read<F, R>(&self, key: &K, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let lock = self.locks.entry(key.clone()).or_insert_with(|| RwLock::new(()));
        let _guard = lock.read();
        f()
    }

    pub fn write<F, R>(&self, key: &K, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let lock = self.locks.entry(key.clone()).or_insert_with(|| RwLock::new(()));
        let _guard = lock.write();
        f()
    }
}

impl<K: Eq + std::hash::Hash + Clone> Default for EntityLock<K> {
    fn default() -> Self {
        Self::new()
    }
}

/// Atomic update result
pub struct AtomicUpdate<T> {
    pub previous: Option<T>,
    pub current: T,
}

/// Compare-and-swap operation
pub struct CompareAndSwap<T> {
    previous: Option<T>,
    new: T,
}

impl<T> CompareAndSwap<T> {
    pub fn new(previous: Option<T>, new: T) -> Self {
        Self { previous, new }
    }

    pub fn execute(&self, current: &T) -> bool
    where
        T: PartialEq,
    {
        match &self.previous {
            Some(prev) => current == prev,
            None => true,
        }
    }
}
