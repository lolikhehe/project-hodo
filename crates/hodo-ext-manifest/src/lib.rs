//! Chrome and Gecko WebExtension manifest parsing (MV2 and MV3).
//!
//! Content scripts are frozen here as a parsed contract ([`ContentScript`],
//! [`MatchPattern`], [`RunAt`]). Injection is a separate, later wave that
//! consumes these structs and never re-parses the raw manifest strings.
//!
//! Declarative net request follows the same split: [`DnrRuleResource`] is the
//! manifest-side declaration and [`parse_dnr_rules`] freezes the separate rule
//! files into [`DnrRule`]. Applying those rules is a later, engine-side wave.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use hodo_ext_i18n::{I18nError, Messages};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestVersion {
    V2,
    V3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserFlavor {
    Chrome,
    Gecko,
    Generic,
}

/// Injection time for a content script, mirroring the WebExtension `run_at`
/// values. Absent `run_at` means [`RunAt::DocumentIdle`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RunAt {
    DocumentStart,
    DocumentEnd,
    #[default]
    DocumentIdle,
}

/// Scheme half of a parsed match pattern. [`Scheme::Any`] is the `*` token and
/// matches http, https and file URLs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    Http,
    Https,
    File,
    Any,
}

/// A parsed `<scheme>://<host><path>` WebExtension match pattern. Hosts are
/// stored as written; matching lowercases both sides. Kept structured so the
/// injector never has to re-parse a raw pattern string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchPattern {
    pub scheme: Scheme,
    pub host: String,
    pub path: String,
}

/// One `content_scripts` entry. MV3 nests these under `content_scripts`; MV2
/// declares the same top-level key. `js`/`css` hold validated in-package paths
/// only, and `__MSG_` tokens are deliberately not substituted into them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentScript {
    pub matches: Vec<MatchPattern>,
    pub js: Vec<String>,
    pub css: Vec<String>,
    pub run_at: RunAt,
}

/// One `declarative_net_request.rule_resources` entry. `path` is validated as a
/// relative in-package path; rule application is a later engine-side wave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnrRuleResource {
    pub id: u32,
    pub enabled: bool,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub version: ManifestVersion,
    pub name: String,
    pub ext_version: String,
    pub permissions: Vec<String>,
    /// URL patterns from `host_permissions`; never grant API namespaces.
    pub host_permissions: Vec<String>,
    /// Gecko/Edge MV2 style background scripts.
    pub background_scripts: Vec<String>,
    /// Chrome MV3 style service worker.
    pub service_worker: Option<String>,
    pub gecko_id: Option<String>,
    /// Top-level `"key"` field used by the Chrome Web Store.
    pub chrome_key: Option<String>,
    pub action_title: Option<String>,
    /// `action.default_popup` / `browser_action.default_popup`, validated as a
    /// relative in-package path. Deliberately NOT a `__MSG_` target: it is a
    /// file path, so substituting locale text would break the lookup.
    pub popup_path: Option<String>,
    /// Largest `action.default_icon` / `browser_action.default_icon` entry as
    /// `(size, path)`. Size `0` means the single-string form was used (no size
    /// declared). Paths are validated like `popup_path` and are likewise not
    /// `__MSG_` targets. Top-level `icons` is a separate field and is not a
    /// fallback for this one.
    pub popup_icon: Option<(u32, String)>,
    /// Parsed `content_scripts` entries (same key in MV2 and MV3). Empty when
    /// the manifest declares none.
    pub content_scripts: Vec<ContentScript>,
    /// Parsed `declarative_net_request.rule_resources` entries. Empty when the
    /// manifest declares none; declaring any requires a DNR permission.
    pub declarative_net_request: Vec<DnrRuleResource>,
    pub icons: HashMap<String, String>,
    /// Required when the extension ships a `_locales` directory.
    pub default_locale: Option<String>,
    /// Kept so `__MSG_` tokens can be substituted (CSS/HTML are NOT touched).
    pub description: Option<String>,
    /// `name` after `__MSG_` resolution; set by [`resolve_messages`].
    pub resolved_name: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("invalid json: {0}")]
    Json(String),
    #[error("missing field: {0}")]
    Missing(&'static str),
    #[error("unsupported manifest_version: {0}")]
    UnsupportedVersion(u8),
    #[error("invalid in-package path: {0}")]
    InvalidPath(String),
    #[error("invalid match pattern: {0}")]
    InvalidMatchPattern(String),
    /// Ports in match patterns are deferred to wave 1; reject rather than
    /// guess how a port should be matched.
    #[error("match pattern ports are not supported yet (wave 1): {0}")]
    MatchPatternPort(String),
    #[error("invalid content_scripts entry: {0}")]
    InvalidContentScript(String),
    #[error("too many content_scripts entries: {0} (max {MAX_CONTENT_SCRIPTS})")]
    TooManyContentScripts(usize),
    #[error("too many {0} files in a content_scripts entry: {1} (max {MAX_SCRIPT_FILES})")]
    TooManyContentScriptFiles(String, usize),
    #[error("invalid declarative_net_request: {0}")]
    InvalidDnr(String),
    #[error("declarative_net_request requires the declarativeNetRequest or declarativeNetRequestFeedback permission")]
    DnrMissingPermission,
}

/// Lowercase-normalize and dedup one permission field, skipping non-strings.
fn normalized_list(root: &Value, key: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    if let Some(values) = root.get(key).and_then(Value::as_array) {
        for value in values.iter().filter_map(Value::as_str) {
            let normalized = value.to_lowercase();
            if seen.insert(normalized.clone()) {
                out.push(normalized);
            }
        }
    }
    out
}

/// Keep only relative paths that stay inside the extension package. Blocks URL
/// schemes, OS-absolute paths (POSIX, Windows drive, UNC) and `..` traversal so
/// a manifest cannot point the host at remote or outside content.
fn validate_package_path(raw: &str) -> Result<String, ManifestError> {
    let lower = raw.to_ascii_lowercase();
    if lower.starts_with("javascript:")
        || lower.starts_with("data:")
        || lower.starts_with("http://")
        || lower.starts_with("https://")
    {
        return Err(ManifestError::InvalidPath(raw.to_owned()));
    }
    let bytes = raw.as_bytes();
    if bytes.is_empty()
        || bytes[0] == b'/'
        || bytes[0] == b'\\'
        || (bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic())
        || raw.split(['/', '\\']).any(|part| part == "..")
    {
        return Err(ManifestError::InvalidPath(raw.to_owned()));
    }
    Ok(raw.to_owned())
}

/// Fail-closed parse caps for content scripts. Oversized manifests are rejected
/// outright rather than truncated, so a hostile extension cannot inflate the
/// injector's per-page work.
pub const MAX_CONTENT_SCRIPTS: usize = 64;
pub const MAX_SCRIPT_FILES: usize = 64;

fn parse_scheme(raw: &str) -> Option<Scheme> {
    match raw {
        "http" => Some(Scheme::Http),
        "https" => Some(Scheme::Https),
        "file" => Some(Scheme::File),
        "*" => Some(Scheme::Any),
        _ => None,
    }
}

/// A literal host carries no wildcard and no port; only DNS-ish characters
/// pass, so stray `*`, `:` or whitespace is rejected.
fn is_literal_host(host: &str) -> bool {
    !host.is_empty()
        && !host.starts_with('.')
        && !host.ends_with('.')
        && !host.contains("..")
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

impl MatchPattern {
    /// Parse a `<scheme>://<host><path>` pattern into its components, rejecting
    /// anything outside that shape (empty input, missing scheme/path, `**`,
    /// ports — which are deferred to wave 1 — and malformed wildcard hosts).
    pub fn parse(raw: &str) -> Result<Self, ManifestError> {
        if raw.is_empty() || raw.contains("**") {
            return Err(ManifestError::InvalidMatchPattern(raw.to_owned()));
        }
        let (scheme_raw, rest) = raw
            .split_once("://")
            .ok_or_else(|| ManifestError::InvalidMatchPattern(raw.to_owned()))?;
        let scheme = parse_scheme(scheme_raw)
            .ok_or_else(|| ManifestError::InvalidMatchPattern(raw.to_owned()))?;

        let (host, path) = match rest.find('/') {
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, ""),
        };
        if host.contains(':') {
            return Err(ManifestError::MatchPatternPort(raw.to_owned()));
        }
        if !path.starts_with('/') || path.chars().any(char::is_whitespace) {
            return Err(ManifestError::InvalidMatchPattern(raw.to_owned()));
        }
        let host_ok = match host {
            "*" => true,
            "" => scheme == Scheme::File,
            _ => match host.strip_prefix("*.") {
                Some(base) => is_literal_host(base),
                None => is_literal_host(host),
            },
        };
        if !host_ok {
            return Err(ManifestError::InvalidMatchPattern(raw.to_owned()));
        }
        Ok(Self {
            scheme,
            host: host.to_owned(),
            path: path.to_owned(),
        })
    }

    /// Whether this pattern matches a page URL. Pure logic used by the later
    /// injector on each load: scheme `*` covers http/https/file, `*.host` also
    /// covers the bare host and all subdomains, and path uses `*` split globs.
    pub fn matches_url(&self, url: &str) -> bool {
        let Some((scheme, url_host, url_path)) = split_url(url) else {
            return false;
        };
        let scheme_ok = match self.scheme {
            Scheme::Any => matches!(scheme, Scheme::Http | Scheme::Https | Scheme::File),
            other => other == scheme,
        };
        if !scheme_ok {
            return false;
        }
        let host = url_host.to_ascii_lowercase();
        let pattern_host = self.host.to_ascii_lowercase();
        let host_ok = if pattern_host == "*" {
            true
        } else if let Some(base) = pattern_host.strip_prefix("*.") {
            host == base || host.ends_with(&format!(".{base}"))
        } else {
            host == pattern_host
        };
        host_ok && glob_matches(&self.path, url_path)
    }
}

