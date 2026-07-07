//! Small thread-safe TTL cache used for search results and provider failures.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct TtlCache<K, V> {
    ttl: Duration,
    map: Mutex<HashMap<K, (Instant, V)>>,
}

impl<K: Eq + Hash + Clone, V: Clone> TtlCache<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            map: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut map = self.map.lock().unwrap();
        match map.get(key) {
            Some((at, v)) if at.elapsed() < self.ttl => Some(v.clone()),
            Some(_) => {
                map.remove(key);
                None
            }
            None => None,
        }
    }

    pub fn put(&self, key: K, value: V) {
        let mut map = self.map.lock().unwrap();
        // Opportunistic cleanup keeps the map from growing unbounded.
        if map.len() > 256 {
            let ttl = self.ttl;
            map.retain(|_, (at, _)| at.elapsed() < ttl);
        }
        map.insert(key, (Instant::now(), value));
    }

    pub fn contains(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
}
