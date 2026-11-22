use std::collections::HashMap;

use tiny_skia::Path;

/// Geometry key for a cached path. Floats are stored as bit patterns so the key
/// is `Eq + Hash` and two identical shapes always hit the same entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PathKey {
    bits: [u32; 8],
}

impl PathKey {
    /// Canonicalize -0.0 to +0.0: the two spellings describe the same geometry
    /// but have different bit patterns, which would otherwise miss the cache.
    fn bit_key(value: f32) -> u32 {
        if value == 0.0 {
            0.0_f32.to_bits()
        } else {
            value.to_bits()
        }
    }

    pub fn new(x: f32, y: f32, w: f32, h: f32, radii: [f32; 4]) -> Self {
        Self {
            bits: [
                Self::bit_key(x),
                Self::bit_key(y),
                Self::bit_key(w),
                Self::bit_key(h),
                Self::bit_key(radii[0]),
                Self::bit_key(radii[1]),
                Self::bit_key(radii[2]),
                Self::bit_key(radii[3]),
            ],
        }
    }
}

/// Fixed-capacity LRU cache for shape masks (§K.3). No path is recomputed while
/// it stays hot; the cache holds at most `cap` paths.
pub struct PathCache {
    map: HashMap<PathKey, Path>,
    recency: Vec<PathKey>, // least-recently-used first
    cap: usize,
}

impl PathCache {
    pub const DEFAULT_CAP: usize = 64;

    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAP)
    }

    pub fn with_capacity(cap: usize) -> Self {
        assert!(cap > 0, "path cache capacity must be positive");
        Self {
            map: HashMap::new(),
            recency: Vec::new(),
            cap,
        }
    }

    pub fn cap(&self) -> usize {
        self.cap
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    fn touch(&mut self, key: PathKey) {
        self.recency.retain(|k| *k != key);
        self.recency.push(key);
    }

    pub fn get(&mut self, key: &PathKey) -> Option<Path> {
        let path = self.map.get(key)?.clone();
        self.touch(*key);
        Some(path)
    }

    pub fn insert(&mut self, key: PathKey, path: Path) {
        if !self.map.contains_key(&key) && self.map.len() == self.cap {
            let lru = self.recency.remove(0);
            self.map.remove(&lru);
        }
        self.map.insert(key, path);
        self.touch(key);
    }

    /// Return the cached path or build it once with `f`.
    pub fn get_or_insert_with(&mut self, key: PathKey, f: impl FnOnce() -> Path) -> Path {
        if let Some(path) = self.get(&key) {
            return path;
        }
        let path = f();
        self.insert(key, path.clone());
        path
    }
}

impl Default for PathCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::pill;
    use tiny_skia::Rect;

    fn key(i: u32) -> PathKey {
        PathKey::new(i as f32, 0.0, 10.0, 10.0, [1.0; 4])
    }

    fn path() -> Path {
        pill(Rect::from_xywh(0.0, 0.0, 10.0, 10.0).unwrap())
    }

    #[test]
    fn default_cap_is_64() {
        assert_eq!(PathCache::new().cap(), 64);
    }

    #[test]
    fn evicts_least_recently_used() {
        let mut c = PathCache::with_capacity(2);
        c.insert(key(1), path());
        c.insert(key(2), path());
        assert!(c.get(&key(1)).is_some()); // key 1 is now MRU
        c.insert(key(3), path()); // evicts key 2
        assert!(c.get(&key(2)).is_none());
        assert!(c.get(&key(1)).is_some());
        assert!(c.get(&key(3)).is_some());
    }

    #[test]
    fn never_grows_past_cap() {
        let mut c = PathCache::with_capacity(3);
        for i in 0..10 {
            c.insert(key(i), path());
            assert!(c.len() <= 3);
        }
    }

    #[test]
    fn get_or_insert_builds_once() {
        let mut c = PathCache::new();
        let _ = c.get_or_insert_with(key(7), path);
        let _ = c.get_or_insert_with(key(7), || panic!("should be cached"));
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn negative_zero_hits_the_positive_zero_slot() {
        let neg = PathKey::new(-0.0, 0.0, 10.0, 10.0, [1.0; 4]);
        let pos = PathKey::new(0.0, 0.0, 10.0, 10.0, [1.0; 4]);
        assert_eq!(neg, pos, "-0.0 and +0.0 must normalize to the same key");

        let mut cache = PathCache::new();
        cache.insert(neg, path());
        assert!(cache.get(&pos).is_some(), "0.0 must hit the -0.0 entry");
        assert_eq!(cache.len(), 1);
    }
}
