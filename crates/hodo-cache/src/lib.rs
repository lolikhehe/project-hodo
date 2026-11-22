//! RFC 9111 private browser cache storage: tiny metadata in RAM, bodies on disk.
//!
//! # Ownership
//!
//! A cache root has a **single owner** (the browser broker). Multiple processes
//! must never write the same root — there is no file lock here, only atomic
//! rename per entry. One writer per file is guaranteed by that contract.
//!
//! # On-disk format (version 2)
//!
//! `root/cache/<first-2-hex>/<fnv64-hex>` holds:
//!
//! ```text
//! magic[4] | version u32 | meta_len u64 | body_len u64 | checksum u64 | metadata | body
//! ```
//!
//! All integers are little-endian. `checksum` is FNV-1a folded over metadata
//! and then body. Metadata is a hand-encoded sequence of length-prefixed
//! strings and maps; any malformed input makes the reader drop the entry
//! (never panic). Paths are derived **only** from the hash, so `..`, `%2e%2e`,
//! backslashes and huge URLs cannot escape the cache directory.

mod mmap;

use memmap2::Mmap;
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Default disk quota (the ТЗ allows 128–256 MiB).
pub const DEFAULT_MAX_TOTAL_BYTES: u64 = 256 * 1024 * 1024;
/// Hard cap on an entry's **uncompressed** body (matches hodo-net's MAX_BYTES).
pub const MAX_ENTRY_BYTES: u64 = 2 * 1024 * 1024;

const MAGIC: &[u8; 4] = b"HDC2";
const VERSION: u32 = 2;
const HEADER_LEN: usize = 32;
/// Marker for an evicted entry whose unlink was deferred (live mmap on Windows).
const DEAD_SUFFIX: &str = ".dead";
/// Temp files created by streaming/atomic writes are reaped on the next open.
const TEMP_MARK: &str = ".tmp-";
const MAX_STRING: usize = 64 * 1024;
const MAX_MAP_ITEMS: usize = 1024;
/// Upper bound on the length-prefixed metadata block, so a corrupt header can
/// never make the mmap reader allocate an unbounded buffer.
const MAX_META_LEN: usize = 1024 * 1024;
/// Network chunks are streamed in 64 KiB pieces; peak RAM is one such buffer.
pub const STREAM_CHUNK_LEN: usize = 64 * 1024;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x1000_0000_01b3;

fn fnv_fold(hash: u64, bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .fold(hash, |h, b| (h ^ u64::from(*b)).wrapping_mul(FNV_PRIME))
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    fnv_fold(FNV_OFFSET, bytes)
}

/// Wall clock abstraction; all freshness math takes `&dyn Clock` so tests can
/// inject a deterministic [`FakeClock`](struct.FakeClock.html) (test-only).
pub trait Clock {
    fn now_unix(&self) -> u64;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("io: {0}")]
    Io(String),
    #[error("payload exceeds cache cap")]
    TooLarge,
    #[error("corrupt cache entry")]
    Corrupt,
}

/// (method, normalized URL without fragment, partition key).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheKey {
    pub method: String,
    pub url: String,
    pub partition: String,
}

impl CacheKey {
    pub fn new(method: &str, url: &str, partition: &str) -> Self {
        Self {
            method: method.to_string(),
            url: url.to_string(),
            partition: partition.to_string(),
        }
    }

    /// Length-prefixed FNV-1a: no ambiguity between the three components.
    pub fn hash(&self) -> u64 {
        let mut h = FNV_OFFSET;
        for part in [&self.method, &self.url, &self.partition] {
            h = fnv_fold(h, &(part.len() as u64).to_le_bytes());
            h = fnv_fold(h, part.as_bytes());
        }
        h
    }
}

/// Stored representation of a cacheable response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    /// Normalized URL (no fragment) this entry is stored under.
    pub url: String,
    pub content_type: String,
    pub data: Vec<u8>,
    pub status_code: u16,
    /// Absolute Unix time at which the entry becomes stale; `None` = never fresh.
    pub expires_at: Option<u64>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    /// Request header values selected by the response's `Vary` header.
    pub vary: BTreeMap<String, String>,
    /// Response headers (minus `Set-Cookie`), kept for 304 merge and redirects.
    pub headers: BTreeMap<String, String>,
    pub partition_key: String,
    pub request_method: String,
    pub stored_at: u64,
}

impl CacheEntry {
    pub fn is_fresh(&self, clock: &dyn Clock) -> bool {
        self.expires_at.is_some_and(|t| clock.now_unix() < t)
    }

    /// RFC 9111 §4.3.4: merge 304 headers into the stored response. Fields
    /// absent from the 304 keep their stored values; the body is untouched.
    /// A `Vary` on the 304 replaces the stored Vary selection, so a changed
    /// `Vary` cannot keep false-hitting on an old selection.
    pub fn merge_headers(
        &mut self,
        headers: &BTreeMap<String, String>,
        request_headers: &BTreeMap<String, String>,
        clock: &dyn Clock,
    ) {
        for (name, value) in headers {
            if name == "content-length" || name == "set-cookie" {
                continue;
            }
            self.headers.insert(name.clone(), value.clone());
            match name.as_str() {
                "content-type" => self.content_type = value.clone(),
                "etag" => self.etag = parse_quoted_etag(value),
                "last-modified" => self.last_modified = Some(value.clone()),
                _ => {}
            }
        }
        if let Some(vary) = headers.get("vary") {
            self.vary = vary_selection(vary, request_headers);
        }
        self.stored_at = clock.now_unix();
    }
}

/// Recompute the stored Vary selection from a (possibly new) `Vary` header and
/// the headers of the revalidation request. An unusable selection (`*`) is
/// stored as a never-matching sentinel so it becomes a conservative miss.
fn vary_selection(
    vary: &str,
    request_headers: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    vary.split(',')
        .map(|token| token.trim().to_ascii_lowercase())
        .filter(|name| !name.is_empty())
        .map(|name| {
            let value = request_headers.get(&name).cloned().unwrap_or_default();
            (name, value)
        })
        .collect()
}

/// Minimal ETag sanity check used when merging a 304. Weak (`W/"x"`) accepted;
/// malformed values become `None` instead of being replayed as validators.
fn parse_quoted_etag(value: &str) -> Option<String> {
    let v = value.trim();
    let opaque = v.strip_prefix("W/").unwrap_or(v);
    if opaque.len() >= 2 && opaque.starts_with('"') && opaque.ends_with('"') {
        Some(v.to_string())
    } else {
        None
    }
}