/// Split a URL into the pieces a match pattern cares about. The port is
/// dropped (patterns cannot carry one in wave 1) and the path excludes query
/// and fragment.
fn split_url(url: &str) -> Option<(Scheme, &str, &str)> {
    let (scheme_raw, rest) = url.split_once("://")?;
    let scheme = parse_scheme(&scheme_raw.to_ascii_lowercase())?;
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    let host = authority.split(':').next().unwrap_or(authority);
    let path_start = rest.find('/').unwrap_or(rest.len());
    let query_start = rest.find(['?', '#']).unwrap_or(rest.len());
    let path = if path_start < query_start {
        &rest[path_start..query_start]
    } else {
        "/"
    };
    Some((scheme, host, path))
}

/// Glob matching where `*` stands for any (possibly empty) run of characters.
fn glob_matches(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    let first = parts[0];
    if !text.starts_with(first) {
        return false;
    }
    let mut remaining = &text[first.len()..];
    let last = parts[parts.len() - 1];
    if parts.len() == 1 {
        return remaining.is_empty();
    }
    for part in &parts[1..parts.len() - 1] {
        if part.is_empty() {
            continue;
        }
        let Some(idx) = remaining.find(part) else {
            return false;
        };
        remaining = &remaining[idx + part.len()..];
    }
    remaining.ends_with(last)
}

fn parse_file_list(entry: &Value, key: &str) -> Result<Vec<String>, ManifestError> {
    let Some(values) = entry.get(key) else {
        return Ok(Vec::new());
    };
    let values = values
        .as_array()
        .ok_or_else(|| ManifestError::InvalidContentScript(format!("{key} must be an array")))?;
    if values.len() > MAX_SCRIPT_FILES {
        return Err(ManifestError::TooManyContentScriptFiles(
            key.to_owned(),
            values.len(),
        ));
    }
    values
        .iter()
        .map(|value| {
            let path = value.as_str().ok_or_else(|| {
                ManifestError::InvalidContentScript(format!("{key} must contain only strings"))
            })?;
            validate_package_path(path)
        })
        .collect()
}

fn parse_content_scripts(root: &Value) -> Result<Vec<ContentScript>, ManifestError> {
    let Some(entries_value) = root.get("content_scripts") else {
        return Ok(Vec::new());
    };
    let entries = entries_value.as_array().ok_or_else(|| {
        ManifestError::InvalidContentScript("content_scripts must be an array".to_owned())
    })?;
    if entries.len() > MAX_CONTENT_SCRIPTS {
        return Err(ManifestError::TooManyContentScripts(entries.len()));
    }

    let mut scripts = Vec::with_capacity(entries.len());
    for entry in entries {
        if !entry.is_object() {
            return Err(ManifestError::InvalidContentScript(
                "entry must be an object".to_owned(),
            ));
        }
        let matches = entry
            .get("matches")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                ManifestError::InvalidContentScript("matches must be an array".to_owned())
            })?
            .iter()
            .map(|value| {
                let raw = value.as_str().ok_or_else(|| {
                    ManifestError::InvalidContentScript(
                        "matches must contain only strings".to_owned(),
                    )
                })?;
                MatchPattern::parse(raw)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let js = parse_file_list(entry, "js")?;
        let css = parse_file_list(entry, "css")?;
        let run_at = match entry.get("run_at") {
            None => RunAt::DocumentIdle,
            Some(Value::String(raw)) => match raw.as_str() {
                "document_start" => RunAt::DocumentStart,
                "document_end" => RunAt::DocumentEnd,
                "document_idle" => RunAt::DocumentIdle,
                other => {
                    return Err(ManifestError::InvalidContentScript(format!(
                        "unknown run_at: {other}"
                    )))
                }
            },
            Some(_) => {
                return Err(ManifestError::InvalidContentScript(
                    "run_at must be a string".to_owned(),
                ))
            }
        };
        scripts.push(ContentScript {
            matches,
            js,
            css,
            run_at,
        });
    }
    Ok(scripts)
}

/// Parse the manifest's `declarative_net_request.rule_resources`. Unknown keys
/// are rejected rather than ignored (Chrome ignores them) so a typo cannot
/// silently drop a declared rule resource. The permission gate is checked only
/// when at least one resource is actually declared.
fn parse_dnr_resources(
    root: &Value,
    permissions: &[String],
) -> Result<Vec<DnrRuleResource>, ManifestError> {
    let Some(value) = root.get("declarative_net_request") else {
        return Ok(Vec::new());
    };
    let object = value.as_object().ok_or_else(|| {
        ManifestError::InvalidDnr("declarative_net_request must be an object".to_owned())
    })?;
    for key in object.keys() {
        if key != "rule_resources" {
            return Err(ManifestError::InvalidDnr(format!(
                "unknown declarative_net_request field: {key}"
            )));
        }
    }
    let Some(entries) = object.get("rule_resources") else {
        return Ok(Vec::new());
    };
    let entries = entries
        .as_array()
        .ok_or_else(|| ManifestError::InvalidDnr("rule_resources must be an array".to_owned()))?;
    if entries.is_empty() {
        return Ok(Vec::new());
    }

    // Permission names are lowercased by `normalized_list`, so compare
    // case-insensitively against the two DNR permissions Chrome recognizes.
    let has_permission = permissions.iter().any(|permission| {
        permission.eq_ignore_ascii_case("declarativeNetRequest")
            || permission.eq_ignore_ascii_case("declarativeNetRequestFeedback")
    });
    if !has_permission {
        return Err(ManifestError::DnrMissingPermission);
    }

    let mut seen = HashSet::new();
    let mut resources = Vec::with_capacity(entries.len());
    for entry in entries {
        let obj = entry.as_object().ok_or_else(|| {
            ManifestError::InvalidDnr("rule_resources entries must be objects".to_owned())
        })?;
        for key in obj.keys() {
            if !matches!(key.as_str(), "id" | "enabled" | "path") {
                return Err(ManifestError::InvalidDnr(format!(
                    "unknown rule_resources field: {key}"
                )));
            }
        }
        let id = obj
            .get("id")
            .and_then(Value::as_u64)
            .and_then(|id| u32::try_from(id).ok())
            .ok_or_else(|| {
                ManifestError::InvalidDnr("rule_resources id must be a u32".to_owned())
            })?;
        if !seen.insert(id) {
            return Err(ManifestError::InvalidDnr(format!(
                "duplicate rule_resources id: {id}"
            )));
        }
        let enabled = obj.get("enabled").and_then(Value::as_bool).ok_or_else(|| {
            ManifestError::InvalidDnr("rule_resources enabled must be a boolean".to_owned())
        })?;
        let path = obj
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ManifestError::InvalidDnr("rule_resources path must be a string".to_owned())
            })
            .and_then(validate_package_path)?;
        resources.push(DnrRuleResource { id, enabled, path });
    }
    Ok(resources)
}

/// Accepts both Chrome-style and Gecko-style WebExtension manifests.
pub fn parse(json: &str) -> Result<Manifest, ManifestError> {
    let root: Value = serde_json::from_str(json).map_err(|e| ManifestError::Json(e.to_string()))?;

    let name = root
        .get("name")
        .and_then(Value::as_str)
        .ok_or(ManifestError::Missing("name"))?
        .to_owned();
    let ext_version = root
        .get("version")
        .and_then(Value::as_str)
        .ok_or(ManifestError::Missing("version"))?
        .to_owned();

    let version = match root.get("manifest_version").and_then(Value::as_u64) {
        Some(2) => ManifestVersion::V2,
        Some(3) => ManifestVersion::V3,
        other => {
            return Err(ManifestError::UnsupportedVersion(
                u8::try_from(other.unwrap_or(0)).unwrap_or(u8::MAX),
            ));
        }
    };

    // Normalize each field independently: host permissions are URL patterns and
    // must never be conflated with the API permissions the host gates on.
    let permissions = normalized_list(&root, "permissions");
    let host_permissions = normalized_list(&root, "host_permissions");
    let content_scripts = parse_content_scripts(&root)?;
    let declarative_net_request = parse_dnr_resources(&root, &permissions)?;

    let mut background_scripts = Vec::new();
    let mut service_worker = None;
    if let Some(background) = root.get("background").and_then(Value::as_object) {
        if let Some(scripts) = background.get("scripts").and_then(Value::as_array) {
            background_scripts.extend(scripts.iter().filter_map(Value::as_str).map(str::to_owned));
        }
        service_worker = background
            .get("service_worker")
            .and_then(Value::as_str)
            .map(str::to_owned);
    }

    let gecko_id = ["browser_specific_settings", "applications"]
        .iter()
        .filter_map(|key| root.get(*key))
        .filter_map(|settings| settings.get("gecko"))
        .filter_map(|gecko| gecko.get("id"))
        .find_map(Value::as_str)
        .map(str::to_owned);

    let chrome_key = root.get("key").and_then(Value::as_str).map(str::to_owned);
    let default_locale = root
        .get("default_locale")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let description = root
        .get("description")
        .and_then(Value::as_str)
        .map(str::to_owned);

    let action_title = ["action", "browser_action"]
        .iter()
        .filter_map(|key| root.get(*key))
        .filter_map(|action| action.get("default_title"))
        .find_map(Value::as_str)
        .map(str::to_owned);

    let popup_path = ["action", "browser_action"]
        .iter()
        .filter_map(|key| root.get(*key))
        .filter_map(|action| action.get("default_popup"))
        .find_map(Value::as_str)
        .map(validate_package_path)
        .transpose()?;

    // Validate every declared icon path, then keep the largest size. A single
    // string has no declared size, represented by 0.
    let mut icon_candidates: Vec<(u32, String)> = Vec::new();
    for key in ["action", "browser_action"] {
        let Some(icon) = root.get(key).and_then(|action| action.get("default_icon")) else {
            continue;
        };
        match icon {
            Value::String(path) => icon_candidates.push((0, validate_package_path(path)?)),
            Value::Object(entries) => {
                for (size, path) in entries {
                    let (Ok(size), Some(path)) = (size.parse::<u32>(), path.as_str()) else {
                        continue;
                    };
                    icon_candidates.push((size, validate_package_path(path)?));
                }
            }
            _ => {}
        }
        if !icon_candidates.is_empty() {
            break;
        }
    }
    let popup_icon = icon_candidates.into_iter().max_by_key(|(size, _)| *size);

    let icons = match root.get("icons").and_then(Value::as_object) {
        Some(entries) => entries
            .iter()
            .filter_map(|(size, path)| path.as_str().map(|path| (size.clone(), path.to_owned())))
            .collect(),
        None => HashMap::new(),
    };

    Ok(Manifest {
        version,
        name,
        ext_version,
        permissions,
        host_permissions,
        background_scripts,
        service_worker,
        gecko_id,
        chrome_key,
        action_title,
        popup_path,
        popup_icon,
        content_scripts,
        declarative_net_request,
        icons,
        default_locale,
        description,
        resolved_name: None,
    })
}

