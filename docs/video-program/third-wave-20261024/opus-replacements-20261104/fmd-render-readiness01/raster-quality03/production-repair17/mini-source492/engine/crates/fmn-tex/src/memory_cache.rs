//! Engine-local, bounded LRU of canonical FMNTEX payloads. Keeping the
//! encoded representation makes the byte budget exact and preserves the
//! codec's fallible allocation and bit-for-bit span/layout reconstruction.

use fmn_cache::CacheKey;
use std::collections::VecDeque;
use std::sync::Arc;

/// Maximum number of typeset documents retained by one engine.
pub const TYPESET_MEMORY_CACHE_MAX_ENTRIES: usize = 128;
/// Maximum encoded payload bytes retained by one engine (metadata is bounded
/// separately by [`TYPESET_MEMORY_CACHE_MAX_ENTRIES`]).
pub const TYPESET_MEMORY_CACHE_MAX_BYTES: usize = 4 * 1024 * 1024;

/// Diagnostics for an engine's memory cache, independent of scene state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TypesetCacheStats {
    /// Lookups served from a resident payload.
    pub hits: u64,
    /// Lookups that needed the disk cache or a new layout. Failed layouts
    /// count as misses too; requests without a complete key are not counted.
    pub misses: u64,
    /// Currently retained documents.
    pub entries: usize,
    /// Currently retained encoded payload bytes, excluding caller-owned data.
    pub bytes: usize,
}

struct Entry {
    key: CacheKey,
    bytes: Arc<[u8]>,
}

#[derive(Default)]
pub(crate) struct MemoryCache {
    // Oldest use at the front. At most 128 keys keeps a bounded linear lookup
    // small and avoids an independently allocated map/order index.
    entries: VecDeque<Entry>,
    stats: TypesetCacheStats,
}

impl MemoryCache {
    pub(crate) fn stats(&self) -> TypesetCacheStats {
        self.stats
    }

    pub(crate) fn get(&mut self, key: &CacheKey) -> Option<Arc<[u8]>> {
        let entry = self
            .entries
            .iter()
            .position(|entry| entry.key == *key)
            .and_then(|index| self.entries.remove(index));
        let Some(entry) = entry else {
            self.stats.misses = self.stats.misses.saturating_add(1);
            return None;
        };
        self.stats.hits = self.stats.hits.saturating_add(1);
        let bytes = Arc::clone(&entry.bytes);
        self.entries.push_back(entry);
        Some(bytes)
    }

    /// Drop every resident payload. Hit and miss counters are cumulative and
    /// survive; readers holding a payload keep their own `Arc`.
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.stats.entries = 0;
        self.stats.bytes = 0;
    }

    pub(crate) fn insert(&mut self, key: CacheKey, bytes: Vec<u8>) {
        if bytes.len() > TYPESET_MEMORY_CACHE_MAX_BYTES
            || self.entries.iter().any(|entry| entry.key == key)
        {
            // Concurrent cold requests may finish in either order. Keep the
            // already-published immutable payload; never duplicate accounting.
            return;
        }
        if self.entries.len() < TYPESET_MEMORY_CACHE_MAX_ENTRIES
            && self.entries.try_reserve(1).is_err()
        {
            return;
        }
        while self.entries.len() >= TYPESET_MEMORY_CACHE_MAX_ENTRIES
            || self.stats.bytes + bytes.len() > TYPESET_MEMORY_CACHE_MAX_BYTES
        {
            let Some(oldest) = self.entries.pop_front() else {
                break;
            };
            self.stats.bytes -= oldest.bytes.len();
        }
        self.stats.bytes += bytes.len();
        self.entries.push_back(Entry {
            key,
            bytes: bytes.into(),
        });
        self.stats.entries = self.entries.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(index: usize) -> CacheKey {
        CacheKey::of_content(index.to_string().as_bytes())
    }

    #[test]
    fn entry_limit_evicts_the_least_recently_used_document() {
        let mut cache = MemoryCache::default();
        for index in 0..TYPESET_MEMORY_CACHE_MAX_ENTRIES {
            cache.insert(key(index), vec![0; 8]);
        }
        assert!(cache.get(&key(0)).is_some());
        cache.insert(key(TYPESET_MEMORY_CACHE_MAX_ENTRIES), vec![1; 8]);
        assert!(cache.get(&key(0)).is_some());
        assert!(cache.get(&key(1)).is_none());
        assert_eq!(cache.stats.entries, TYPESET_MEMORY_CACHE_MAX_ENTRIES);
        assert_eq!(cache.stats.bytes, TYPESET_MEMORY_CACHE_MAX_ENTRIES * 8);
        // Repeated publication from a racing miss cannot consume the budget.
        cache.insert(key(0), vec![9; 100]);
        assert_eq!(cache.stats.bytes, TYPESET_MEMORY_CACHE_MAX_ENTRIES * 8);
    }

    #[test]
    fn clear_empties_residency_but_keeps_cumulative_counters_and_readers() {
        let mut cache = MemoryCache::default();
        cache.insert(key(0), vec![5; 16]);
        let retained = cache.get(&key(0)).unwrap();
        assert!(cache.get(&key(1)).is_none());
        cache.clear();
        assert_eq!(
            cache.stats(),
            TypesetCacheStats {
                hits: 1,
                misses: 1,
                entries: 0,
                bytes: 0,
            }
        );
        assert!(cache.get(&key(0)).is_none());
        assert_eq!(cache.stats().misses, 2);
        assert_eq!(&*retained, &[5; 16]);
        // The budget is fully available again after a clear.
        cache.insert(key(2), vec![1; TYPESET_MEMORY_CACHE_MAX_BYTES]);
        assert_eq!(cache.stats().bytes, TYPESET_MEMORY_CACHE_MAX_BYTES);
    }

    #[test]
    fn byte_limit_and_oversize_bypass_keep_outstanding_readers_valid() {
        let mut cache = MemoryCache::default();
        cache.insert(key(0), vec![7; TYPESET_MEMORY_CACHE_MAX_BYTES]);
        let retained = cache.get(&key(0)).unwrap();
        cache.insert(key(1), vec![9; TYPESET_MEMORY_CACHE_MAX_BYTES + 1]);
        assert_eq!(cache.stats.entries, 1);
        assert_eq!(cache.stats.bytes, TYPESET_MEMORY_CACHE_MAX_BYTES);
        cache.insert(key(2), vec![3; 10]);
        assert!(cache.get(&key(0)).is_none());
        assert_eq!(cache.stats.entries, 1);
        assert_eq!(cache.stats.bytes, 10);
        assert_eq!(retained.len(), TYPESET_MEMORY_CACHE_MAX_BYTES);
        assert!(retained.iter().all(|byte| *byte == 7));
    }
}