/// Body bytes for a warm cache hit: a read-only mmap when the OS allows it, an
/// owned `Vec` when it does not. Borrow as `&[u8]` — no full-file copy.
pub enum MappedBody {
    Mapped(Mmap),
    Owned(Vec<u8>),
}

impl MappedBody {
    /// The body bytes, with a lifetime tied to this mapping/allocation.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Mapped(map) => map,
            Self::Owned(bytes) => bytes,
        }
    }

    /// `true` when the zero-copy mmap path was used.
    pub fn is_mapped(&self) -> bool {
        matches!(self, Self::Mapped(_))
    }

    fn into_vec(self) -> Vec<u8> {
        match self {
            Self::Mapped(map) => map.to_vec(),
            Self::Owned(bytes) => bytes,
        }
    }
}

/// Metadata plus a zero-copy body view for a warm cache hit.
pub struct MappedEntry {
    /// Stored metadata. `data` is intentionally empty; the body lives in `body`.
    pub meta: CacheEntry,
    body: MappedBody,
}

impl MappedEntry {
    pub fn metadata(&self) -> &CacheEntry {
        &self.meta
    }

    /// The body bytes — the `mapped_body(handle)` view used by warm loaders.
    pub fn mapped_body(&self) -> &[u8] {
        self.body.as_bytes()
    }

    pub fn is_mapped(&self) -> bool {
        self.body.is_mapped()
    }

    pub fn is_fresh(&self, clock: &dyn Clock) -> bool {
        self.meta.is_fresh(clock)
    }

    /// Materialize a full `CacheEntry` (used only by 304 revalidation, which
    /// writes the entry back to disk).
    pub fn into_owned(self) -> CacheEntry {
        let mut meta = self.meta;
        meta.data = self.body.into_vec();
        meta
    }
}

/// Test hook: force every `get_mapped` down the read() fallback path. This
/// simulates the Windows lock/AV case deterministically.
#[doc(hidden)]
pub fn set_forced_mmap_fallback(forced: bool) {
    FORCED_MMAP_FALLBACK.store(forced, Ordering::Relaxed);
}

static FORCED_MMAP_FALLBACK: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub bytes: u64,
    pub evictions: u64,
}

static GLOBAL_HITS: AtomicU64 = AtomicU64::new(0);
static GLOBAL_MISSES: AtomicU64 = AtomicU64::new(0);
static GLOBAL_BYTES: AtomicU64 = AtomicU64::new(0);
static GLOBAL_EVICTIONS: AtomicU64 = AtomicU64::new(0);

/// Process-wide counters for `about:hodo` (the shell hook-up is a later sweep).
pub fn cache_stats() -> CacheStats {
    CacheStats {
        hits: GLOBAL_HITS.load(Ordering::Relaxed),
        misses: GLOBAL_MISSES.load(Ordering::Relaxed),
        bytes: GLOBAL_BYTES.load(Ordering::Relaxed),
        evictions: GLOBAL_EVICTIONS.load(Ordering::Relaxed),
    }
}

fn bump(counter: &AtomicU64) {
    counter.fetch_add(1, Ordering::Relaxed);
}

struct EntryMeta {
    hash: u64,
    key: CacheKey,
    size: u64,
    checksum: u64,
    last_used: u64,
}

pub struct DiskCache {
    root: PathBuf,
    max_total_bytes: u64,
    index: Vec<EntryMeta>,
    total: u64,
    tick: u64,
    stats: CacheStats,
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

impl DiskCache {
    /// Opens (or creates) a cache under `root` and rebuilds the index by
    /// scanning `<root>/cache/*/*`. Corrupt files are discarded on sight.
    pub fn open(root: &Path, max_total_bytes: u64) -> Result<Self, CacheError> {
        let cache_dir = root.join("cache");
        fs::create_dir_all(&cache_dir).map_err(|e| CacheError::Io(e.to_string()))?;
        // Only the current user may read/write the cache on Unix. On Windows
        // the directory inherits the user-profile ACL (no unsafe ACL calls).
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&cache_dir, fs::Permissions::from_mode(0o700));
        }