pub fn flavor(m: &Manifest) -> BrowserFlavor {
    if m.gecko_id.is_some() {
        BrowserFlavor::Gecko
    } else if m.chrome_key.is_some() {
        BrowserFlavor::Chrome
    } else {
        BrowserFlavor::Generic
    }
}

/// Resolve `__MSG_name__` tokens in `name`, `description` and `action_title`.
/// `popup_path` and `popup_icon` are deliberately not rewritten: they are
/// in-package file paths, and substituting locale text into a path would break
/// the file lookup (and could even reintroduce an invalid path). CSS is
/// likewise not rewritten, and HTML is only localized explicitly through
/// `i18n.getMessage` in the JS bridge. Called by the host/loader once the
/// extension root is known; `parse` stays root-free and cheap.
pub fn resolve_messages(root: &Path, manifest: &mut Manifest) -> Result<(), I18nError> {
    resolve_messages_with_ui(root, manifest, &hodo_ext_i18n::ui_locale())
}

/// Same as [`resolve_messages`] with an explicit UI locale (tests, `--lang`).
pub fn resolve_messages_with_ui(
    root: &Path,
    manifest: &mut Manifest,
    ui_locale: &str,
) -> Result<(), I18nError> {
    let messages =
        hodo_ext_i18n::load_locales(root, manifest.default_locale.as_deref(), ui_locale)?
            .with_extension_id(manifest.gecko_id.as_deref().unwrap_or(""));
    manifest.name = substitute_messages(&manifest.name, &messages);
    manifest.description = manifest
        .description
        .take()
        .map(|text| substitute_messages(&text, &messages));
    manifest.action_title = manifest
        .action_title
        .take()
        .map(|text| substitute_messages(&text, &messages));
    manifest.resolved_name = Some(manifest.name.clone());
    Ok(())
}

