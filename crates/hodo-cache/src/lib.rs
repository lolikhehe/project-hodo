//! Size-capped disk cache. Payloads live in files; only tiny LRU metadata stays in RAM.

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("io: {0}")]
    Io(String),
    #[error("payload exceeds cache cap")]
    TooLarge,
    #[error("corrupt cache entry")]
    Corrupt,
}

#[derive(Debug)]
pub struct CacheEntry {
    pub content_type: String,
    pub data: Vec<u8>,
}

struct EntryMeta {
    hash: u64,
    content_type: String,
    size: u64,
    checksum: u64,
    last_used: u64,
}

pub struct DiskCache {
    root: PathBuf,
    max_total_bytes: u64,
    index: Vec<EntryMeta>,
    total: u64,
    clock: u64,
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x1000_0000_01b3;
    bytes.iter().fold(OFFSET, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(PRIME)
    })
}

impl DiskCache {
    /// Opens (or creates) a cache under `root`. Metadata is process-local; payloads are
    /// files at `root/cache/<fnv64-hex>`.
    pub fn open(root: &Path, max_total_bytes: u64) -> Result<Self, CacheError> {
        let root = root.to_path_buf();
        fs::create_dir_all(root.join("cache")).map_err(|e| CacheError::Io(e.to_string()))?;
        Ok(Self {
            root,
            max_total_bytes,
            index: Vec::new(),
            total: 0,
            clock: 0,
        })
    }

    fn entry_path(&self, hash: u64) -> PathBuf {
        self.root.join("cache").join(format!("{hash:016x}"))
    }

    fn remove_at(&mut self, pos: usize) {
        let meta = self.index.swap_remove(pos);
        self.total -= meta.size;
    }

    fn evict_one(&mut self) {
        if let Some(pos) = self
            .index
            .iter()
            .enumerate()
            .min_by_key(|(_, e)| e.last_used)
            .map(|(pos, _)| pos)
        {
            let hash = self.index[pos].hash;
            self.remove_at(pos);
            let _ = fs::remove_file(self.entry_path(hash));
        }
    }

    pub fn put(
        &mut self,
        url_key: &str,
        content_type: &str,
        data: &[u8],
    ) -> Result<(), CacheError> {
        let size = data.len() as u64;
        if size > self.max_total_bytes / 2 {
            return Err(CacheError::TooLarge);
        }
        let hash = fnv1a_64(url_key.as_bytes());
        fs::write(self.entry_path(hash), data).map_err(|e| CacheError::Io(e.to_string()))?;

        // A re-put replaces the entry instead of counting it twice.
        if let Some(pos) = self.index.iter().position(|e| e.hash == hash) {
            self.remove_at(pos);
        }
        while self.total + size > self.max_total_bytes && !self.index.is_empty() {
            self.evict_one();
        }

        self.clock += 1;
        self.index.push(EntryMeta {
            hash,
            content_type: content_type.to_string(),
            size,
            checksum: fnv1a_64(data),
            last_used: self.clock,
        });
        self.total += size;
        Ok(())
    }