        let mut cache = Self {
            root: root.to_path_buf(),
            max_total_bytes,
            index: Vec::new(),
            total: 0,
            tick: 0,
            stats: CacheStats::default(),
        };
        cache.rebuild_index()?;
        cache.enforce_quota();
        cache.stats.bytes = cache.total;
        Ok(cache)
    }

    fn entry_path(&self, hash: u64) -> PathBuf {
        let hex = format!("{hash:016x}");
        self.root.join("cache").join(&hex[..2]).join(&hex)
    }

    fn rebuild_index(&mut self) -> Result<(), CacheError> {
        let cache_dir = self.root.join("cache");
        let shards = fs::read_dir(&cache_dir).map_err(|e| CacheError::Io(e.to_string()))?;
        for shard in shards.flatten() {
            if !shard.path().is_dir() {
                continue;
            }
            // Reap leftovers from crashes and from evictions deferred on
            // Windows (a live mmap can keep the file open past unlink). Do this
            // before indexing so neither a temp nor a tombstoned file is
            // re-admitted.
            let mut reaped: HashSet<String> = HashSet::new();
            let files = match fs::read_dir(shard.path()) {
                Ok(f) => f,
                Err(_) => continue,
            };
            for file in files.flatten() {
                let name = file.file_name().to_string_lossy().into_owned();
                if let Some(hex) = name.strip_suffix(DEAD_SUFFIX) {
                    reaped.insert(hex.to_string());
                }
                if name.contains(TEMP_MARK) {
                    let path = file.path();
                    let stale = path
                        .metadata()
                        .and_then(|m| m.modified())
                        .map(|t| {
                            t.elapsed()
                                .is_ok_and(|age| age >= Duration::from_secs(24 * 60 * 60))
                        })
                        .unwrap_or(true);
                    let hex_name = name.split(TEMP_MARK).next().unwrap_or("");
                    let target = path.with_file_name(hex_name);
                    let unlinked = !target.exists();
                    if stale || unlinked {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
            for hex in &reaped {
                let _ = fs::remove_file(shard.path().join(hex));
            }

            let files = match fs::read_dir(shard.path()) {
                Ok(f) => f,
                Err(_) => continue,
            };
            for file in files.flatten() {
                let path = file.path();
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if name.len() != 16 || !name.bytes().all(|b| b.is_ascii_hexdigit()) {
                    continue; // temp files and foreign files are never indexed
                }
                if reaped.contains(name) {
                    continue; // deferred unlink not yet possible (file still mapped)
                }
                let Some((entry, checksum)) = read_entry_file(&path) else {
                    let _ = fs::remove_file(&path);
                    continue;
                };
                let hash = u64::from_str_radix(name, 16).unwrap_or(0);
                self.index.push(EntryMeta {
                    hash,
                    key: CacheKey::new(&entry.request_method, &entry.url, &entry.partition_key),
                    size: entry.data.len() as u64,
                    checksum,
                    last_used: entry.stored_at,
                });
                self.total = self.total.saturating_add(entry.data.len() as u64);
                self.tick = self.tick.max(entry.stored_at);
            }
            // Drop markers only once their target is really gone; otherwise
            // keep them so the next open still suppresses re-admission.
            for hex in &reaped {
                if !shard.path().join(hex).exists() {
                    let _ = fs::remove_file(shard.path().join(format!("{hex}{DEAD_SUFFIX}")));
                }
            }
        }
        Ok(())
    }

    fn forget_at(&mut self, pos: usize) {
        let meta = self.index.swap_remove(pos);
        self.total = self.total.saturating_sub(meta.size);
    }

    fn remove_at(&mut self, pos: usize) {
        let hash = self.index[pos].hash;
        self.forget_at(pos);
        let path = self.entry_path(hash);
        if fs::remove_file(&path).is_err() {
            // Windows: a live read-only mmap may hold the file open, so unlink
            // fails with a sharing violation. Tombstone it; the next open()
            // reaps it instead of re-admitting it from the index rebuild.
            let _ = fs::write(path.with_extension("dead"), b"");
        }
    }

    fn evict_one(&mut self) {
        let Some(pos) = self
            .index
            .iter()
            .enumerate()
            .min_by_key(|(_, e)| e.last_used)
            .map(|(pos, _)| pos)
        else {
            return;
        };
        self.remove_at(pos);
        self.stats.evictions += 1;
        bump(&GLOBAL_EVICTIONS);
    }

    fn enforce_quota(&mut self) {
        while self.total > self.max_total_bytes {
            self.evict_one();
        }
    }

    /// Stores an entry, replacing any previous value for the same key.
    pub fn put(&mut self, key: &CacheKey, entry: &CacheEntry) -> Result<(), CacheError> {
        let size = entry.data.len() as u64;
        if size > MAX_ENTRY_BYTES || size > self.max_total_bytes {
            return Err(CacheError::TooLarge);
        }
        let hash = key.hash();
        let bytes = encode_entry(entry)?;
        // Write first; only touch the in-RAM index after a durable write.
        atomic_write(&self.entry_path(hash), &bytes)?;
        self.install_index(key, hash, size, fnv1a_64(&bytes[HEADER_LEN..]));
        Ok(())
    }

    /// Shared index maintenance for `put` and the streaming commit path.
    fn install_index(&mut self, key: &CacheKey, hash: u64, size: u64, checksum: u64) {
        // Forget old metadata only: the new file already replaced it on disk.
        if let Some(pos) = self
            .index
            .iter()
            .position(|e| e.hash == hash && e.key == *key)
        {
            self.forget_at(pos);
        }
        self.tick += 1;
        if let Some(pos) = self.index.iter().position(|e| e.hash == hash) {
            // Extremely unlikely hash collision with a different key: drop it.
            self.forget_at(pos);
        }
        self.index.push(EntryMeta {
            hash,
            key: key.clone(),
            size,
            checksum,
            last_used: self.tick,
        });
        self.total = self.total.saturating_add(size);
        while self.total > self.max_total_bytes && self.index.len() > 1 {
            self.evict_one();
        }
        self.stats.bytes = self.total;
        GLOBAL_BYTES.store(self.total, Ordering::Relaxed);
    }

    /// Looks up an entry and refreshes its LRU position. A found entry counts
    /// as a hit even if it later turns out to be stale (it still saved work).
    pub fn get(&mut self, key: &CacheKey) -> Option<CacheEntry> {
        let hash = key.hash();
        let pos = self
            .index
            .iter()
            .position(|e| e.hash == hash && e.key == *key)?;
        let path = self.entry_path(hash);
        let Some((entry, checksum)) = read_entry_file(&path) else {
            self.remove_at(pos);
            self.stats.misses += 1;
            bump(&GLOBAL_MISSES);
            return None;
        };
        if entry.data.len() as u64 != self.index[pos].size || checksum != self.index[pos].checksum {
            self.remove_at(pos);
            self.stats.misses += 1;
            bump(&GLOBAL_MISSES);
            return None;
        }
        self.tick += 1;
        self.index[pos].last_used = self.tick;
        self.stats.hits += 1;
        bump(&GLOBAL_HITS);
        Some(entry)
    }

    /// Warm-hit lookup: metadata plus a zero-copy `&[u8]` body view.
    ///
    /// Unlike [`DiskCache::get`], the body is not read into a `Vec`. The
    /// read-only mapping owns its bytes; if mmap is refused (Windows lock/AV
    /// sharing violations) we retry briefly and fall back to `read()`. A
    /// later hit tries mmap again. "Read-only" plus the single-writer rename
    /// contract keeps the mapping valid for its whole owned lifetime.
    pub fn get_mapped(&mut self, key: &CacheKey) -> Option<MappedEntry> {
        let hash = key.hash();
        let pos = self
            .index
            .iter()
            .position(|e| e.hash == hash && e.key == *key)?;
        let path = self.entry_path(hash);
        let Some((meta, body, checksum)) = read_entry_mapped(&path) else {
            self.remove_at(pos);
            self.stats.misses += 1;
            bump(&GLOBAL_MISSES);
            return None;
        };
        if body.as_bytes().len() as u64 != self.index[pos].size
            || checksum != self.index[pos].checksum
        {
            self.remove_at(pos);
            self.stats.misses += 1;
            bump(&GLOBAL_MISSES);
            return None;
        }
        self.tick += 1;
        self.index[pos].last_used = self.tick;
        self.stats.hits += 1;
        bump(&GLOBAL_HITS);
        Some(MappedEntry { meta, body })
    }

    /// Begin a write-through streaming entry. The response body is fed to
    /// [`StreamWriter::write`] in chunks and committed with [`StreamWriter::commit`];
    /// only one chunk is ever resident. `entry.data` is ignored (the body is
    /// supplied by the stream); all other fields become the stored metadata.
    pub fn stream_put<'a>(
        &'a mut self,
        key: &CacheKey,
        entry: &CacheEntry,
    ) -> Result<StreamWriter<'a>, CacheError> {
        let meta = encode_metadata(entry)?;
        let target = self.entry_path(key.hash());
        let temp = temp_path_for(&target)?;
        let mut file = File::create(&temp).map_err(|e| CacheError::Io(e.to_string()))?;
        // Reserve the header; commit() seeks back and fills in the real values
        // once body_len and the rolling checksum are known.
        file.write_all(&[0u8; HEADER_LEN])
            .and_then(|()| file.write_all(&meta))
            .map_err(|e| CacheError::Io(e.to_string()))?;
        Ok(StreamWriter {
            cache: self,
            key: key.clone(),
            target,
            temp,
            file: Some(file),
            meta_len: meta.len(),
            checksum: fnv_fold(FNV_OFFSET, &meta),
            body_len: 0,
            committed: false,
        })
    }

    /// Invalidate one key (used by future non-GET / unsafe methods).
    pub fn invalidate(&mut self, key: &CacheKey) {
        let hash = key.hash();
        if let Some(pos) = self
            .index
            .iter()
            .position(|e| e.hash == hash && e.key == *key)
        {
            self.remove_at(pos);
        }
        self.stats.bytes = self.total;
        GLOBAL_BYTES.store(self.total, Ordering::Relaxed);
    }

    /// Drop every entry; returns the number of removed files.
    pub fn clear(&mut self) -> usize {
        let count = self.index.len();
        self.index.clear();
        self.total = 0;
        self.stats.bytes = 0;
        GLOBAL_BYTES.store(0, Ordering::Relaxed);
        let cache_dir = self.root.join("cache");
        if let Ok(shards) = fs::read_dir(&cache_dir) {
            for shard in shards.flatten() {
                let name = shard.file_name();
                let name = name.to_string_lossy();
                if shard.path().is_dir()
                    && name.len() == 2
                    && name.bytes().all(|b| b.is_ascii_hexdigit())
                {
                    let _ = fs::remove_dir_all(shard.path());
                }
            }
        }
        count
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

    /// Per-instance counters; [`cache_stats`] is the process-wide view.
    pub fn stats(&self) -> CacheStats {
        self.stats
    }
}

/// A cache entry being written straight from a response body to a temp file.
/// RAM peak is the caller's chunk buffer (64 KiB in hodo-net), never the body.
pub struct StreamWriter<'a> {
    cache: &'a mut DiskCache,
    key: CacheKey,
    target: PathBuf,
    temp: PathBuf,
    file: Option<File>,
    meta_len: usize,
    /// FNV-1a state after metadata, folded with each body chunk.
    checksum: u64,
    body_len: u64,
    committed: bool,
}