/// Single pass: text produced for a token is never rescanned.
fn substitute_messages(text: &str, messages: &Messages) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'_' && text[i..].starts_with("__MSG_") {
            if let Some(offset) = text[i + 6..].find("__") {
                let end = i + 6 + offset + 2;
                let name = &text[i + 6..end - 2];
                if !name.is_empty()
                    && name
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'@')
                {
                    out.push_str(&messages.get_message(name, &[]));
                    i = end;
                    continue;
                }
            }
        }
        match text[i..].chars().next() {
            Some(ch) => {
                out.push(ch);
                i += ch.len_utf8();
            }
            None => break,
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Declarative net request rule files
// ---------------------------------------------------------------------------
//
// Frozen contract for the separate JSON rule files named by
// `declarative_net_request.rule_resources.path`. Chrome silently ignores unknown
// rule fields; we reject them so a typo cannot turn into a rule the engine never
// enforces. Unknown keys, bad shapes and oversized inputs all fail closed.

/// Fail-closed caps for a single DNR rule file.
pub const MAX_DNR_RULES: usize = 30_000;
pub const MAX_DNR_REGEX_LEN: usize = 1_024;
pub const MAX_DNR_DOMAINS: usize = 1_000;

#[derive(Debug, thiserror::Error)]
pub enum DnrError {
    #[error("invalid json: {0}")]
    Json(String),
    #[error("rule file must be a JSON array of rule objects")]
    NotAnArray,
    #[error("too many DNR rules: {0} (max {MAX_DNR_RULES})")]
    TooManyRules(usize),
    #[error("duplicate DNR rule id: {0}")]
    DuplicateId(u32),
    #[error("regexFilter is {0} chars (max {MAX_DNR_REGEX_LEN})")]
    RegexTooLong(usize),
    #[error("{0} has {1} entries (max {MAX_DNR_DOMAINS})")]
    TooManyDomains(&'static str, usize),
    #[error("rule {index}: {reason}")]
    InvalidRule { index: usize, reason: String },
    #[error("rule {index}: unknown {context} field: {field}")]
    UnknownField {
        index: usize,
        context: &'static str,
        field: String,
    },
}

/// The fixed WebExtension resource-type set. Anything outside it is rejected
/// instead of being passed through as an unknown filter to the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnrResourceType {
    MainFrame,
    SubFrame,
    StyleSheet,
    Script,
    Image,
    Font,
    Object,
    XmlHttpRequest,
    Other,
}

impl DnrResourceType {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "main_frame" => Some(Self::MainFrame),
            "sub_frame" => Some(Self::SubFrame),
            "stylesheet" => Some(Self::StyleSheet),
            "script" => Some(Self::Script),
            "image" => Some(Self::Image),
            "font" => Some(Self::Font),
            "object" => Some(Self::Object),
            "xmlhttprequest" => Some(Self::XmlHttpRequest),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnrActionType {
    Block,
    Allow,
    Redirect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnrCondition {
    pub url_filter: Option<String>,
    pub regex_filter: Option<String>,
    pub domains: Option<Vec<String>>,
    pub initiator_domains: Option<Vec<String>>,
    pub excluded_domains: Option<Vec<String>>,
    pub excluded_initiator_domains: Option<Vec<String>>,
    pub resource_types: Option<Vec<DnrResourceType>>,
    pub is_url_filter_case_sensitive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnrAction {
    pub type_: DnrActionType,
    pub redirect_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnrRule {
    pub id: u32,
    pub priority: u32,
    pub condition: DnrCondition,
    pub action: DnrAction,
}

fn reject_unknown_dnr(
    obj: &Map<String, Value>,
    allowed: &[&str],
    index: usize,
    context: &'static str,
) -> Result<(), DnrError> {
    for key in obj.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(DnrError::UnknownField {
                index,
                context,
                field: key.clone(),
            });
        }
    }
    Ok(())
}

fn dnr_u32(obj: &Value, field: &str, index: usize) -> Result<u32, DnrError> {
    let value = obj.get(field).ok_or_else(|| DnrError::InvalidRule {
        index,
        reason: format!("missing {field}"),
    })?;
    value
        .as_u64()
        .and_then(|number| u32::try_from(number).ok())
        .ok_or_else(|| DnrError::InvalidRule {
            index,
            reason: format!("{field} must be a u32"),
        })
}

fn dnr_string(obj: &Value, field: &str, index: usize) -> Result<Option<String>, DnrError> {
    match obj.get(field) {
        None => Ok(None),
        Some(Value::String(raw)) => Ok(Some(raw.clone())),
        Some(_) => Err(DnrError::InvalidRule {
            index,
            reason: format!("{field} must be a string"),
        }),
    }
}

fn dnr_string_list(
    obj: &Value,
    field: &'static str,
    index: usize,
) -> Result<Option<Vec<String>>, DnrError> {
    let Some(value) = obj.get(field) else {
        return Ok(None);
    };
    let array = value.as_array().ok_or_else(|| DnrError::InvalidRule {
        index,
        reason: format!("{field} must be an array"),
    })?;
    if array.len() > MAX_DNR_DOMAINS {
        return Err(DnrError::TooManyDomains(field, array.len()));
    }
    array
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| DnrError::InvalidRule {
                    index,
                    reason: format!("{field} must contain only strings"),
                })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn dnr_resource_types(obj: &Value, index: usize) -> Result<Option<Vec<DnrResourceType>>, DnrError> {
    let Some(value) = obj.get("resourceTypes") else {
        return Ok(None);
    };
    let array = value.as_array().ok_or_else(|| DnrError::InvalidRule {
        index,
        reason: "resourceTypes must be an array".to_owned(),
    })?;
    array
        .iter()
        .map(|value| {
            let raw = value.as_str().ok_or_else(|| DnrError::InvalidRule {
                index,
                reason: "resourceTypes must contain only strings".to_owned(),
            })?;
            DnrResourceType::parse(raw).ok_or_else(|| DnrError::InvalidRule {
                index,
                reason: format!("unknown resource type: {raw}"),
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn parse_dnr_condition(value: &Value, index: usize) -> Result<DnrCondition, DnrError> {
    let obj = value.as_object().ok_or_else(|| DnrError::InvalidRule {
        index,
        reason: "condition must be an object".to_owned(),
    })?;
    reject_unknown_dnr(
        obj,
        &[
            "urlFilter",
            "regexFilter",
            "domains",
            "initiatorDomains",
            "excludedDomains",
            "excludedInitiatorDomains",
            "resourceTypes",
            "isUrlFilterCaseSensitive",
        ],
        index,
        "condition",
    )?;
    let url_filter = dnr_string(value, "urlFilter", index)?;
    let regex_filter = dnr_string(value, "regexFilter", index)?;
    if url_filter.is_some() == regex_filter.is_some() {
        return Err(DnrError::InvalidRule {
            index,
            reason: "condition must have exactly one of urlFilter or regexFilter".to_owned(),
        });
    }
    if let Some(regex) = &regex_filter {
        let len = regex.chars().count();
        if len > MAX_DNR_REGEX_LEN {
            return Err(DnrError::RegexTooLong(len));
        }
    }
    let is_url_filter_case_sensitive = match value.get("isUrlFilterCaseSensitive") {
        None => false,
        Some(Value::Bool(flag)) => *flag,
        Some(_) => {
            return Err(DnrError::InvalidRule {
                index,
                reason: "isUrlFilterCaseSensitive must be a boolean".to_owned(),
            })
        }
    };
    Ok(DnrCondition {
        url_filter,
        regex_filter,
        domains: dnr_string_list(value, "domains", index)?,
        initiator_domains: dnr_string_list(value, "initiatorDomains", index)?,
        excluded_domains: dnr_string_list(value, "excludedDomains", index)?,
        excluded_initiator_domains: dnr_string_list(value, "excludedInitiatorDomains", index)?,
        resource_types: dnr_resource_types(value, index)?,
        is_url_filter_case_sensitive,
    })
}

/// Absolute HTTP(S) URLs only: a scheme plus a non-empty, whitespace-free
/// authority. Relative, file/data/javascript URLs are rejected.
fn is_absolute_http_url(raw: &str) -> bool {
    let lower = raw.to_ascii_lowercase();
    let Some(rest) = lower
        .strip_prefix("http://")
        .or_else(|| lower.strip_prefix("https://"))
    else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    !authority.is_empty() && !authority.chars().any(char::is_whitespace)
}

fn parse_dnr_action(value: &Value, index: usize) -> Result<DnrAction, DnrError> {
    let obj = value.as_object().ok_or_else(|| DnrError::InvalidRule {
        index,
        reason: "action must be an object".to_owned(),
    })?;
    reject_unknown_dnr(obj, &["type", "redirectUrl"], index, "action")?;
    let raw_type =
        value
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| DnrError::InvalidRule {
                index,
                reason: "action.type must be a string".to_owned(),
            })?;
    let type_ = match raw_type {
        "block" => DnrActionType::Block,
        "allow" => DnrActionType::Allow,
        "redirect" => DnrActionType::Redirect,
        other => {
            return Err(DnrError::InvalidRule {
                index,
                reason: format!("unknown action type: {other}"),
            })
        }
    };
    let redirect_url = dnr_string(value, "redirectUrl", index)?;
    match type_ {
        DnrActionType::Redirect => {
            let url = redirect_url.ok_or_else(|| DnrError::InvalidRule {
                index,
                reason: "redirect action requires redirectUrl".to_owned(),
            })?;
            if !is_absolute_http_url(&url) {
                return Err(DnrError::InvalidRule {
                    index,
                    reason: format!("redirectUrl must be an absolute http(s) URL: {url}"),
                });
            }
            Ok(DnrAction {
                type_,
                redirect_url: Some(url),
            })
        }
        _ => {
            if redirect_url.is_some() {
                return Err(DnrError::InvalidRule {
                    index,
                    reason: "redirectUrl is only valid for redirect actions".to_owned(),
                });
            }
            Ok(DnrAction {
                type_,
                redirect_url: None,
            })
        }
    }
}

/// Parse a standalone declarativeNetRequest rule file (not a manifest). Rule
/// files are top-level JSON arrays of rule objects using Chrome's camelCase
/// field names. Inputs are validated fail-closed before anything is returned.
pub fn parse_dnr_rules(json: &str) -> Result<Vec<DnrRule>, DnrError> {
    let root: Value = serde_json::from_str(json).map_err(|e| DnrError::Json(e.to_string()))?;
    let entries = root.as_array().ok_or(DnrError::NotAnArray)?;
    if entries.len() > MAX_DNR_RULES {
        return Err(DnrError::TooManyRules(entries.len()));
    }

    let mut seen = HashSet::with_capacity(entries.len());
    let mut rules = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let obj = entry.as_object().ok_or_else(|| DnrError::InvalidRule {
            index,
            reason: "rule must be an object".to_owned(),
        })?;
        reject_unknown_dnr(
            obj,
            &["id", "priority", "condition", "action"],
            index,
            "rule",
        )?;
        let id = dnr_u32(entry, "id", index)?;
        if !seen.insert(id) {
            return Err(DnrError::DuplicateId(id));
        }
        // Chrome's default priority is 1 when the field is omitted.
        let priority = match entry.get("priority") {
            None => 1,
            Some(_) => dnr_u32(entry, "priority", index)?,
        };
        let condition_value = entry
            .get("condition")
            .ok_or_else(|| DnrError::InvalidRule {
                index,
                reason: "missing condition".to_owned(),
            })?;
        let action_value = entry.get("action").ok_or_else(|| DnrError::InvalidRule {
            index,
            reason: "missing action".to_owned(),
        })?;
        rules.push(DnrRule {
            id,
            priority,
            condition: parse_dnr_condition(condition_value, index)?,
            action: parse_dnr_action(action_value, index)?,
        });
    }
    Ok(rules)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHROME_MV3: &str = r#"{
        "manifest_version": 3,
        "name": "Chrome Ext",
        "version": "1.2.3",
        "key": "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A",
        "permissions": ["Storage", "tabs", 7, null],
        "host_permissions": ["https://EXAMPLE.com/*", "https://example.com/*"],
        "background": { "service_worker": "sw.js" },
        "action": { "default_title": "Chrome Action" },
        "icons": { "48": "icon48.png", "128": "icon128.png", "bad": 5 },
        "unknown_field": 42
    }"#;

    const GECKO_MV2: &str = r#"{
        "manifest_version": 2,
        "name": "Gecko Ext",
        "version": "0.1",
        "browser_specific_settings": { "gecko": { "id": "ext@example.com" } },
        "browser_action": { "default_title": "Gecko Action" },
        "background": { "scripts": ["bg.js", "polyfill.js"] },
        "permissions": ["bookmarks"]
    }"#;

    const LEGACY_APPLICATIONS: &str = r#"{
        "manifest_version": 2,
        "name": "Legacy Ext",
        "version": "1.0",
        "applications": { "gecko": { "id": "legacy@example.com" } }
    }"#;

    const MIXED_MV2: &str = r#"{
        "manifest_version": 2,
        "name": "Mixed Ext",
        "version": "1.0",
        "background": {
            "scripts": ["legacy.js"],
            "service_worker": "modern.js"
        }
    }"#;

    #[test]
    fn chrome_mv3_parses_chrome_fields() {
        let m = parse(CHROME_MV3).expect("valid chrome manifest");
        assert_eq!(m.version, ManifestVersion::V3);
        assert_eq!(m.name, "Chrome Ext");
        assert_eq!(m.ext_version, "1.2.3");
        assert_eq!(m.service_worker.as_deref(), Some("sw.js"));
        assert!(m.background_scripts.is_empty());
        assert_eq!(m.action_title.as_deref(), Some("Chrome Action"));
        assert!(m.chrome_key.is_some());
        assert_eq!(m.gecko_id, None);
        assert_eq!(flavor(&m), BrowserFlavor::Chrome);
        assert_eq!(m.icons.len(), 2);
        assert_eq!(m.icons.get("48").map(String::as_str), Some("icon48.png"));
        assert_eq!(m.permissions, vec!["storage", "tabs"]);
        assert_eq!(m.host_permissions, vec!["https://example.com/*"]);
    }

    #[test]
    fn host_permissions_never_grant_api_namespaces() {
        let m = parse(
            r#"{"manifest_version":3,"name":"x","version":"1",
                "permissions":[],
                "host_permissions":["Storage","https://EXAMPLE.com/*","storage"]}"#,
        )
        .expect("valid manifest");
        assert!(m.permissions.is_empty());
        assert_eq!(m.host_permissions, vec!["storage", "https://example.com/*"]);
    }

    #[test]
    fn gecko_mv2_parses_gecko_fields() {
        let m = parse(GECKO_MV2).expect("valid gecko manifest");
        assert_eq!(m.version, ManifestVersion::V2);
        assert_eq!(m.gecko_id.as_deref(), Some("ext@example.com"));
        assert_eq!(m.action_title.as_deref(), Some("Gecko Action"));
        assert_eq!(m.background_scripts, vec!["bg.js", "polyfill.js"]);
        assert_eq!(m.service_worker, None);
        assert_eq!(m.chrome_key, None);
        assert_eq!(flavor(&m), BrowserFlavor::Gecko);
    }

    #[test]
    fn legacy_applications_key_is_accepted() {
        let m = parse(LEGACY_APPLICATIONS).expect("valid legacy manifest");
        assert_eq!(m.gecko_id.as_deref(), Some("legacy@example.com"));
        assert_eq!(flavor(&m), BrowserFlavor::Gecko);
    }

    #[test]
    fn mixed_mv2_background_keeps_scripts_and_service_worker() {
        let m = parse(MIXED_MV2).expect("valid mixed manifest");
        assert_eq!(m.version, ManifestVersion::V2);
        assert_eq!(m.background_scripts, vec!["legacy.js"]);
        assert_eq!(m.service_worker.as_deref(), Some("modern.js"));
    }

    #[test]
    fn missing_fields_and_bad_version_are_rejected() {
        assert!(matches!(
            parse(r#"{"manifest_version":2,"version":"1"}"#),
            Err(ManifestError::Missing("name"))
        ));
        assert!(matches!(
            parse(r#"{"manifest_version":2,"name":"x"}"#),
            Err(ManifestError::Missing("version"))
        ));
        assert!(matches!(
            parse(r#"{"name":"x","version":"1"}"#),
            Err(ManifestError::UnsupportedVersion(0))
        ));
        assert!(matches!(
            parse(r#"{"manifest_version":4,"name":"x","version":"1"}"#),
            Err(ManifestError::UnsupportedVersion(4))
        ));
        assert!(matches!(parse("not json"), Err(ManifestError::Json(_))));
    }

    #[test]
    fn flavor_is_generic_without_browser_markers() {
        let m = parse(r#"{"manifest_version":3,"name":"x","version":"1"}"#)
            .expect("valid generic manifest");
        assert_eq!(flavor(&m), BrowserFlavor::Generic);
        assert!(m.permissions.is_empty());
        assert!(m.icons.is_empty());
    }

    #[test]
    fn parse_keeps_default_locale_and_description() {
        let m = parse(
            r#"{"manifest_version":3,"name":"x","version":"1",
                "default_locale":"uk-UA","description":"desc"}"#,
        )
        .expect("valid manifest");
        assert_eq!(m.default_locale.as_deref(), Some("uk-UA"));
        assert_eq!(m.description.as_deref(), Some("desc"));
        assert_eq!(m.resolved_name, None);
    }

    const MV3_POPUP: &str = r#"{
        "manifest_version": 3,
        "name": "Popup",
        "version": "1",
        "action": {
            "default_title": "Popup Title",
            "default_popup": "popup.html",
            "default_icon": {
                "16": "icons/16.png",
                "48": "icons/48.png",
                "128": "icons/128.png",
                "bad": 5
            }
        }
    }"#;

    #[test]
    fn chrome_mv3_popup_parses_and_picks_largest_icon() {
        let m = parse(MV3_POPUP).expect("valid mv3 popup manifest");
        assert_eq!(m.popup_path.as_deref(), Some("popup.html"));
        assert_eq!(m.popup_icon, Some((128, "icons/128.png".to_owned())));
        assert_eq!(m.action_title.as_deref(), Some("Popup Title"));
    }

    #[test]
    fn gecko_mv2_browser_action_popup_parses_single_icon() {
        let m = parse(
            r#"{"manifest_version":2,"name":"x","version":"1",
                "browser_action":{
                    "default_popup":"panel/popup.html",
                    "default_icon":"icons/icon.png"
                }}"#,
        )
        .expect("valid mv2 popup manifest");
        assert_eq!(m.popup_path.as_deref(), Some("panel/popup.html"));
        // Single-string form declares no size; 0 is the documented sentinel.
        assert_eq!(m.popup_icon, Some((0, "icons/icon.png".to_owned())));
    }

    #[test]
    fn missing_action_leaves_popup_fields_none() {
        for json in [
            r#"{"manifest_version":3,"name":"x","version":"1"}"#,
            r#"{"manifest_version":2,"name":"x","version":"1",
                "browser_action":{"default_title":"no popup here"}}"#,
        ] {
            let m = parse(json).expect("valid manifest");
            assert_eq!(m.popup_path, None);
            assert_eq!(m.popup_icon, None);
        }
    }

    #[test]
    fn invalid_popup_paths_are_rejected() {
        for bad in [
            "javascript:alert(1)",
            "data:text/html,<script>",
            "http://example.com/popup.html",
            "https://example.com/popup.html",
            "/etc/passwd",
            "\\server\\share\\popup.html",
            "C:\\Windows\\popup.html",
            "d:/popup.html",
            "..\\popup.html",
            "icons/../../popup.html",
            "",
        ] {
            let json = format!(
                r#"{{"manifest_version":3,"name":"x","version":"1",
                    "action":{{"default_popup":{}}}}}"#,
                serde_json::to_string(bad).expect("encode path")
            );
            assert!(
                matches!(parse(&json), Err(ManifestError::InvalidPath(_))),
                "popup path should be rejected: {bad:?}"
            );
        }
    }

    #[test]
    fn invalid_popup_icon_paths_are_rejected() {
        for bad in [
            "javascript:x",
            "data:x",
            "https://x/p.png",
            "/p.png",
            "../p.png",
        ] {
            for field in [
                format!(r#""{bad}""#),
                format!(
                    r#"{{"16": {}}}"#,
                    serde_json::to_string(bad).expect("encode path")
                ),
            ] {
                let json = format!(
                    r#"{{"manifest_version":3,"name":"x","version":"1",
                        "action":{{"default_icon":{field}}}}}"#
                );
                assert!(
                    matches!(parse(&json), Err(ManifestError::InvalidPath(_))),
                    "icon path should be rejected: {field}"
                );
            }
        }
    }

    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TempDir(PathBuf);
    impl TempDir {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "hodo-manifest-i18n-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create temp dir");
            Self(path)
        }

        fn write_extension(&self, manifest: &str, messages: &str) {
            fs::write(self.0.join("manifest.json"), manifest).expect("write manifest");
            let locale = self.0.join("_locales").join("en");
            fs::create_dir_all(&locale).expect("create locale dir");
            fs::write(locale.join("messages.json"), messages).expect("write messages");
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const MESSAGES: &str = r#"{
        "appName": { "message": "Localized Name" },
        "appDesc": { "message": "Localized Description" }
    }"#;

    #[test]
    fn msg_tokens_resolve_in_chrome_mv3_manifest() {
        let dir = TempDir::new();
        let manifest = r#"{
            "manifest_version": 3,
            "name": "__MSG_appName__",
            "description": "__MSG_appDesc__",
            "version": "1.0",
            "default_locale": "en",
            "action": { "default_title": "__MSG_appName__" }
        }"#;
        dir.write_extension(manifest, MESSAGES);

        let mut m = parse(manifest).expect("valid manifest");
        assert_eq!(m.name, "__MSG_appName__"); // parse stays raw
        resolve_messages_with_ui(&dir.0, &mut m, "en").expect("resolve");
        assert_eq!(m.name, "Localized Name");
        assert_eq!(m.resolved_name.as_deref(), Some("Localized Name"));
        assert_eq!(m.description.as_deref(), Some("Localized Description"));
        assert_eq!(m.action_title.as_deref(), Some("Localized Name"));
    }

    #[test]
    fn msg_tokens_resolve_in_gecko_mv2_manifest() {
        let dir = TempDir::new();
        let manifest = r#"{
            "manifest_version": 2,
            "name": "__MSG_appName__",
            "description": "__MSG_appDesc__",
            "version": "1.0",
            "default_locale": "en",
            "applications": { "gecko": { "id": "gecko@example.com" } },
            "browser_action": { "default_title": "__MSG_appName__" }
        }"#;
        dir.write_extension(manifest, MESSAGES);

        let mut m = parse(manifest).expect("valid manifest");
        resolve_messages_with_ui(&dir.0, &mut m, "en").expect("resolve");
        assert_eq!(m.name, "Localized Name");
        assert_eq!(m.description.as_deref(), Some("Localized Description"));
        assert_eq!(m.action_title.as_deref(), Some("Localized Name"));
        assert_eq!(flavor(&m), BrowserFlavor::Gecko);
    }

    #[test]
    fn i18n_gecko_demo_resolves_to_i18n_demo() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../extensions/i18n-gecko");
        let json = fs::read_to_string(root.join("manifest.json")).expect("demo manifest");
        let mut m = parse(&json).expect("valid demo manifest");
        resolve_messages_with_ui(&root, &mut m, "en").expect("resolve demo");
        assert_eq!(m.name, "i18n Demo");
        assert_eq!(m.resolved_name.as_deref(), Some("i18n Demo"));
        assert_eq!(m.action_title.as_deref(), Some("i18n Demo"));
    }

    #[test]
    fn resolve_without_locales_leaves_plain_fields_untouched() {
        let dir = TempDir::new();
        let manifest = r#"{"manifest_version":3,"name":"Plain Name","version":"1"}"#;
        fs::write(dir.0.join("manifest.json"), manifest).expect("write manifest");
        let mut m = parse(manifest).expect("valid manifest");
        resolve_messages_with_ui(&dir.0, &mut m, "de-DE").expect("resolve");
        assert_eq!(m.name, "Plain Name");
        assert_eq!(m.resolved_name.as_deref(), Some("Plain Name"));
    }

    #[test]
    fn msg_tokens_are_not_substituted_into_popup_paths() {
        let dir = TempDir::new();
        let manifest = r#"{
            "manifest_version": 3,
            "name": "__MSG_appName__",
            "version": "1",
            "default_locale": "en",
            "action": {
                "default_popup": "__MSG_appName__",
                "default_icon": { "16": "__MSG_appDesc__" }
            }
        }"#;
        let messages = r#"{
            "appName": { "message": "Localized Name" },
            "appDesc": { "message": "Localized Description" }
        }"#;
        dir.write_extension(manifest, messages);

        let mut m = parse(manifest).expect("valid manifest");
        resolve_messages_with_ui(&dir.0, &mut m, "en").expect("resolve");
        assert_eq!(m.name, "Localized Name");
        // Paths stay raw: they are file names, not localizable display text.
        assert_eq!(m.popup_path.as_deref(), Some("__MSG_appName__"));
        assert_eq!(m.popup_icon, Some((16, "__MSG_appDesc__".to_owned())));
    }

    #[test]
    fn popup_chrome_fixture_parses_popup_fields() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../extensions/popup-chrome");
        let json = fs::read_to_string(root.join("manifest.json")).expect("popup demo manifest");
        let m = parse(&json).expect("valid popup demo manifest");
        assert_eq!(m.version, ManifestVersion::V3);
        assert_eq!(m.popup_path.as_deref(), Some("popup.html"));
        // The demo declares its PNGs via top-level `icons`, not default_icon.
        assert_eq!(m.popup_icon, None);
        assert_eq!(m.icons.get("16").map(String::as_str), Some("icons/16.png"));
        assert_eq!(m.icons.get("48").map(String::as_str), Some("icons/48.png"));
        assert_eq!(m.action_title, None);
    }

    #[test]
    fn all_extension_fixtures_keep_popup_defaults() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../extensions");
        let mut checked = 0usize;
        for entry in fs::read_dir(&root).expect("extensions dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path().join("manifest.json");
            if !path.is_file() {
                continue;
            }
            let json = fs::read_to_string(&path).expect("fixture manifest");
            let m = parse(&json).expect("fixture parses");
            if entry.file_name().to_string_lossy() == "popup-chrome" {
                assert_eq!(m.popup_path.as_deref(), Some("popup.html"));
                assert_eq!(m.popup_icon, None);
            } else {
                assert_eq!(m.popup_path, None, "{:?}", entry.file_name());
                assert_eq!(m.popup_icon, None, "{:?}", entry.file_name());
            }
            // None of the shipped fixtures declare content scripts.
            assert!(
                m.content_scripts.is_empty(),
                "{:?} gained content_scripts",
                entry.file_name()
            );
            checked += 1;
        }
        assert!(
            checked >= 10,
            "expected extension fixtures, found {checked}"
        );
    }

    fn with_content_scripts(version: u8, entries: &str) -> String {
        format!(
            r#"{{"manifest_version":{version},"name":"x","version":"1",
                "content_scripts":{entries}}}"#
        )
    }

    #[test]
    fn mv3_content_scripts_parse_all_fields() {
        let json = with_content_scripts(
            3,
            r#"[{
                "matches": ["https://example.com/*", "https://*.example.org/foo/*", "*://*/*", "file:///tmp/*"],
                "js": ["content.js", "js\\other.js"],
                "css": ["style.css"],
                "run_at": "document_start"
            }]"#,
        );
        let m = parse(&json).expect("valid content_scripts manifest");
        assert_eq!(m.content_scripts.len(), 1);
        let cs = &m.content_scripts[0];
        assert_eq!(
            cs.matches,
            vec![
                MatchPattern {
                    scheme: Scheme::Https,
                    host: "example.com".to_owned(),
                    path: "/*".to_owned(),
                },
                MatchPattern {
                    scheme: Scheme::Https,
                    host: "*.example.org".to_owned(),
                    path: "/foo/*".to_owned(),
                },
                MatchPattern {
                    scheme: Scheme::Any,
                    host: "*".to_owned(),
                    path: "/*".to_owned(),
                },
                MatchPattern {
                    scheme: Scheme::File,
                    host: String::new(),
                    path: "/tmp/*".to_owned(),
                },
            ]
        );
        assert_eq!(cs.js, vec!["content.js", "js\\other.js"]);
        assert_eq!(cs.css, vec!["style.css"]);
        assert_eq!(cs.run_at, RunAt::DocumentStart);
    }

    #[test]
    fn mv2_top_level_content_scripts_are_accepted() {
        let json = with_content_scripts(
            2,
            r#"[{"matches": ["http://example.com/*"], "css": ["legacy.css"],
                "run_at": "document_end"}]"#,
        );
        let m = parse(&json).expect("valid mv2 content_scripts manifest");
        assert_eq!(m.version, ManifestVersion::V2);
        assert_eq!(m.content_scripts.len(), 1);
        assert_eq!(m.content_scripts[0].matches[0].scheme, Scheme::Http);
        assert_eq!(m.content_scripts[0].css, vec!["legacy.css"]);
        assert_eq!(m.content_scripts[0].run_at, RunAt::DocumentEnd);
    }

    #[test]
    fn run_at_defaults_to_document_idle() {
        let json = with_content_scripts(3, r#"[{"matches": ["https://x/*"]}]"#);
        let m = parse(&json).expect("valid manifest");
        assert_eq!(m.content_scripts[0].run_at, RunAt::DocumentIdle);

        let explicit = with_content_scripts(
            3,
            r#"[{"matches": ["https://x/*"], "run_at": "document_idle"}]"#,
        );
        let m = parse(&explicit).expect("valid manifest");
        assert_eq!(m.content_scripts[0].run_at, RunAt::DocumentIdle);
    }

    #[test]
    fn unknown_run_at_is_rejected() {
        let json =
            with_content_scripts(3, r#"[{"matches": ["https://x/*"], "run_at": "whenever"}]"#);
        assert!(matches!(
            parse(&json),
            Err(ManifestError::InvalidContentScript(_))
        ));
    }

    fn pattern_in_entry(pattern: &str) -> String {
        with_content_scripts(
            3,
            &format!(
                r#"[{{"matches": [{}]}}]"#,
                serde_json::to_string(pattern).expect("encode pattern")
            ),
        )
    }

    #[test]
    fn pattern_rejects_empty() {
        assert!(matches!(
            parse(&pattern_in_entry("")),
            Err(ManifestError::InvalidMatchPattern(_))
        ));
    }

    #[test]
    fn pattern_rejects_missing_scheme() {
        assert!(matches!(
            parse(&pattern_in_entry("example.com/*")),
            Err(ManifestError::InvalidMatchPattern(_))
        ));
    }

    #[test]
    fn pattern_rejects_unsupported_scheme() {
        assert!(matches!(
            parse(&pattern_in_entry("ftp://example.com/*")),
            Err(ManifestError::InvalidMatchPattern(_))
        ));
    }

    #[test]
    fn pattern_rejects_double_star() {
        for bad in ["https://example.com/**", "https://**.example.com/*"] {
            assert!(
                matches!(
                    parse(&pattern_in_entry(bad)),
                    Err(ManifestError::InvalidMatchPattern(_))
                ),
                "double star should be rejected: {bad}"
            );
        }
    }

    #[test]
    fn pattern_rejects_ports_with_clear_error() {
        assert!(matches!(
            parse(&pattern_in_entry("https://example.com:8443/*")),
            Err(ManifestError::MatchPatternPort(_))
        ));
    }

    #[test]
    fn pattern_rejects_missing_path() {
        assert!(matches!(
            parse(&pattern_in_entry("https://example.com")),
            Err(ManifestError::InvalidMatchPattern(_))
        ));
    }

    #[test]
    fn pattern_rejects_malformed_wildcard_host() {
        for bad in [
            "https://*./*",
            "https://*..example.com/*",
            "https://exa mple.com/*",
        ] {
            assert!(
                matches!(
                    parse(&pattern_in_entry(bad)),
                    Err(ManifestError::InvalidMatchPattern(_))
                ),
                "bad host should be rejected: {bad}"
            );
        }
    }

    #[test]
    fn content_script_paths_follow_popup_rules() {
        for bad in [
            "javascript:alert(1)",
            "data:text/html,<script>",
            "http://example.com/x.js",
            "https://example.com/x.css",
            "/abs/x.js",
            "\\\\server\\share\\x.js",
            "C:\\Windows\\x.js",
            "d:/x.js",
            "..\\x.js",
            "js/../../x.js",
            "",
        ] {
            for field in ["js", "css"] {
                let json = with_content_scripts(
                    3,
                    &format!(
                        r#"[{{"matches": ["https://x/*"], "{field}": [{}]}}]"#,
                        serde_json::to_string(bad).expect("encode path")
                    ),
                );
                assert!(
                    matches!(parse(&json), Err(ManifestError::InvalidPath(_))),
                    "{field} path should be rejected: {bad:?}"
                );
            }
        }
    }

    #[test]
    fn msg_tokens_are_not_substituted_into_content_script_paths() {
        let json = with_content_scripts(
            3,
            r#"[{"matches": ["https://x/*"], "js": ["__MSG_jsFile__"],
                "css": ["__MSG_cssFile__"]}]"#,
        );
        let dir = TempDir::new();
        fs::write(dir.0.join("manifest.json"), &json).expect("write manifest");
        let mut m = parse(&json).expect("valid manifest");
        resolve_messages_with_ui(&dir.0, &mut m, "en").expect("resolve without locales");
        assert_eq!(m.content_scripts[0].js, vec!["__MSG_jsFile__"]);
        assert_eq!(m.content_scripts[0].css, vec!["__MSG_cssFile__"]);
    }

    #[test]
    fn matches_url_literal_host_and_scheme() {
        let p = MatchPattern::parse("https://example.com/*").expect("valid pattern");
        assert!(p.matches_url("https://example.com/a"));
        assert!(p.matches_url("HTTPS://EXAMPLE.COM/a"));
        assert!(!p.matches_url("http://example.com/a"));
        assert!(!p.matches_url("https://other.com/a"));
        assert!(!p.matches_url("ftp://example.com/a"));
    }

    #[test]
    fn matches_url_subdomain_wildcard() {
        let p = MatchPattern::parse("https://*.example.com/*").expect("valid pattern");
        assert!(p.matches_url("https://example.com/a"));
        assert!(p.matches_url("https://sub.example.com/a"));
        assert!(p.matches_url("https://a.b.example.com/a"));
        assert!(!p.matches_url("https://example.org/a"));
        assert!(!p.matches_url("https://notexample.com/a"));
    }

    #[test]
    fn matches_url_host_and_scheme_all() {
        let p = MatchPattern::parse("*://*/*").expect("valid pattern");
        assert!(p.matches_url("https://example.com/a"));
        assert!(p.matches_url("http://anything.org/x"));
        assert!(p.matches_url("file:///tmp/a.js"));
        assert!(!p.matches_url("ftp://example.com/a"));

        let file = MatchPattern::parse("file:///tmp/*").expect("valid pattern");
        assert!(file.matches_url("file:///tmp/a.js"));
        assert!(!file.matches_url("https://host/tmp/a.js"));
    }

    #[test]
    fn matches_url_path_globs() {
        let prefix = MatchPattern::parse("https://example.com/foo/*").expect("valid pattern");
        assert!(prefix.matches_url("https://example.com/foo/bar"));
        assert!(!prefix.matches_url("https://example.com/foobar"));
        assert!(!prefix.matches_url("https://example.com/bar/foo"));

        let middle = MatchPattern::parse("https://example.com/*/config").expect("valid pattern");
        assert!(middle.matches_url("https://example.com/a/config"));
        assert!(middle.matches_url("https://example.com/a/b/config"));
        assert!(!middle.matches_url("https://example.com/a/settings"));

        let exact = MatchPattern::parse("https://example.com/a.js").expect("valid pattern");
        assert!(exact.matches_url("https://example.com/a.js"));
        assert!(!exact.matches_url("https://example.com/a.js.map"));
    }

    #[test]
    fn matches_url_ignores_port_query_and_fragment() {
        let p = MatchPattern::parse("https://example.com/foo/*").expect("valid pattern");
        assert!(p.matches_url("https://example.com:8443/foo/bar?q=1#frag"));
    }

    fn many_entries(count: usize) -> String {
        let entry = r#"{"matches": ["https://x/*"]}"#;
        with_content_scripts(3, &format!("[{}]", [entry].repeat(count).join(",")))
    }

    #[test]
    fn content_script_count_cap_is_fail_closed() {
        let at_cap = parse(&many_entries(MAX_CONTENT_SCRIPTS)).expect("64 entries allowed");
        assert_eq!(at_cap.content_scripts.len(), MAX_CONTENT_SCRIPTS);
        assert!(matches!(
            parse(&many_entries(MAX_CONTENT_SCRIPTS + 1)),
            Err(ManifestError::TooManyContentScripts(65))
        ));
    }

    #[test]
    fn content_script_file_cap_is_fail_closed() {
        for field in ["js", "css"] {
            let files = format!("[{}]", [r#""f.js""#].repeat(MAX_SCRIPT_FILES).join(","));
            let at_cap = with_content_scripts(
                3,
                &format!(r#"[{{"matches": ["https://x/*"], "{field}": {files}}}]"#),
            );
            assert_eq!(
                parse(&at_cap).expect("64 files allowed").content_scripts[0]
                    .matches
                    .len(),
                1
            );

            let too_many = format!("[{}]", [r#""f.js""#].repeat(MAX_SCRIPT_FILES + 1).join(","));
            let json = with_content_scripts(
                3,
                &format!(r#"[{{"matches": ["https://x/*"], "{field}": {too_many}}}]"#),
            );
            assert!(
                matches!(
                    parse(&json),
                    Err(ManifestError::TooManyContentScriptFiles(ref key, 65)) if key == field
                ),
                "{field} cap should be enforced"
            );
        }
    }

    // ----- declarative net request: manifest declaration -----

    fn with_dnr(permissions: &str, resources: &str) -> String {
        format!(
            r#"{{"manifest_version":3,"name":"x","version":"1",
                "permissions":{permissions},
                "declarative_net_request":{{"rule_resources":{resources}}}}}"#
        )
    }

    #[test]
    fn dnr_manifest_resources_parse() {
        let m = parse(&with_dnr(
            r#"["Tabs","declarativeNetRequest"]"#,
            r#"[{"id":1,"enabled":true,"path":"rules/ads.json"},
                {"id":2,"enabled":false,"path":"rules/trackers.json"}]"#,
        ))
        .expect("valid dnr manifest");
        assert_eq!(
            m.declarative_net_request,
            vec![
                DnrRuleResource {
                    id: 1,
                    enabled: true,
                    path: "rules/ads.json".to_owned(),
                },
                DnrRuleResource {
                    id: 2,
                    enabled: false,
                    path: "rules/trackers.json".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn dnr_manifest_without_resources_needs_no_permission() {
        let m = parse(
            r#"{"manifest_version":3,"name":"x","version":"1",
                "declarative_net_request":{"rule_resources":[]}}"#,
        )
        .expect("empty resource list needs no gate");
        assert!(m.declarative_net_request.is_empty());
    }

    #[test]
    fn dnr_manifest_resources_without_permission_are_rejected() {
        for permissions in ["[]", r#"["tabs","storage"]"#] {
            let json = with_dnr(
                permissions,
                r#"[{"id":1,"enabled":true,"path":"rules.json"}]"#,
            );
            assert!(
                matches!(parse(&json), Err(ManifestError::DnrMissingPermission)),
                "declared resources require a DNR permission: {permissions}"
            );
        }
    }

    #[test]
    fn dnr_manifest_accepts_either_dnr_permission() {
        for permission in ["declarativeNetRequest", "declarativeNetRequestFeedback"] {
            let json = with_dnr(
                &format!(r#"["{permission}"]"#),
                r#"[{"id":1,"enabled":true,"path":"rules.json"}]"#,
            );
            assert!(parse(&json).is_ok(), "{permission} should satisfy the gate");
        }
    }

    #[test]
    fn dnr_manifest_resource_paths_follow_popup_matrix() {
        for bad in [
            "javascript:alert(1)",
            "data:text/html,<script>",
            "http://example.com/rules.json",
            "https://example.com/rules.json",
            "/etc/passwd",
            "\\server\\share\\rules.json",
            "C:\\Windows\\rules.json",
            "d:/rules.json",
            "..\\rules.json",
            "rules/../../rules.json",
            "",
        ] {
            let json = with_dnr(
                r#"["declarativeNetRequest"]"#,
                &format!(
                    r#"[{{"id":1,"enabled":true,"path":{}}}]"#,
                    serde_json::to_string(bad).expect("encode path")
                ),
            );
            assert!(
                matches!(parse(&json), Err(ManifestError::InvalidPath(_))),
                "DNR resource path should be rejected: {bad:?}"
            );
        }
    }

    #[test]
    fn dnr_manifest_rejects_unknown_fields_and_bad_shapes() {
        let prefix = r#"{"manifest_version":3,"name":"x","version":"1",
            "permissions":["declarativeNetRequest"],"declarative_net_request":"#;
        for (dnr, label) in [
            (r#"{"rule_resources":[],"extra":1}"#, "unknown outer field"),
            (
                r#"[{"id":1,"enabled":true,"path":"r.json","extra":1}]"#,
                "unknown resource field",
            ),
            (
                r#"[{"id":-1,"enabled":true,"path":"r.json"}]"#,
                "negative id",
            ),
            (
                r#"[{"id":1,"enabled":"yes","path":"r.json"}]"#,
                "non-bool enabled",
            ),
            (r#"[{"id":1,"enabled":true}]"#, "missing path"),
            (r#"{"rule_resources":"rules.json"}"#, "non-array resources"),
        ] {
            let json = format!("{prefix}{dnr}}}");
            assert!(
                matches!(parse(&json), Err(ManifestError::InvalidDnr(_))),
                "{label} should fail: {json}"
            );
        }
        let duplicate = with_dnr(
            r#"["declarativeNetRequest"]"#,
            r#"[{"id":1,"enabled":true,"path":"a.json"},
                {"id":1,"enabled":true,"path":"b.json"}]"#,
        );
        assert!(matches!(
            parse(&duplicate),
            Err(ManifestError::InvalidDnr(_))
        ));
    }

    // ----- declarative net request: standalone rule files -----

    fn dnr_rules(json: &str) -> String {
        format!("[{json}]")
    }

    const MINIMAL_RULE: &str =
        r#"{"id":1,"condition":{"urlFilter":"ads"},"action":{"type":"block"}}"#;

    #[test]
    fn dnr_minimal_rule_file_parses() {
        let parsed = parse_dnr_rules(&dnr_rules(MINIMAL_RULE)).expect("minimal rule file");
        assert_eq!(parsed.len(), 1);
        let rule = &parsed[0];
        assert_eq!(rule.id, 1);
        assert_eq!(rule.priority, 1); // Chrome default when omitted
        assert_eq!(rule.condition.url_filter.as_deref(), Some("ads"));
        assert_eq!(rule.condition.regex_filter, None);
        assert_eq!(rule.condition.domains, None);
        assert!(!rule.condition.is_url_filter_case_sensitive);
        assert_eq!(rule.action.type_, DnrActionType::Block);
        assert_eq!(rule.action.redirect_url, None);
    }

    #[test]
    fn dnr_camel_case_fields_parse() {
        let json = r#"[{
            "id":7,
            "priority":42,
            "condition":{
                "urlFilter":"||ads.example.com^",
                "domains":["example.com","*.example.org"],
                "initiatorDomains":["site.test"],
                "excludedDomains":["good.example.com"],
                "excludedInitiatorDomains":["internal.test"],
                "resourceTypes":["main_frame","sub_frame","stylesheet","script","image","font","object","xmlhttprequest","other"],
                "isUrlFilterCaseSensitive":true
            },
            "action":{"type":"allow"}
        }]"#;
        let parsed = parse_dnr_rules(json).expect("camelCase rule file");
        let rule = &parsed[0];
        let condition = &rule.condition;
        assert_eq!(rule.priority, 42);
        assert_eq!(condition.url_filter.as_deref(), Some("||ads.example.com^"));
        assert_eq!(
            condition.domains,
            Some(vec!["example.com".to_owned(), "*.example.org".to_owned()])
        );
        assert_eq!(
            condition.initiator_domains,
            Some(vec!["site.test".to_owned()])
        );
        assert_eq!(
            condition.excluded_domains,
            Some(vec!["good.example.com".to_owned()])
        );
        assert_eq!(
            condition.excluded_initiator_domains,
            Some(vec!["internal.test".to_owned()])
        );
        assert_eq!(
            condition.resource_types,
            Some(vec![
                DnrResourceType::MainFrame,
                DnrResourceType::SubFrame,
                DnrResourceType::StyleSheet,
                DnrResourceType::Script,
                DnrResourceType::Image,
                DnrResourceType::Font,
                DnrResourceType::Object,
                DnrResourceType::XmlHttpRequest,
                DnrResourceType::Other,
            ])
        );
        assert!(condition.is_url_filter_case_sensitive);
        assert_eq!(rule.action.type_, DnrActionType::Allow);
    }

    #[test]
    fn dnr_regex_filter_rule_parses() {
        let json = r#"[{"id":2,"condition":{"regexFilter":"^https://ads\\.example\\.com/"},
            "action":{"type":"block"}}]"#;
        let parsed = parse_dnr_rules(json).expect("regex rule file");
        assert_eq!(
            parsed[0].condition.regex_filter.as_deref(),
            Some(r#"^https://ads\.example\.com/"#)
        );
        assert_eq!(parsed[0].condition.url_filter, None);
    }

    #[test]
    fn dnr_url_and_regex_filter_conflict_is_rejected() {
        let conflict = dnr_rules(
            r#"{"id":1,"condition":{"urlFilter":"ads","regexFilter":"ads"},
                "action":{"type":"block"}}"#,
        );
        assert!(matches!(
            parse_dnr_rules(&conflict),
            Err(DnrError::InvalidRule { index: 0, .. })
        ));
        let neither = dnr_rules(r#"{"id":1,"condition":{},"action":{"type":"block"}}"#);
        assert!(parse_dnr_rules(&neither).is_err());
    }

    #[test]
    fn dnr_redirect_requires_an_absolute_http_url() {
        let missing =
            dnr_rules(r#"{"id":1,"condition":{"urlFilter":"a"},"action":{"type":"redirect"}}"#);
        assert!(parse_dnr_rules(&missing).is_err());
        for bad in [
            "/local.html",
            "ftp://example.com/x",
            "javascript:alert(1)",
            "https://",
            "not a url",
        ] {
            let json = dnr_rules(&format!(
                r#"{{"id":1,"condition":{{"urlFilter":"a"}},
                    "action":{{"type":"redirect","redirectUrl":{}}}}}"#,
                serde_json::to_string(bad).expect("encode url")
            ));
            assert!(
                parse_dnr_rules(&json).is_err(),
                "bad redirectUrl should fail: {bad:?}"
            );
        }
        let good = dnr_rules(
            r#"{"id":1,"condition":{"urlFilter":"a"},
                "action":{"type":"redirect","redirectUrl":"https://example.com/blocked.html"}}"#,
        );
        let parsed = parse_dnr_rules(&good).expect("valid redirect rule");
        assert_eq!(parsed[0].action.type_, DnrActionType::Redirect);
        assert_eq!(
            parsed[0].action.redirect_url.as_deref(),
            Some("https://example.com/blocked.html")
        );
    }

    #[test]
    fn dnr_block_and_allow_reject_redirect_url() {
        for action in ["block", "allow"] {
            let json = dnr_rules(&format!(
                r#"{{"id":1,"condition":{{"urlFilter":"a"}},
                    "action":{{"type":"{action}","redirectUrl":"https://x/"}}}}"#
            ));
            assert!(
                parse_dnr_rules(&json).is_err(),
                "{action} should not carry redirectUrl"
            );
        }
    }

    #[test]
    fn dnr_unknown_fields_are_rejected() {
        for (rule, context) in [
            (
                r#"{"id":1,"priority":1,"condition":{"urlFilter":"a"},
                    "action":{"type":"block"},"extra":1}"#,
                "rule",
            ),
            (
                r#"{"id":1,"condition":{"urlFilter":"a","bogus":true},
                    "action":{"type":"block"}}"#,
                "condition",
            ),
            (
                r#"{"id":1,"condition":{"urlFilter":"a"},
                    "action":{"type":"block","bogus":1}}"#,
                "action",
            ),
        ] {
            assert!(
                matches!(
                    parse_dnr_rules(&dnr_rules(rule)),
                    Err(DnrError::UnknownField {
                        context: actual, ..
                    }) if actual == context
                ),
                "unknown {context} field should fail: {rule}"
            );
        }
    }

    #[test]
    fn dnr_unknown_resource_type_is_rejected() {
        let json = dnr_rules(
            r#"{"id":1,"condition":{"urlFilter":"a","resourceTypes":["main_frame","bogus"]},
                "action":{"type":"block"}}"#,
        );
        assert!(matches!(
            parse_dnr_rules(&json),
            Err(DnrError::InvalidRule { .. })
        ));
    }

    fn many_dnr_rules(count: usize) -> String {
        let entries: Vec<String> = (0..count)
            .map(|id| {
                format!(
                    r#"{{"id":{id},"condition":{{"urlFilter":"a"}},"action":{{"type":"block"}}}}"#
                )
            })
            .collect();
        format!("[{}]", entries.join(","))
    }

    #[test]
    fn dnr_rule_count_cap_is_fail_closed() {
        let at_cap = parse_dnr_rules(&many_dnr_rules(MAX_DNR_RULES)).expect("30,000 rules allowed");
        assert_eq!(at_cap.len(), MAX_DNR_RULES);
        assert!(matches!(
            parse_dnr_rules(&many_dnr_rules(MAX_DNR_RULES + 1)),
            Err(DnrError::TooManyRules(30_001))
        ));
    }

    fn dnr_rule_with_regex(pattern: &str) -> String {
        format!(
            r#"[{{"id":1,"condition":{{"regexFilter":{}}},"action":{{"type":"block"}}}}]"#,
            serde_json::to_string(pattern).expect("encode regex")
        )
    }

    #[test]
    fn dnr_regex_length_cap_is_fail_closed() {
        let at_cap = dnr_rule_with_regex(&"a".repeat(MAX_DNR_REGEX_LEN));
        assert!(parse_dnr_rules(&at_cap).is_ok());
        let too_long = dnr_rule_with_regex(&"a".repeat(MAX_DNR_REGEX_LEN + 1));
        assert!(matches!(
            parse_dnr_rules(&too_long),
            Err(DnrError::RegexTooLong(1025))
        ));
    }

    fn dnr_rule_with_domains(count: usize) -> String {
        let domains: Vec<String> = (0..count).map(|i| format!(r#""d{i}.example""#)).collect();
        format!(
            r#"[{{"id":1,"condition":{{"urlFilter":"a","domains":[{}]}},"action":{{"type":"block"}}}}]"#,
            domains.join(",")
        )
    }

    #[test]
    fn dnr_domain_list_cap_is_fail_closed() {
        let at_cap =
            parse_dnr_rules(&dnr_rule_with_domains(MAX_DNR_DOMAINS)).expect("1,000 domains");
        assert_eq!(
            at_cap[0].condition.domains.as_ref().expect("domains").len(),
            MAX_DNR_DOMAINS
        );
        let too_many = dnr_rule_with_domains(MAX_DNR_DOMAINS + 1);
        assert!(matches!(
            parse_dnr_rules(&too_many),
            Err(DnrError::TooManyDomains("domains", 1001))
        ));
    }

    #[test]
    fn dnr_duplicate_ids_are_rejected() {
        let json = format!("[{MINIMAL_RULE},{MINIMAL_RULE}]");
        assert!(matches!(
            parse_dnr_rules(&json),
            Err(DnrError::DuplicateId(1))
        ));
    }

    #[test]
    fn dnr_bad_json_and_non_array_are_rejected() {
        assert!(matches!(
            parse_dnr_rules("not json"),
            Err(DnrError::Json(_))
        ));
        assert!(matches!(
            parse_dnr_rules(r#"{"id":1}"#),
            Err(DnrError::NotAnArray)
        ));
        assert!(matches!(
            parse_dnr_rules("[42]"),
            Err(DnrError::InvalidRule { index: 0, .. })
        ));
    }
}
