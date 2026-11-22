//! Ad/tracker filtering with a deliberately small filter grammar.
//!
//! Supported syntax (the frozen subset):
//! - `||host^` host anchors, including `||host/path^` (host + path substring)
//! - plain lowercase substrings (`^` is a separator hint and is stripped)
//! - `@@` exception rules, evaluated **before** blocking rules
//! - `$domain=a|b` (positive only) and `$third-party` modifiers
//!
//! Everything else (regex `/.../`, wildcards, element hiding `##`, resource-type
//! modifiers such as `$script`, negated `$~domain`, ...) is skipped and counted.
//! Real EasyList/EasyPrivacy files are vendored under `assets/filters` and parsed
//! in full at startup; an unrecognised line never fails the whole list.

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

const MAX_LINES: usize = 200_000;
const MAX_RULE_BYTES: usize = 4096;
const BLOCKED_SAMPLE_LEN: usize = 50;

/// A tiny built-in safety net. The vendored lists are the primary source; this
/// only guarantees a few universally-agreed ad hosts survive modifier skips
/// (e.g. EasyList's `||doubleclick.net^$popup` has an out-of-grammar modifier).
const BUILTIN_FILTERS: &str = "\
||doubleclick.net^\n\
||googletagmanager.com^\n\
";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockDecision {
    Allow,
    Block {
        rule: String,
    },
    /// Advisory hit (a caller in warn-only mode can surface it without blocking).
    Warn {
        rule: String,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParseStats {
    pub parsed: usize,
    pub skipped: usize,
    pub comments: usize,
}

#[derive(Debug, Clone)]
struct RuleMeta {
    text: String,
    /// `None` = every site; `Some` = only these registrable domains.
    domains: Option<Vec<String>>,
    third_party_only: bool,
}

#[derive(Debug, Clone)]
struct Anchor {
    host: String,
    /// path prefix anchored directly after the matched host
    path: Option<String>,
    /// whether the rule ended in `^` (separator boundary after the token)
    trailing_boundary: bool,
    meta: RuleMeta,
}

/// One plain-substring pattern. `^` is a separator boundary at the indicated
/// edge rather than a literal character.
#[derive(Debug, Clone)]
struct PatternMeta {
    leading_boundary: bool,
    trailing_boundary: bool,
    meta: RuleMeta,
}

/// A compiled, immutable filter list.
#[derive(Debug)]
pub struct FilterList {
    block_ac: Option<AhoCorasick>,
    /// AC pattern index -> rules sharing that literal (boundary flags differ).
    block_ac_meta: Vec<Vec<PatternMeta>>,
    exception_ac: Option<AhoCorasick>,
    exception_ac_meta: Vec<Vec<PatternMeta>>,
    block_anchors: HashMap<String, Vec<Anchor>>,
    exception_anchors: HashMap<String, Vec<Anchor>>,
    stats: ParseStats,
    build_millis: u128,
}

impl Default for FilterList {
    fn default() -> Self {
        Self::parse("")
    }
}

impl FilterList {
    /// Parse and compile a filter list. Invalid lines are counted, not fatal.
    pub fn parse(text: &str) -> Self {
        let started = Instant::now();
        let mut stats = ParseStats::default();
        let mut block_anchors: HashMap<String, Vec<Anchor>> = HashMap::new();
        let mut exception_anchors: HashMap<String, Vec<Anchor>> = HashMap::new();
        let mut block_literals: Vec<String> = Vec::new();
        let mut block_meta: Vec<Vec<PatternMeta>> = Vec::new();
        let mut block_index: HashMap<String, usize> = HashMap::new();
        let mut exception_literals: Vec<String> = Vec::new();
        let mut exception_meta: Vec<Vec<PatternMeta>> = Vec::new();
        let mut exception_index: HashMap<String, usize> = HashMap::new();

        for raw in text.lines() {
            if stats.parsed + stats.skipped >= MAX_LINES {
                stats.skipped += 1;
                continue;
            }
            let line = raw.trim();
            if line.is_empty() || line.starts_with('!') || line.starts_with('[') {
                stats.comments += 1;
                continue;
            }
            if line.len() > MAX_RULE_BYTES {
                stats.skipped += 1;
                continue;
            }
            // Element hiding, scriptlets and comments are outside our grammar.
            if line.starts_with('#') || line.contains("##") || line.contains("#@#") {
                stats.skipped += 1;
                continue;
            }

            let exception = line.starts_with("@@");
            let body = if exception { &line[2..] } else { line };
            let (pattern, modifier_text) = match body.rsplit_once('$') {
                Some((p, m)) if !p.is_empty() => (p, Some(m)),
                _ => (body, None),
            };

            let mut domains: Option<Vec<String>> = None;
            let mut third_party_only = false;
            if let Some(modifiers) = modifier_text {
                let mut unsupported = false;
                for modifier in modifiers.split(',') {
                    if let Some(list) = modifier.strip_prefix("domain=") {
                        if list.contains('~') || list.is_empty() {
                            unsupported = true;
                            break;
                        }
                        domains = Some(
                            list.split('|')
                                .map(|d| d.trim().to_ascii_lowercase())
                                .filter(|d| !d.is_empty())
                                .collect(),
                        );
                    } else if modifier.eq_ignore_ascii_case("third-party") {
                        third_party_only = true;
                    } else {
                        unsupported = true;
                        break;
                    }
                }
                if unsupported {
                    stats.skipped += 1;
                    continue;
                }
            }

            if pattern.starts_with('/') && pattern.ends_with('/') && pattern.len() > 2 {
                stats.skipped += 1; // regular expression
                continue;
            }
            if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
                stats.skipped += 1; // wildcards / character classes
                continue;
            }
            let meta = RuleMeta {
                text: line.to_string(),
                domains,
                third_party_only,
            };

            if let Some(host_part) = pattern.strip_prefix("||") {
                // `||host^` / `||host/path^`: host is everything up to the first
                // `/` or `^`; the remainder is an optional anchored path.
                let end = host_part.find(['/', '^']).unwrap_or(host_part.len());
                let host = host_part[..end].to_ascii_lowercase();
                if host.is_empty() {
                    stats.skipped += 1;
                    continue;
                }
                let rest = &host_part[end..];
                let (path, trailing_boundary) = match rest.strip_prefix('/') {
                    Some(p) => (
                        Some(format!("/{}", p.trim_end_matches('^').to_ascii_lowercase())),
                        p.ends_with('^'),
                    ),
                    None => (None, true), // `||host^`: boundary after the host
                };
                let anchor = Anchor {
                    host,
                    path,
                    trailing_boundary,
                    meta,
                };
                if exception {
                    exception_anchors
                        .entry(anchor.host.clone())
                        .or_default()
                        .push(anchor);
                } else {
                    block_anchors
                        .entry(anchor.host.clone())
                        .or_default()
                        .push(anchor);
                }
            } else {
                if pattern.contains('|') {
                    // A lone `|` is outside the grammar: keeping it as a
                    // literal yields an unanchored fragment that can't match.
                    stats.skipped += 1;
                    continue;
                }
                let leading_boundary = pattern.starts_with('^');
                let trailing_boundary = pattern.ends_with('^');
                let literal: String = pattern
                    .trim_matches('^')
                    .chars()
                    .flat_map(char::to_lowercase)
                    .collect();
                if literal.is_empty() {
                    stats.skipped += 1;
                    continue;
                }
                let pm = PatternMeta {
                    leading_boundary,
                    trailing_boundary,
                    meta,
                };
                let (literals, metas, index) = if exception {
                    (
                        &mut exception_literals,
                        &mut exception_meta,
                        &mut exception_index,
                    )
                } else {
                    (&mut block_literals, &mut block_meta, &mut block_index)
                };
                if let Some(&i) = index.get(&literal) {
                    metas[i].push(pm);
                } else {
                    index.insert(literal.clone(), metas.len());
                    metas.push(vec![pm]);
                    literals.push(literal);
                }
            }
            stats.parsed += 1;
        }

        let mut build = AhoCorasickBuilder::new();
        build
            .match_kind(MatchKind::Standard)
            .ascii_case_insensitive(false);
        let block_ac = (!block_literals.is_empty())
            .then(|| build.build(&block_literals).ok())
            .flatten();
        let exception_ac = (!exception_literals.is_empty())
            .then(|| build.build(&exception_literals).ok())
            .flatten();

        Self {
            block_ac,
            block_ac_meta: block_meta,
            exception_ac,
            exception_ac_meta: exception_meta,
            block_anchors,
            exception_anchors,
            stats,
            build_millis: started.elapsed().as_millis(),
        }
    }

    pub fn stats(&self) -> ParseStats {
        self.stats
    }

    /// Time spent compiling the automatons, in milliseconds.
    pub fn build_millis(&self) -> u128 {
        self.build_millis
    }

    /// Number of compiled substring patterns (useful for perf reports).
    pub fn pattern_count(&self) -> usize {
        self.block_ac_meta.iter().map(Vec::len).sum::<usize>()
            + self.exception_ac_meta.iter().map(Vec::len).sum::<usize>()
    }

    /// Number of compiled host-anchor rules.
    pub fn anchor_count(&self) -> usize {
        self.block_anchors.values().map(Vec::len).sum::<usize>()
            + self.exception_anchors.values().map(Vec::len).sum::<usize>()
    }

    fn match_set(
        &self,
        ac: Option<&AhoCorasick>,
        meta: &[Vec<PatternMeta>],
        anchors: &HashMap<String, Vec<Anchor>>,
        ctx: &MatchCtx<'_>,
    ) -> Option<String> {
        // Host anchors: O(#labels) HashMap probes with dot-suffix fallback.
        // A path is matched only directly after the host, never anywhere else
        // in the URL (query strings and other hosts must not create false hits).
        let mut start = 0;
        loop {
            let suffix = &ctx.host[start..];
            if let Some(entries) = anchors.get(suffix) {
                for anchor in entries {
                    if !rule_applies(&anchor.meta, ctx.top_site, ctx.is_third_party) {
                        continue;
                    }
                    if let Some(path) = &anchor.path {
                        if !ctx.path.starts_with(path.as_str()) {
                            continue;
                        }
                        if anchor.trailing_boundary
                            && ctx.path[path.len()..]
                                .chars()
                                .next()
                                .is_some_and(|c| !is_boundary(c))
                        {
                            continue;
                        }
                    }
                    return Some(anchor.meta.text.clone());
                }
            }
            match suffix.find('.') {
                Some(rel) => start += rel + 1,
                None => break,
            }
        }

        // Substrings: AC finds literal candidates; `^` is enforced as a real
        // separator boundary at each match site instead of being stripped.
        if let Some(ac) = ac {
            for mat in ac.find_iter(ctx.url_lower) {
                for pm in &meta[mat.pattern()] {
                    if !rule_applies(&pm.meta, ctx.top_site, ctx.is_third_party) {
                        continue;
                    }
                    if pm.leading_boundary
                        && mat.start() > 0
                        && ctx.url_lower[..mat.start()]
                            .chars()
                            .next_back()
                            .is_some_and(|c| !is_boundary(c))
                    {
                        continue;
                    }
                    if pm.trailing_boundary
                        && ctx.url_lower[mat.end()..]
                            .chars()
                            .next()
                            .is_some_and(|c| !is_boundary(c))
                    {
                        continue;
                    }
                    return Some(pm.meta.text.clone());
                }
            }
        }
        None
    }

    /// Decide whether `url` should be blocked. Exceptions are checked first.
    pub fn should_block(&self, url: &str, top_site: &str, is_third_party: bool) -> BlockDecision {
        let url_lower = url.to_ascii_lowercase();
        let host = host_from_url(&url_lower);
        let path = path_from_url(&url_lower);
        let ctx = MatchCtx {
            url_lower: &url_lower,
            host: &host,
            path: &path,
            top_site,
            is_third_party,
        };
        if self
            .match_set(
                self.exception_ac.as_ref(),
                &self.exception_ac_meta,
                &self.exception_anchors,
                &ctx,
            )
            .is_some()
        {
            return BlockDecision::Allow;
        }
        match self.match_set(
            self.block_ac.as_ref(),
            &self.block_ac_meta,
            &self.block_anchors,
            &ctx,
        ) {
            Some(rule) => BlockDecision::Block { rule },
            None => BlockDecision::Allow,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct MatchCtx<'a> {
    url_lower: &'a str,
    host: &'a str,
    /// path (with query) after the authority, always starting with `/`
    path: &'a str,
    top_site: &'a str,
    is_third_party: bool,
}

/// `^` in ABP matches a separator: anything that is not alphanumeric.
fn is_boundary(ch: char) -> bool {
    !ch.is_ascii_alphanumeric()
}

/// Lowercased path-plus-query from an absolute URL, anchored at `/`.
fn path_from_url(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let after_authority = rest
        .find(['/', '?', '#'])
        .map(|i| &rest[i..])
        .unwrap_or("/");
    let path = after_authority.split('#').next().unwrap_or(after_authority);
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

fn rule_applies(meta: &RuleMeta, top_site: &str, is_third_party: bool) -> bool {
    if meta.third_party_only && !is_third_party {
        return false;
    }
    if let Some(domains) = &meta.domains {
        let site = top_site.to_ascii_lowercase();
        return domains.iter().any(|d| d.eq_ignore_ascii_case(&site));
    }
    true
}

/// Lowercased host (without userinfo, port or brackets) from an absolute URL.
fn host_from_url(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let hostport = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let host = if let Some(end) = hostport.find(']') {
        &hostport[..=end]
    } else {
        hostport.rsplit_once(':').map_or(hostport, |(h, _)| h)
    };
    host.trim_matches(|c| c == '[' || c == ']').to_string()
}

// ---------------------------------------------------------------------------
// Process-wide default list, disable paths and privacy counters
// ---------------------------------------------------------------------------

struct Global {
    enabled: bool,
    list: FilterList,
    allow_sites: HashSet<String>,
    blocked_urls: VecDeque<String>,
}

static GLOBAL: OnceLock<Mutex<Global>> = OnceLock::new();
static BLOCKED_COUNT: AtomicU64 = AtomicU64::new(0);
static BLOCKED_BYTES: AtomicU64 = AtomicU64::new(0);

fn global() -> &'static Mutex<Global> {
    GLOBAL.get_or_init(|| {
        let env_disabled = std::env::var("HODO_NO_ADBLOCK")
            .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
        Mutex::new(Global {
            enabled: !env_disabled,
            list: load_vendored_lists(),
            allow_sites: HashSet::new(),
            blocked_urls: VecDeque::with_capacity(BLOCKED_SAMPLE_LEN),
        })
    })
}

fn vendored_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/filters")
        .join(name)
}

/// Load the vendored EasyList + EasyPrivacy files plus the tiny built-in net.
/// Missing/unreadable files are tolerated (the built-in rules still apply).
pub fn load_vendored_lists() -> FilterList {
    let mut combined = String::from(BUILTIN_FILTERS);
    combined.push('\n');
    for name in ["easylist.txt", "easyprivacy.txt"] {
        if let Ok(text) = std::fs::read_to_string(vendored_path(name)) {
            combined.push_str(&text);
            combined.push('\n');
        }
    }
    FilterList::parse(&combined)
}

/// Main entry point used by `hodo-net::fetch`. Enabled by default; honours all
/// three disable paths (env at startup, global setter, per-site allowlist).
/// Records the hit in the process-wide counters and URL ring.
pub fn should_block(url: &str, top_site: &str, is_third_party: bool) -> BlockDecision {
    should_block_record(url, top_site, is_third_party, true)
}

/// Evaluates the same rules as [`should_block`] but only records the hit when
/// `record` is true. Private fetches and prefetch pass `false` so their traffic
/// never moves the privacy-report counters or ring.
pub fn should_block_record(
    url: &str,
    top_site: &str,
    is_third_party: bool,
    record: bool,
) -> BlockDecision {
    let mut guard = global().lock().unwrap_or_else(|e| e.into_inner());
    let site = top_site.to_ascii_lowercase();
    if !guard.enabled || (!site.is_empty() && guard.allow_sites.contains(&site)) {
        return BlockDecision::Allow;
    }
    let decision = guard.list.should_block(url, top_site, is_third_party);
    record_block(&decision, url, record, &mut guard);
    decision
}

/// Only a real `Block` moves the blocked counters and URL ring. `Warn` is an
/// advisory hit counted separately by the caller's warning tracker.
fn record_block(decision: &BlockDecision, url: &str, record: bool, guard: &mut Global) {
    if record && matches!(decision, BlockDecision::Block { .. }) {
        BLOCKED_COUNT.fetch_add(1, Ordering::Relaxed);
        BLOCKED_BYTES.fetch_add(url.len() as u64, Ordering::Relaxed);
        if guard.blocked_urls.len() == BLOCKED_SAMPLE_LEN {
            guard.blocked_urls.pop_front();
        }
        guard.blocked_urls.push_back(url.to_string());
    }
}

/// Runtime disable path for about:hodo/settings (also `HODO_NO_ADBLOCK=1`).
pub fn set_adblock_enabled(enabled: bool) {
    global().lock().unwrap_or_else(|e| e.into_inner()).enabled = enabled;
}

pub fn adblock_enabled() -> bool {
    global().lock().unwrap_or_else(|e| e.into_inner()).enabled
}

/// Per-site allowlist: no blocking for `top_site` until revoked.
pub fn allow_site(top_site: &str) {
    global()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .allow_sites
        .insert(top_site.to_ascii_lowercase());
}

pub fn revoke_site(top_site: &str) {
    global()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .allow_sites
        .remove(&top_site.to_ascii_lowercase());
}

pub fn is_site_allowed(top_site: &str) -> bool {
    global()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .allow_sites
        .contains(&top_site.to_ascii_lowercase())
}

pub fn blocked_count() -> u64 {
    BLOCKED_COUNT.load(Ordering::Relaxed)
}

pub fn blocked_bytes() -> u64 {
    BLOCKED_BYTES.load(Ordering::Relaxed)
}

/// Last 50 blocked URLs (oldest first) for the future privacy report.
pub fn blocked_urls() -> Vec<String> {
    global()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .blocked_urls
        .iter()
        .cloned()
        .collect()
}

/// `(parsed, skipped)` for the default global list (vendored + built-in).
pub fn rule_counts() -> (usize, usize) {
    let guard = global().lock().unwrap_or_else(|e| e.into_inner());
    (guard.list.stats.parsed, guard.list.stats.skipped)
}

/// Automaton build time in milliseconds for the default global list.
pub fn automaton_build_millis() -> u128 {
    global()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .list
        .build_millis
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests asserting exact global counter deltas must not interleave.
    static COUNTER_TEST_LOCK: Mutex<()> = Mutex::new(());

    const TINY: &str = "\
! synthetic list\n\
||ads.example.com^\n\
||evil-example.com^\n\
^ad-frame^\n\
@@||ads.example.com/ok^\n\
||doubleclick.net^$domain=news.test\n\
||tracker.test^$third-party\n\
";

    fn decision(list: &FilterList, url: &str, top: &str, third: bool) -> bool {
        matches!(
            list.should_block(url, top, third),
            BlockDecision::Block { .. }
        )
    }

    #[test]
    fn anchor_match_and_suffix_trap() {
        let list = FilterList::parse(TINY);
        assert!(decision(
            &list,
            "https://ads.example.com/banner.js",
            "",
            true
        ));
        assert!(decision(
            &list,
            "https://sub.ads.example.com/b.js",
            "",
            true
        ));
        assert!(!decision(&list, "https://example.com/", "", true));
        // evil-example.com must not match example.com's suffix.
        assert!(decision(&list, "https://evil-example.com/x", "", true));
        assert!(!decision(&list, "https://good-example.com/x", "", true));
    }

    #[test]
    fn exception_precedence() {
        let list = FilterList::parse(TINY);
        assert!(decision(&list, "https://ads.example.com/bad", "", true));
        assert!(
            !decision(&list, "https://ads.example.com/ok", "", true),
            "@@ rule must win"
        );
    }

    #[test]
    fn domain_and_third_party_modifiers() {
        let list = FilterList::parse(TINY);
        assert!(decision(
            &list,
            "https://doubleclick.net/a",
            "news.test",
            true
        ));
        assert!(!decision(
            &list,
            "https://doubleclick.net/a",
            "other.test",
            true
        ));
        assert!(decision(&list, "https://tracker.test/x", "page.test", true));
        assert!(!decision(
            &list,
            "https://tracker.test/x",
            "page.test",
            false
        ));
    }

    #[test]
    fn substring_and_separator_semantics() {
        let list = FilterList::parse(TINY);
        assert!(decision(&list, "https://cdn.test/ad-frame/1.js", "", true));
        assert!(!decision(&list, "https://cdn.test/nothing", "", true));
        // Parsing stats and unsupported-line skipping are observable.
        let stats = FilterList::parse("||ok.test^\n/ad.js$script\n/^ad^/\n##.ad\n");
        assert_eq!(stats.stats().parsed, 1);
        assert_eq!(stats.stats().skipped, 3);
    }

    #[test]
    fn unsupported_lines_are_counted_not_fatal() {
        let mixed = "! comment\n||good.test^\n/ad.js$script\n##.overlay\n@@||good.test/ok^\n";
        let list = FilterList::parse(mixed);
        assert_eq!(list.stats().parsed, 2);
        assert_eq!(list.stats().skipped, 2);
        assert_eq!(list.stats().comments, 1);
        assert!(decision(&list, "https://good.test/x", "", true));
        assert!(!decision(&list, "https://good.test/ok", "", true));
    }

    #[test]
    fn single_bar_rules_are_skipped_and_counted() {
        let list = FilterList::parse("||ok.test^\nfoo|bar\n|leading\ntail|\n");
        assert_eq!(list.stats().parsed, 1);
        assert_eq!(list.stats().skipped, 3);
        assert!(decision(&list, "https://ok.test/x", "", true));
        assert!(!decision(&list, "https://x.test/foo|bar", "", true));
    }

    #[test]
    fn host_rule_path_is_anchored_after_the_host() {
        let list = FilterList::parse("||ads.test/banner^\n");
        assert!(decision(&list, "https://ads.test/banner.js", "", true));
        assert!(
            !decision(&list, "https://ads.test/other?x=banner", "", true),
            "path must not match inside the query"
        );
        assert!(
            !decision(&list, "https://evil.test/?q=/banner", "", true),
            "path must not match on another host"
        );
        assert!(
            !decision(&list, "https://ads.test/bannerX", "", true),
            "trailing ^ requires a separator"
        );
    }

    #[test]
    fn caret_rules_match_separator_boundaries_not_bare_substrings() {
        let block = FilterList::parse("^ad^\n");
        assert!(decision(&block, "https://cdn.test/ad/1.js", "", true));
        assert!(
            !decision(&block, "https://cdn.test/upload", "", true),
            "bare 'ad' inside a word must not match"
        );
        // Stripping ^ used to turn @@^ad^ into a bare `ad` exception that
        // over-blocked (allowed) every URL containing those two letters.
        let exception = FilterList::parse("||ads.test^\n@@^ad^\n");
        assert!(decision(&exception, "https://ads.test/upload", "", true));
        assert!(
            !decision(&exception, "https://ads.test/ad/", "", true),
            "boundary exception must still win where it matches"
        );
    }

    #[test]
    fn unrecorded_blocks_leave_counters_and_ring_untouched() {
        let _guard = COUNTER_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let enabled_before = adblock_enabled();
        set_adblock_enabled(true);
        let url = "https://doubleclick.net/x";
        let before = blocked_count();
        let bytes_before = blocked_bytes();
        let ring_before = blocked_urls();

        assert!(matches!(
            should_block_record(url, "", true, false),
            BlockDecision::Block { .. }
        ));
        assert_eq!(blocked_count(), before);
        assert_eq!(blocked_bytes(), bytes_before);
        assert_eq!(blocked_urls(), ring_before);

        assert!(matches!(
            should_block_record(url, "", true, true),
            BlockDecision::Block { .. }
        ));
        assert_eq!(blocked_count(), before + 1);
        assert_eq!(blocked_bytes(), bytes_before + url.len() as u64);
        set_adblock_enabled(enabled_before);
    }

    #[test]
    fn perf_smoke_2000_rules_1000_urls() {
        let mut rules = String::new();
        for i in 0..2000 {
            rules.push_str(&format!("||ad{i:04}.test^\n"));
        }
        let started = Instant::now();
        let list = FilterList::parse(&rules);
        let build = started.elapsed();
        let urls: Vec<String> = (0..1000)
            .map(|i| format!("https://cdn{i:04}.test/page{i}/asset.js"))
            .collect();
        let match_start = Instant::now();
        let mut hits = 0;
        for url in &urls {
            if decision(&list, url, "", true) {
                hits += 1;
            }
        }
        let elapsed = match_start.elapsed();
        eprintln!(
            "tiny list: build {build:?}, 1000 urls {elapsed:?} ({:?}/url)",
            elapsed / 1000
        );
        assert_eq!(hits, 0);
        assert!(
            elapsed.as_millis() < 50,
            "1000 URLs took {elapsed:?} (budget 50ms debug)"
        );
    }

    #[test]
    fn real_vendored_lists_parse_and_block() {
        let list = load_vendored_lists();
        let stats = list.stats();
        eprintln!(
            "real lists: parsed={} skipped={} patterns={} anchors={} build={}ms",
            stats.parsed,
            stats.skipped,
            list.pattern_count(),
            list.anchor_count(),
            list.build_millis()
        );
        assert!(
            stats.parsed > 50_000,
            "expected >50000 rules, got {}",
            stats.parsed
        );
        assert!(decision(&list, "https://doubleclick.net/banner", "", true));
        assert!(decision(
            &list,
            "https://googlesyndication.com/pagead/ads.js",
            "",
            true
        ));
        assert!(
            !decision(&list, "https://example.com/", "", true),
            "example.com must be allowed"
        );
        // A real @@ exception from EasyPrivacy is honoured.
        assert!(
            !decision(
                &list,
                "https://google-analytics.com/analytics.js",
                "beinsports.com",
                true
            ),
            "EasyPrivacy @@ exception must win"
        );
    }

    #[test]
    fn per_list_counts_are_reported() {
        for (name, floor) in [("easylist.txt", 40_000), ("easyprivacy.txt", 40_000)] {
            let text = std::fs::read_to_string(vendored_path(name)).unwrap_or_default();
            let list = FilterList::parse(&text);
            let stats = list.stats();
            eprintln!(
                "{name}: parsed={} skipped={} comments={} patterns={} anchors={} build_ms={}",
                stats.parsed,
                stats.skipped,
                stats.comments,
                list.pattern_count(),
                list.anchor_count(),
                list.build_millis()
            );
            assert!(stats.parsed > floor, "{name}: only {} rules", stats.parsed);
        }
    }

    #[test]
    fn default_list_perf_smoke() {
        // Combined vendored lists (~98k rules) exercise the full matcher.
        let list = load_vendored_lists();
        let urls: Vec<String> = (0..1000)
            .map(|i| format!("https://cdn{i:04}.example.org/a/b/asset{i}.js?x=1"))
            .collect();
        let started = Instant::now();
        let mut hits = 0usize;
        for url in &urls {
            if decision(&list, url, "", true) {
                hits += 1;
            }
        }
        let elapsed = started.elapsed();
        let per_url_us = elapsed.as_micros() / 1000;
        eprintln!(
            "real list: {} rules, 1000 urls in {elapsed:?} (~{per_url_us} us/url), hits={hits}",
            list.stats().parsed
        );
        #[cfg(not(debug_assertions))]
        assert!(
            per_url_us < 1000,
            "per-URL budget 1ms exceeded: {per_url_us}us"
        );
        #[cfg(debug_assertions)]
        assert!(elapsed.as_millis() < 2000, "debug perf sanity: {elapsed:?}");
    }

    #[test]
    fn disable_paths_and_counters() {
        let _guard = COUNTER_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let enabled_before = adblock_enabled();
        let url = "https://doubleclick.net/x";
        let before = blocked_count();
        set_adblock_enabled(false);
        assert!(!adblock_enabled());
        assert_eq!(should_block(url, "", true), BlockDecision::Allow);
        set_adblock_enabled(true);
        allow_site("doubleclick.net");
        assert!(is_site_allowed("DOUBLECLICK.net"));
        assert_eq!(
            should_block(url, "doubleclick.net", true),
            BlockDecision::Allow
        );
        revoke_site("doubleclick.net");
        assert!(matches!(
            should_block(url, "", true),
            BlockDecision::Block { .. }
        ));
        assert_eq!(blocked_count(), before + 1);
        assert!(blocked_bytes() > 0);
        assert!(!blocked_urls().is_empty());
        set_adblock_enabled(enabled_before);
    }

    #[test]
    fn warn_variant_is_part_of_the_api() {
        let decision = BlockDecision::Warn {
            rule: "||warn.test^".into(),
        };
        assert_eq!(
            decision,
            BlockDecision::Warn {
                rule: "||warn.test^".into()
            }
        );
    }

    #[test]
    fn warn_decision_does_not_touch_block_counters() {
        let _guard = COUNTER_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let url = "https://warn-only.test/ad";
        let before = blocked_count();
        let bytes_before = blocked_bytes();
        let ring_before = blocked_urls();

        let mut g = global().lock().unwrap_or_else(|e| e.into_inner());
        let warn = BlockDecision::Warn {
            rule: "||warn-only.test^".into(),
        };
        record_block(&warn, url, true, &mut g);
        drop(g);

        assert_eq!(blocked_count(), before);
        assert_eq!(blocked_bytes(), bytes_before);
        assert_eq!(blocked_urls(), ring_before);

        // A real Block still moves all three, so the split is meaningful.
        let mut g = global().lock().unwrap_or_else(|e| e.into_inner());
        let block = BlockDecision::Block {
            rule: "||warn-only.test^".into(),
        };
        record_block(&block, url, true, &mut g);
        drop(g);
        assert_eq!(blocked_count(), before + 1);
        assert_eq!(blocked_bytes(), bytes_before + url.len() as u64);
    }
}