impl StreamWriter<'_> {
    /// Append one body chunk. Enforces the same 2 MiB / quota caps as `put`.
    pub fn write(&mut self, chunk: &[u8]) -> Result<(), CacheError> {
        let next = self.body_len.saturating_add(chunk.len() as u64);
        if next > MAX_ENTRY_BYTES || next > self.cache.max_total_bytes {
            return Err(CacheError::TooLarge);
        }
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| CacheError::Io("stream already closed".into()))?;
        file.write_all(chunk)
            .map_err(|e| CacheError::Io(e.to_string()))?;
        self.body_len = next;
        self.checksum = fnv_fold(self.checksum, chunk);
        Ok(())
    }

    /// Fill in the header, sync, atomically rename into place and index the entry.
    pub fn commit(mut self) -> Result<(), CacheError> {
        let mut file = self
            .file
            .take()
            .ok_or_else(|| CacheError::Io("stream already closed".into()))?;
        let mut head = Vec::with_capacity(HEADER_LEN);
        head.extend_from_slice(MAGIC);
        head.extend_from_slice(&VERSION.to_le_bytes());
        head.extend_from_slice(&(self.meta_len as u64).to_le_bytes());
        head.extend_from_slice(&self.body_len.to_le_bytes());
        head.extend_from_slice(&self.checksum.to_le_bytes());
        file.seek(SeekFrom::Start(0))
            .and_then(|_| file.write_all(&head))
            .and_then(|()| file.sync_all())
            .map_err(|e| CacheError::Io(e.to_string()))?;
        drop(file);
        rename_temp_with_retry(&self.temp, &self.target)?;
        self.committed = true;
        self.cache
            .install_index(&self.key, self.key.hash(), self.body_len, self.checksum);
        Ok(())
    }
}

impl Drop for StreamWriter<'_> {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_file(&self.temp);
        }
    }
}

fn encode_metadata(entry: &CacheEntry) -> Result<Vec<u8>, CacheError> {
    let mut meta = Vec::new();
    put_str(&mut meta, &entry.url)?;
    put_u16(&mut meta, entry.status_code);
    put_u64(&mut meta, entry.expires_at.unwrap_or(0));
    put_u64(&mut meta, entry.stored_at);
    put_str(&mut meta, &entry.content_type)?;
    put_opt_str(&mut meta, entry.etag.as_deref())?;
    put_opt_str(&mut meta, entry.last_modified.as_deref())?;
    put_str(&mut meta, &entry.partition_key)?;
    put_str(&mut meta, &entry.request_method)?;
    put_map(&mut meta, &entry.vary)?;
    put_map(&mut meta, &entry.headers)?;
    Ok(meta)
}

fn encode_entry(entry: &CacheEntry) -> Result<Vec<u8>, CacheError> {
    let meta = encode_metadata(entry)?;
    let mut out = Vec::with_capacity(HEADER_LEN + meta.len() + entry.data.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&(meta.len() as u64).to_le_bytes());
    out.extend_from_slice(&(entry.data.len() as u64).to_le_bytes());
    let checksum = fnv_fold(fnv_fold(FNV_OFFSET, &meta), &entry.data);
    out.extend_from_slice(&checksum.to_le_bytes());
    out.extend_from_slice(&meta);
    out.extend_from_slice(&entry.data);
    Ok(out)
}

/// Decode the metadata block. The returned entry has an empty body; callers
/// attach it from the file bytes or the mmap view.
fn parse_metadata(bytes: &[u8]) -> Option<CacheEntry> {
    let mut cursor = Cursor { bytes, pos: 0 };
    let url = cursor.string()?;
    let status_code = cursor.u16()?;
    let expires_at = cursor.u64()?;
    let stored_at = cursor.u64()?;
    let content_type = cursor.string()?;
    let etag = cursor.opt_string()?;
    let last_modified = cursor.opt_string()?;
    let partition_key = cursor.string()?;
    let request_method = cursor.string()?;
    let vary = cursor.map()?;
    let headers = cursor.map()?;
    if cursor.pos != bytes.len() {
        return None;
    }
    Some(CacheEntry {
        url,
        content_type,
        data: Vec::new(),
        status_code,
        expires_at: (expires_at != 0).then_some(expires_at),
        etag,
        last_modified,
        vary,
        headers,
        partition_key,
        request_method,
        stored_at,
    })
}

