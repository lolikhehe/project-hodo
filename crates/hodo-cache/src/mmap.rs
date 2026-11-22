//! Read-only file mapping for cache bodies.
//!
//! The workspace forbids `unsafe`, and `memmap2`'s safe `map_if_safe()` cannot
//! prove immutability on Windows (it returns `None` there), so the single
//! audited call below opts out locally with `#[allow(unsafe_code)]`.
//!
//! # Safety
//!
//! Cache entries are created once via atomic rename and never mutated in place
//! (the single-writer ownership contract documented at the crate root). The
//! returned mapping is read-only, owns the OS mapping independently of the
//! `File`, and `offset + len` is validated against the file length by the
//! caller. If the OS refuses the mapping (Windows lock/AV sharing violations),
//! the caller retries briefly and then falls back to `read()`; a later warm
//! access tries mmap again.

#![allow(unsafe_code)]

use memmap2::{Mmap, MmapOptions};
use std::fs::File;
use std::io;

/// Map `len` bytes at `offset` from `file` into a read-only owned slice.
pub(crate) fn map_read_only(file: &File, offset: u64, len: usize) -> io::Result<Mmap> {
    let mut options = MmapOptions::new();
    options.offset(offset).len(len);
    // SAFETY: see module docs — immutable single-writer cache file, read-only
    // mapping, caller-validated offset/len. memmap2 also aligns the offset on
    // Windows internally (allocation granularity), so unaligned body offsets
    // are safe here.
    unsafe { options.map(file) }
}