    pub fn get(&mut self, url_key: &str) -> Option<CacheEntry> {
        let hash = fnv1a_64(url_key.as_bytes());
        let pos = self.index.iter().position(|e| e.hash == hash)?;
        let path = self.entry_path(hash);
        let data = match fs::read(&path) {
            Ok(data) => data,
            Err(_) => {
                self.remove_at(pos);
                return None;
            }
        };
        let (size, checksum, content_type) = {
            let meta = &self.index[pos];
            (meta.size, meta.checksum, meta.content_type.clone())
        };
        if size != data.len() as u64 || checksum != fnv1a_64(&data) {
            self.remove_at(pos);
            let _ = fs::remove_file(&path);
            return None;
        }
        self.clock += 1;
        self.index[pos].last_used = self.clock;
        Some(CacheEntry { content_type, data })
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn total_bytes(&self) -> u64 {
        self.total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

    struct TempCache {
        root: PathBuf,
        cache: DiskCache,
    }

    impl TempCache {
        fn new(max_total_bytes: u64) -> Self {
            let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!("hodo-cache-{}-{n}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(&root).unwrap();
            let cache = DiskCache::open(&root, max_total_bytes).unwrap();
            Self { root, cache }
        }

        fn path_for(&self, key: &str) -> PathBuf {
            self.cache.entry_path(fnv1a_64(key.as_bytes()))
        }
    }

    impl Drop for TempCache {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn roundtrip() {
        let mut t = TempCache::new(4096);
        assert!(t.cache.is_empty());
        t.cache
            .put("https://x/a.css", "text/css", b"body{}")
            .unwrap();
        t.cache
            .put("https://x/a.png", "image/png", &[1, 2, 3])
            .unwrap();
        assert_eq!(t.cache.len(), 2);
        assert_eq!(t.cache.total_bytes(), 9);

        let css = t.cache.get("https://x/a.css").unwrap();
        assert_eq!(css.content_type, "text/css");
        assert_eq!(css.data, b"body{}".to_vec());
        let png = t.cache.get("https://x/a.png").unwrap();
        assert_eq!(png.content_type, "image/png");
        assert_eq!(png.data, vec![1, 2, 3]);
        assert!(t.cache.get("https://x/missing").is_none());
    }

    #[test]
    fn lru_eviction() {
        let mut t = TempCache::new(1024);
        for (i, key) in ["a", "b", "c", "d"].iter().enumerate() {
            t.cache
                .put(key, "text/plain", &vec![b'a' + i as u8; 300])
                .unwrap();
        }
        // 4 x 300 > 1024: the oldest insertion went first.
        assert_eq!(t.cache.len(), 3);
        assert!(t.cache.get("a").is_none());
        assert!(t.path_for("b").exists());
        assert_eq!(t.cache.total_bytes(), 900);

        // Refreshing "b" makes "c" the next victim (not "b").
        assert!(t.cache.get("b").is_some());
        t.cache.put("e", "text/plain", &[b'e'; 300]).unwrap();
        assert!(t.cache.get("c").is_none());
        assert!(t.cache.get("b").is_some());
        assert!(t.cache.get("d").is_some());
        assert!(t.cache.get("e").is_some());
        assert_eq!(t.cache.total_bytes(), 900);
    }

    #[test]
    fn too_large_entry_rejected() {
        let mut t = TempCache::new(1024);
        assert!(matches!(
            t.cache.put("k", "text/plain", &[0u8; 513]),
            Err(CacheError::TooLarge)
        ));
        assert!(t.cache.is_empty());
        t.cache.put("k", "text/plain", &[1u8; 512]).unwrap();
        assert_eq!(t.cache.total_bytes(), 512);
    }

    #[test]
    fn corrupt_entry_is_pruned() {
        let mut t = TempCache::new(4096);
        t.cache.put("k", "text/plain", b"hello").unwrap();
        fs::remove_file(t.path_for("k")).unwrap();
        assert!(t.cache.get("k").is_none());
        assert!(t.cache.is_empty());
        assert_eq!(t.cache.total_bytes(), 0);

        // Same-length modification is caught by the checksum.
        t.cache.put("k", "text/plain", b"hello").unwrap();
        fs::write(t.path_for("k"), b"world").unwrap();
        assert!(t.cache.get("k").is_none());
        assert!(t.cache.is_empty());
        assert!(!t.path_for("k").exists());
    }

    #[test]
    fn replacing_key_counts_once() {
        let mut t = TempCache::new(4096);
        t.cache.put("k", "a", &[1u8; 100]).unwrap();
        t.cache.put("k", "b", &[2u8; 50]).unwrap();
        assert_eq!(t.cache.len(), 1);
        assert_eq!(t.cache.total_bytes(), 50);
        let entry = t.cache.get("k").unwrap();
        assert_eq!(entry.content_type, "b");
        assert_eq!(entry.data, vec![2u8; 50]);
    }
}