fn read_entry_file(path: &Path) -> Option<(CacheEntry, u64)> {
    let bytes = fs::read(path).ok()?;
    if bytes.len() < HEADER_LEN || &bytes[..4] != MAGIC {
        return None;
    }
    if u32::from_le_bytes(bytes[4..8].try_into().ok()?) != VERSION {
        return None;
    }
    let meta_len = u64::from_le_bytes(bytes[8..16].try_into().ok()?) as usize;
    let body_len = u64::from_le_bytes(bytes[16..24].try_into().ok()?) as usize;
    let expected = u64::from_le_bytes(bytes[24..32].try_into().ok()?);
    if meta_len > bytes.len() - HEADER_LEN || meta_len > MAX_META_LEN {
        return None;
    }
    let body_start = HEADER_LEN + meta_len;
    if body_len != bytes.len().checked_sub(body_start)? {
        return None;
    }
    let meta = &bytes[HEADER_LEN..body_start];
    let body = &bytes[body_start..];
    if fnv_fold(fnv_fold(FNV_OFFSET, meta), body) != expected {
        return None;
    }
    let mut entry = parse_metadata(meta)?;
    entry.data = body.to_vec();
    Some((entry, expected))
}

/// Read a header + metadata block, then map (or, on failure, read) just the
/// body. Returns metadata, the owned body view and the verified checksum.
fn read_entry_mapped(path: &Path) -> Option<(CacheEntry, MappedBody, u64)> {
    let mut file = File::open(path).ok()?;
    let file_len = file.metadata().ok()?.len();
    if file_len < HEADER_LEN as u64 {
        return None;
    }
    let mut head = [0u8; HEADER_LEN];
    file.read_exact(&mut head).ok()?;
    if &head[..4] != MAGIC || u32::from_le_bytes(head[4..8].try_into().ok()?) != VERSION {
        return None;
    }
    let meta_len = u64::from_le_bytes(head[8..16].try_into().ok()?) as usize;
    let body_len = u64::from_le_bytes(head[16..24].try_into().ok()?) as usize;
    let expected = u64::from_le_bytes(head[24..32].try_into().ok()?);
    if meta_len > MAX_META_LEN {
        return None;
    }
    let body_start = HEADER_LEN.checked_add(meta_len)?;
    let total = body_start.checked_add(body_len)?;
    if total as u64 != file_len {
        return None;
    }

    let mut meta_bytes = vec![0u8; meta_len];
    file.seek(SeekFrom::Start(HEADER_LEN as u64)).ok()?;
    file.read_exact(&mut meta_bytes).ok()?;
    let meta = parse_metadata(&meta_bytes)?;
    let mut checksum = fnv_fold(FNV_OFFSET, &meta_bytes);

    let body = if body_len == 0 {
        MappedBody::Owned(Vec::new())
    } else if FORCED_MMAP_FALLBACK.load(Ordering::Relaxed) {
        MappedBody::Owned(read_body(&mut file, body_start, body_len)?)
    } else {
        // Windows lock/AV cases: retry briefly, then fall back to read(). The
        // next warm access tries mmap again — no permanent downgrade.
        let mut mapped = None;
        for attempt in 0..3 {
            match mmap::map_read_only(&file, body_start as u64, body_len) {
                Ok(map) => {
                    mapped = Some(map);
                    break;
                }
                Err(error) if is_lock_error(&error) && attempt < 2 => {
                    std::thread::sleep(Duration::from_millis(2 * (attempt + 1)));
                }
                Err(_) => break,
            }
        }
        match mapped {
            Some(map) => {
                checksum = fnv_fold(checksum, &map[..]);
                MappedBody::Mapped(map)
            }
            None => MappedBody::Owned(read_body(&mut file, body_start, body_len)?),
        }
    };
    if !matches!(body, MappedBody::Mapped(_)) {
        checksum = fnv_fold(checksum, body.as_bytes());
    }
    if checksum != expected {
        return None;
    }
    Some((meta, body, expected))
}

fn read_body(file: &mut File, offset: usize, len: usize) -> Option<Vec<u8>> {
    let mut bytes = vec![0u8; len];
    file.seek(SeekFrom::Start(offset as u64)).ok()?;
    file.read_exact(&mut bytes).ok()?;
    Some(bytes)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        let slice = self.bytes.get(self.pos..end)?;
        self.pos = end;
        Some(slice)
    }

    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    fn string(&mut self) -> Option<String> {
        let len = self.u32()? as usize;
        if len > MAX_STRING {
            return None;
        }
        String::from_utf8(self.take(len)?.to_vec()).ok()
    }

    fn opt_string(&mut self) -> Option<Option<String>> {
        let present = self.take(1)?[0] != 0;
        Some(if present { Some(self.string()?) } else { None })
    }

    fn map(&mut self) -> Option<BTreeMap<String, String>> {
        let count = self.u32()? as usize;
        if count > MAX_MAP_ITEMS {
            return None;
        }
        let mut map = BTreeMap::new();
        for _ in 0..count {
            map.insert(self.string()?, self.string()?);
        }
        Some(map)
    }
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_str(out: &mut Vec<u8>, value: &str) -> Result<(), CacheError> {
    if value.len() > MAX_STRING {
        return Err(CacheError::TooLarge);
    }
    out.extend_from_slice(&(value.len() as u32).to_le_bytes());
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn put_opt_str(out: &mut Vec<u8>, value: Option<&str>) -> Result<(), CacheError> {
    out.push(u8::from(value.is_some()));
    if let Some(value) = value {
        put_str(out, value)?;
    }
    Ok(())
}

fn put_map(out: &mut Vec<u8>, map: &BTreeMap<String, String>) -> Result<(), CacheError> {
    if map.len() > MAX_MAP_ITEMS {
        return Err(CacheError::TooLarge);
    }
    out.extend_from_slice(&(map.len() as u32).to_le_bytes());
    for (key, value) in map {
        put_str(out, key)?;
        put_str(out, value)?;
    }
    Ok(())
}

fn is_lock_error(error: &std::io::Error) -> bool {
    // Windows antivirus / indexers may hold a file open briefly after create.
    error.kind() == std::io::ErrorKind::PermissionDenied
        || error.raw_os_error() == Some(32) // ERROR_SHARING_VIOLATION
        || error.raw_os_error() == Some(33) // ERROR_LOCK_VIOLATION
}

fn temp_path_for(target: &Path) -> Result<PathBuf, CacheError> {
    let dir = target
        .parent()
        .ok_or_else(|| CacheError::Io("no parent".into()))?;
    fs::create_dir_all(dir).map_err(|e| CacheError::Io(e.to_string()))?;
    let name = target
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| CacheError::Io("bad name".into()))?;
    let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    Ok(dir.join(format!("{name}.tmp-{}-{n}", std::process::id())))
}

