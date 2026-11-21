//! The single central table of resource caps (ТЗ §1).
//!
//! Parsers, caches, brokers and DOM mutation paths must read these numbers
//! instead of redefining their own. Every cap exists either to bound RAM or to
//! bound recursion: a Rust stack overflow aborts the whole browser rather than
//! panicking, so depth caps are mandatory, not advisory.

/// 2 MiB per network/disk resource — bounds allocation before decompression and matches the existing `hodo-net` hard cap.
pub const MAX_RESOURCE_BYTES: usize = 2 * 1024 * 1024;
/// 2 MiB per extension manifest — an untrusted JSON blob must never exhaust browser RAM.
pub const MAX_MANIFEST_BYTES: usize = 2 * 1024 * 1024;
/// 2 MiB per `_locales/<locale>/messages.json` — same untrusted-file bound as the manifest.
pub const MAX_LOCALE_BYTES: usize = 2 * 1024 * 1024;
/// One million arena nodes — bounds DOM RAM and keeps an opaque `NodeId` slot inside 20 bits.
pub const MAX_DOM_NODES: usize = 1_000_000;
/// 64 KiB per attribute value — unbounded attributes feed the arena directly and are a cheap memory-exhaustion vector; the whole document still rides `MAX_RESOURCE_BYTES`.
pub const MAX_ATTR_LEN: usize = 64 * 1024;
/// 100 000 CSS rules — caps rule-index memory and defeats stylesheet rule-count bombs.
pub const MAX_CSS_RULES: usize = 100_000;
/// 512 levels of DOM/layout/JSON nesting — stack overflow aborts (not panics), so every recursive pass needs a hard depth bound.
pub const MAX_NESTING_DEPTH: usize = 512;
/// 100 active timers per JS context — bounds event-loop queue RAM and runaway `setInterval` chains.
pub const MAX_ACTIVE_TIMERS: usize = 100;
/// 1 MiB per cross-context message — bounds IPC frame allocation before deserialization.
pub const MAX_MSG_BYTES: usize = 1024 * 1024;
/// 64 levels of nested message values — bounds recursive JSON validation, where a stack overflow would abort the host.
pub const MAX_MSG_DEPTH: usize = 64;
/// 16 live extension contexts — caps worker/JS RAM and broker bookkeeping for untrusted extensions.
pub const MAX_EXT_CONTEXTS: usize = 16;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_limit_is_nonzero() {
        let limits = [
            MAX_RESOURCE_BYTES,
            MAX_MANIFEST_BYTES,
            MAX_LOCALE_BYTES,
            MAX_DOM_NODES,
            MAX_ATTR_LEN,
            MAX_CSS_RULES,
            MAX_NESTING_DEPTH,
            MAX_ACTIVE_TIMERS,
            MAX_MSG_BYTES,
            MAX_MSG_DEPTH,
            MAX_EXT_CONTEXTS,
        ];
        for (i, value) in limits.into_iter().enumerate() {
            assert_ne!(value, 0, "limit #{i} must be non-zero");
        }
    }

    #[test]
    fn resource_caps_are_consistent() {
        assert_eq!(MAX_RESOURCE_BYTES, 2 * 1024 * 1024);
        assert_eq!(MAX_MANIFEST_BYTES, MAX_RESOURCE_BYTES);
        assert_eq!(MAX_LOCALE_BYTES, MAX_RESOURCE_BYTES);
        const { assert!(MAX_MSG_BYTES <= MAX_RESOURCE_BYTES) };
        // Attribute values ride the document's resource cap; the per-value cap is
        // deliberately much smaller because attributes feed the arena directly.
        const { assert!(MAX_ATTR_LEN <= MAX_RESOURCE_BYTES) };
        const { assert!(MAX_ATTR_LEN < 1024 * 1024) };
    }

    #[test]
    fn depth_caps_fit_the_stack_and_the_slot_encoding() {
        // 256 bytes is a deliberately conservative per-frame budget; hodo-dom
        // checks the real size_of::<Node>() against the same 64 MiB ceiling.
        const { assert!(MAX_NESTING_DEPTH * 256 < 64 * 1024 * 1024) };
        const { assert!(MAX_MSG_DEPTH < MAX_NESTING_DEPTH) };
        // 20 bits of a u32 NodeId are reserved for the arena slot.
        const { assert!(MAX_DOM_NODES < 1 << 20) };
        const { assert!(MAX_CSS_RULES >= MAX_DOM_NODES / 10) };
    }
}