/// std::fs::rename uses MOVEFILE_REPLACE_EXISTING on Windows. Lock/AV hiccups
/// get the same bounded backoff as before; the caller discards the temp file.
fn rename_temp_with_retry(temp: &Path, target: &Path) -> Result<(), CacheError> {
    for attempt in 0..6 {
        match fs::rename(temp, target) {
            Ok(()) => return Ok(()),
            Err(error) if is_lock_error(&error) && attempt < 5 => {
                std::thread::sleep(Duration::from_millis(5 * (attempt + 1)));
            }
            Err(error) => {
                let _ = fs::remove_file(temp);
                return Err(CacheError::Io(error.to_string()));
            }
        }
    }
    Err(CacheError::Io("rename retries exhausted".into()))
}

fn atomic_write(target: &Path, bytes: &[u8]) -> Result<(), CacheError> {
    let temp = temp_path_for(target)?;
    let write = (|| -> std::io::Result<()> {
        let mut file = File::create(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()
    })();
    if let Err(error) = write {
        let _ = fs::remove_file(&temp);
        return Err(CacheError::Io(error.to_string()));
    }
    rename_temp_with_retry(&temp, target)
}

#[cfg(test)]
pub(crate) struct FakeClock {
    pub now: std::sync::Mutex<u64>,
}

#[cfg(test)]
impl FakeClock {
    pub fn new(now: u64) -> Self {
        Self {
            now: std::sync::Mutex::new(now),
        }
    }

    pub fn advance(&self, seconds: u64) {
        *self.now.lock().unwrap() += seconds;
    }
}

#[cfg(test)]
impl Clock for FakeClock {
    fn now_unix(&self) -> u64 {
        *self.now.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
    use std::time::Instant;

    static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);
    /// Serialize tests that touch the global forced-fallback flag.
    static MMAP_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct TempCache {
        root: PathBuf,
        cache: DiskCache,
    }

    impl TempCache {
        fn new(max_total_bytes: u64) -> Self {
            let n = NEXT_DIR.fetch_add(1, AtomicOrdering::Relaxed);
            let root = std::env::temp_dir().join(format!("hodo-cache-{}-{n}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(&root).unwrap();
            let cache = DiskCache::open(&root, max_total_bytes).unwrap();
            Self { root, cache }
        }

        fn path_for(&self, key: &CacheKey) -> PathBuf {
            self.cache.entry_path(key.hash())
        }
    }

    impl Drop for TempCache {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn key(url: &str) -> CacheKey {
        CacheKey::new("GET", url, "")
    }

    fn entry(body: &[u8], content_type: &str) -> CacheEntry {
        CacheEntry {
            url: String::new(),
            content_type: content_type.into(),
            data: body.to_vec(),
            status_code: 200,
            expires_at: None,
            etag: None,
            last_modified: None,
            vary: BTreeMap::new(),
            headers: BTreeMap::new(),
            partition_key: String::new(),
            request_method: "GET".into(),
            stored_at: 0,
        }
    }

    #[test]
    fn roundtrip_preserves_metadata() {
        let mut t = TempCache::new(4096);
        let k = key("https://x/a.css");
        let mut e = entry(b"body{}", "text/css");
        e.url = k.url.clone();
        e.status_code = 203;
        e.expires_at = Some(1_700_000_100);
        e.etag = Some("W/\"abc\"".into());
        e.last_modified = Some("Sun, 06 Nov 1994 08:49:37 GMT".into());
        e.vary.insert("accept-encoding".into(), "gzip".into());
        e.headers.insert("location".into(), "/next".into());
        e.partition_key = "https://site.example".into();
        e.stored_at = 42;
        t.cache.put(&k, &e).unwrap();

        let got = t.cache.get(&k).unwrap();
        assert_eq!(got, e);
        assert_eq!(got.data, b"body{}");
        assert!(got.is_fresh(&FakeClock::new(1_700_000_000)));
        assert!(!got.is_fresh(&FakeClock::new(1_700_000_100)));
        let stats = t.cache.stats();
        assert_eq!((stats.hits, stats.misses, stats.evictions), (1, 0, 0));
    }

    #[test]
    fn sharded_paths_and_rebuild_on_reopen() {
        let mut t = TempCache::new(4096);
        let k = key("https://x/a.css");
        let mut e = entry(b"body{}", "text/css");
        e.url = k.url.clone();
        t.cache.put(&k, &e).unwrap();
        let path = t.path_for(&k);
        let hash_hex = format!("{:016x}", k.hash());
        assert_eq!(
            path,
            t.root.join("cache").join(&hash_hex[..2]).join(&hash_hex)
        );
        assert!(path.is_file());
        assert!(t.cache.total_bytes() == 6);

        let mut reopened = DiskCache::open(&t.root, 4096).unwrap();
        assert_eq!(reopened.len(), 1);
        assert_eq!(
            reopened.get(&key("https://x/a.css")).unwrap().data,
            b"body{}"
        );
    }

    #[test]
    fn lru_eviction_order_and_stats() {
        let mut t = TempCache::new(1024);
        for (i, url) in ["a", "b", "c", "d"].iter().enumerate() {
            t.cache
                .put(&key(url), &entry(&vec![b'a' + i as u8; 300], "text/plain"))
                .unwrap();
        }
        assert_eq!(t.cache.len(), 3);
        assert!(t.cache.get(&key("a")).is_none());
        assert!(t.cache.get(&key("b")).is_some());
        t.cache
            .put(&key("e"), &entry(&[b'e'; 300], "text/plain"))
            .unwrap();
        assert!(t.cache.get(&key("c")).is_none());
        assert!(t.cache.get(&key("b")).is_some());
        assert_eq!(t.cache.stats().evictions, 2);
        assert_eq!(cache_stats().evictions, 2);
    }

    #[test]
    fn too_large_entry_rejected() {
        let mut t = TempCache::new(8 * 1024 * 1024);
        assert!(matches!(
            t.cache.put(
                &key("k"),
                &entry(&[0u8; (MAX_ENTRY_BYTES + 1) as usize], "text/plain")
            ),
            Err(CacheError::TooLarge)
        ));
        assert!(t.cache.is_empty());
        t.cache
            .put(
                &key("k"),
                &entry(&[1u8; MAX_ENTRY_BYTES as usize], "text/plain"),
            )
            .unwrap();
        assert_eq!(t.cache.total_bytes(), MAX_ENTRY_BYTES);

        // A quota smaller than the entry rejects it too.
        let mut small = TempCache::new(1024);
        assert!(matches!(
            small
                .cache
                .put(&key("k"), &entry(&[1u8; 2048], "text/plain")),
            Err(CacheError::TooLarge)
        ));
    }

    #[test]
    fn corrupt_and_truncated_files_are_dropped_without_panic() {
        let mut t = TempCache::new(4096);
        let k = key("k");
        let path = t.path_for(&k);
        // Each round: write a valid file, corrupt it, read must prune it.
        for case in 0..5 {
            t.cache.put(&k, &entry(b"hello", "text/plain")).unwrap();
            let mut bytes = fs::read(&path).unwrap();
            match case {
                0 => bytes.truncate(1), // truncated header
                1 => bytes[0] = b'X',   // bad magic
                2 => bytes[5] = 9,      // bad version
                3 => bytes[30] ^= 0xff, // bad checksum
                _ => bytes[16] = 0xff,  // bogus body length
            }
            fs::write(&path, &bytes).unwrap();
            assert!(t.cache.get(&k).is_none(), "case {case}");
            assert!(t.cache.is_empty());
            assert!(!path.exists());
        }
    }

    #[test]
    fn hostile_urls_never_escape_hash_only_paths() {
        let mut t = TempCache::new(4096);
        let long = format!("https://x/{}", "a".repeat(10_000));
        for url in [
            "https://x/../../etc/passwd".to_string(),
            "https://x/%2e%2e/%2e%2e/secret".to_string(),
            "https://x\\..\\..\\windows\\system32".to_string(),
            long,
        ] {
            let k = key(&url);
            t.cache.put(&k, &entry(b"ok", "text/plain")).unwrap();
            let path = t.path_for(&k);
            let rel = path.strip_prefix(t.root.join("cache")).unwrap();
            let parts: Vec<_> = rel.components().collect();
            assert_eq!(parts.len(), 2, "path must be <hex2>/<hex16>: {rel:?}");
            assert_eq!(parts[0].as_os_str().to_string_lossy().len(), 2);
            assert_eq!(parts[1].as_os_str().to_string_lossy().len(), 16);
            assert!(path.is_file());
        }
        assert_eq!(t.cache.len(), 4);
    }

    #[test]
    fn replacing_key_keeps_the_new_file() {
        let mut t = TempCache::new(4096);
        let k = key("k");
        t.cache.put(&k, &entry(b"long-old-body", "a")).unwrap();
        t.cache.put(&k, &entry(b"new", "b")).unwrap();
        assert_eq!(t.cache.len(), 1);
        assert_eq!(t.cache.total_bytes(), 3);
        let got = t.cache.get(&k).unwrap();
        assert_eq!(got.data, b"new");
        assert_eq!(got.content_type, "b");
        assert!(t.path_for(&k).exists());
    }

    #[test]
    fn clear_and_invalidate() {
        let mut t = TempCache::new(4096);
        let a = key("a");
        let b = key("b");
        t.cache.put(&a, &entry(b"a", "text/plain")).unwrap();
        t.cache.put(&b, &entry(b"bb", "text/plain")).unwrap();
        t.cache.invalidate(&a);
        assert!(t.cache.get(&a).is_none());
        assert!(t.cache.get(&b).is_some());
        assert_eq!(t.cache.clear(), 1);
        assert!(t.cache.is_empty());
        assert_eq!(t.cache.total_bytes(), 0);
    }

    #[test]
    fn fake_clock_controls_freshness() {
        let clock = FakeClock::new(100);
        let mut e = entry(b"x", "text/plain");
        e.expires_at = Some(110);
        assert!(e.is_fresh(&clock));
        clock.advance(11);
        assert!(!e.is_fresh(&clock));
    }

    #[test]
    fn merge_304_keeps_body_and_updates_metadata() {
        let clock = FakeClock::new(1_000);
        let mut e = entry(b"body", "text/plain");
        e.etag = Some("\"v1\"".into());
        e.expires_at = Some(1_100);
        let mut headers = BTreeMap::new();
        headers.insert("etag".into(), "\"v2\"".into());
        headers.insert("cache-control".into(), "max-age=60".into());
        headers.insert("set-cookie".into(), "secret=1".into());
        e.merge_headers(&headers, &BTreeMap::new(), &clock);
        assert_eq!(e.data, b"body");
        assert_eq!(e.etag.as_deref(), Some("\"v2\""));
        assert!(!e.headers.contains_key("set-cookie"));
        assert_eq!(e.stored_at, 1_000);
    }

    #[test]
    fn merge_304_updates_the_vary_selection() {
        let clock = FakeClock::new(1_000);
        let mut e = entry(b"body", "text/plain");
        e.vary.insert("accept-encoding".into(), "gzip".into());
        let mut headers = BTreeMap::new();
        headers.insert("vary".into(), "Accept-Language".into());
        let mut request = BTreeMap::new();
        request.insert("accept-language".into(), "en-GB".into());
        request.insert("accept-encoding".into(), "br".into());
        e.merge_headers(&headers, &request, &clock);
        assert_eq!(
            e.vary.get("accept-language").map(String::as_str),
            Some("en-GB")
        );
        assert!(
            !e.vary.contains_key("accept-encoding"),
            "stale Vary selection must be replaced"
        );
    }

    #[test]
    fn malformed_headers_never_panic_reader() {
        let mut t = TempCache::new(4096);
        let k = key("k");
        t.cache.put(&k, &entry(b"hello", "text/plain")).unwrap();
        let path = t.path_for(&k);
        for bytes in [
            Vec::new(),
            vec![0; 31],
            vec![0; 32],
            b"HDC2".to_vec(),
            vec![0xff; 200],
        ] {
            fs::write(&path, &bytes).unwrap();
            let _ = read_entry_file(&path);
        }
        assert!(t.cache.get(&k).is_none());
    }

    // --- mmap warm reads ---------------------------------------------------

    #[test]
    fn mapped_body_roundtrip_is_zero_copy() {
        let _guard = MMAP_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut t = TempCache::new(8 * 1024 * 1024);
        let k = key("https://x/big.html");
        let body = vec![b'x'; 512 * 1024];
        let mut e = entry(&body, "text/html");
        e.url = k.url.clone();
        t.cache.put(&k, &e).unwrap();

        let mapped = t.cache.get_mapped(&k).unwrap();
        assert!(
            mapped.is_mapped(),
            "expected the mmap path on this platform"
        );
        assert_eq!(mapped.mapped_body(), body);
        assert_eq!(mapped.metadata().content_type, "text/html");
        assert!(
            mapped.metadata().data.is_empty(),
            "metadata must not carry the body"
        );
        assert_eq!(mapped.into_owned().data, body);
        assert_eq!(t.cache.stats().hits, 1);
    }

    #[test]
    fn forced_mmap_failure_falls_back_to_read() {
        let _guard = MMAP_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut t = TempCache::new(8 * 1024 * 1024);
        let k = key("https://x/fallback");
        let body = vec![b'y'; 128 * 1024];
        t.cache.put(&k, &entry(&body, "text/plain")).unwrap();

        set_forced_mmap_fallback(true);
        let mapped = t.cache.get_mapped(&k).unwrap();
        assert!(!mapped.is_mapped(), "forced failure must use read()");
        assert_eq!(mapped.mapped_body(), body);
        set_forced_mmap_fallback(false);

        // A later access retries mmap and succeeds again.
        assert!(t.cache.get_mapped(&k).unwrap().is_mapped());
    }

    #[test]
    fn eviction_with_live_mapping_reaps_orphan_on_open() {
        let _guard = MMAP_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut t = TempCache::new(8 * 1024 * 1024);
        let k = key("https://x/mapped-evict");
        t.cache
            .put(&k, &entry(&vec![b'm'; 4096], "text/plain"))
            .unwrap();
        let path = t.path_for(&k);
        let mapped = t.cache.get_mapped(&k).unwrap();
        assert!(mapped.is_mapped());
        t.cache.invalidate(&k);
        // Windows may keep the file alive behind the mapping (tombstoned);
        // Unix unlinks it immediately. Either way reopen must see no orphan.
        drop(mapped);
        let reopened = DiskCache::open(&t.root, 8 * 1024 * 1024).unwrap();
        assert_eq!(reopened.len(), 0);
        assert!(!path.exists(), "evicted file must not survive reopen");
    }

    #[test]
    fn crash_left_temp_files_are_reaped_on_open() {
        let t = TempCache::new(4096);
        let k = key("https://x/temp");
        let shard = t
            .root
            .join("cache")
            .join(&format!("{:016x}", k.hash())[..2]);
        fs::create_dir_all(&shard).unwrap();
        let temp = shard.join(format!("{:016x}.tmp-{}-999", k.hash(), std::process::id()));
        fs::write(&temp, b"partial").unwrap();
        assert!(temp.exists());
        let reopened = DiskCache::open(&t.root, 4096).unwrap();
        assert_eq!(reopened.len(), 0);
        assert!(!temp.exists(), "unlinked temp must be reaped on open");
        drop(reopened);
    }

    // --- streaming write-through ------------------------------------------

    #[test]
    fn stream_writer_commits_a_2mib_body_in_64k_chunks() {
        let _guard = MMAP_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut t = TempCache::new(8 * 1024 * 1024);
        let k = key("https://x/stream.bin");
        let body = vec![0x5a; 2 * 1024 * 1024];
        let mut e = entry(&[], "application/octet-stream");
        e.url = k.url.clone();

        let mut writer = t.cache.stream_put(&k, &e).unwrap();
        for chunk in body.chunks(STREAM_CHUNK_LEN) {
            assert!(chunk.len() <= STREAM_CHUNK_LEN);
            writer.write(chunk).unwrap();
        }
        writer.commit().unwrap();

        assert_eq!(t.cache.len(), 1);
        assert_eq!(t.cache.total_bytes(), MAX_ENTRY_BYTES);
        let got = t.cache.get(&k).unwrap();
        assert_eq!(got.data, body);
        assert_eq!(got.content_type, "application/octet-stream");
        // The committed file is also readable through the mmap warm path.
        assert_eq!(t.cache.get_mapped(&k).unwrap().mapped_body(), body);
    }

    #[test]
    fn stream_writer_over_cap_is_rejected_and_temp_removed() {
        let mut t = TempCache::new(8 * 1024 * 1024);
        let k = key("https://x/too-big");
        let mut writer = t.cache.stream_put(&k, &entry(&[], "text/plain")).unwrap();
        assert!(matches!(
            writer.write(&[0u8; (MAX_ENTRY_BYTES + 1) as usize]),
            Err(CacheError::TooLarge)
        ));
        drop(writer);
        assert!(t.cache.is_empty());
        assert_eq!(
            fs::read_dir(
                t.root
                    .join("cache")
                    .join(&format!("{:016x}", k.hash())[..2])
            )
            .unwrap()
            .count(),
            0
        );
    }

    // --- warm-hit perf: big-doc corpus pattern -----------------------------

    #[test]
    fn warm_hit_mmap_vs_read_perf() {
        let _guard = MMAP_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut t = TempCache::new(8 * 1024 * 1024);
        let k = key("https://x/big-doc");
        let body = vec![b'a'; 2 * 1024 * 1024];
        t.cache.put(&k, &entry(&body, "text/html")).unwrap();
        // Warm the OS page cache and the mapping tables first.
        for _ in 0..20 {
            let _ = t.cache.get(&k);
            let _ = t.cache.get_mapped(&k);
        }
        const N: u32 = 200;
        let start = Instant::now();
        for _ in 0..N {
            let e = t.cache.get(&k).unwrap();
            std::hint::black_box(std::str::from_utf8(&e.data).unwrap().len());
        }
        let read_us = start.elapsed().as_secs_f64() * 1_000_000.0 / f64::from(N);
        let start = Instant::now();
        for _ in 0..N {
            let e = t.cache.get_mapped(&k).unwrap();
            std::hint::black_box(std::str::from_utf8(e.mapped_body()).unwrap().len());
        }
        let mmap_us = start.elapsed().as_secs_f64() * 1_000_000.0 / f64::from(N);
        eprintln!(
            "warm 2 MiB doc: read {read_us:.1} us/hit vs mmap {mmap_us:.1} us/hit ({:.2}x)",
            read_us / mmap_us
        );
    }
}
