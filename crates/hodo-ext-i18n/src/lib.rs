//! WebExtension i18n: `_locales/<locale>/messages.json` loading and
//! `i18n.getMessage` substitution.
//!
//! # Security notes
//!
//! - Hard caps per file (2 MiB), key count (10 000), value length (8 KiB) and
//!   key names (`[A-Za-z0-9_@]+`) keep hostile extensions from exhausting RAM.
//! - Placeholder expansion is **single-pass**: the text produced for a
//!   placeholder is never rescanned, so an expansion bomb cannot recurse.
//! - Messages are always plain text, never HTML.
//! - RTL scripts are not detected in this wave: all locales get LTR bidi
//!   values. RTL detection is a follow-up (see [`predefined_value`]).

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::Value;

/// 2 MiB per resource, as in the rest of hodo.
pub const MAX_FILE_BYTES: u64 = hodo_types::limits::MAX_LOCALE_BYTES as u64;
pub const MAX_KEYS: usize = 10_000;
pub const MAX_VALUE_BYTES: usize = 8 * 1024;
pub const MAX_KEY_BYTES: usize = 64;
/// `i18n.getMessage` accepts at most nine substitutions.
pub const MAX_SUBSTITUTIONS: usize = 9;

#[derive(Debug, thiserror::Error)]
pub enum I18nError {
    #[error("_locales exists but the manifest has no default_locale")]
    MissingDefaultLocale,
    #[error("cannot read {0}: {1}")]
    Io(PathBuf, String),
    #[error("invalid {0}: {1}")]
    InvalidMessages(PathBuf, String),
}

/// One entry of `messages.json`.
#[derive(Debug, Clone)]
pub struct MessageEntry {
    pub message: String,
    /// Map from `$name$` in `message` to a content template (`$1`, literal text, …).
    pub placeholders: HashMap<String, String>,
}

/// A loaded (possibly empty) message table.
#[derive(Debug)]
pub struct Messages {
    pub map: HashMap<String, MessageEntry>,
    pub default_locale: String,
    /// Locale the table was actually loaded from.
    pub locale: String,
    /// Requested UI locale, normalized (used by `@@ui_locale`).
    pub ui_locale: String,
    /// Set by the host from the manifest id; empty until then.
    pub extension_id: String,
    warned_missing: Mutex<HashSet<String>>,
}

impl Messages {
    fn empty(default_locale: &str, ui_locale: &str) -> Self {
        Self {
            map: HashMap::new(),
            default_locale: default_locale.to_owned(),
            locale: default_locale.to_owned(),
            ui_locale: ui_locale.to_owned(),
            extension_id: String::new(),
            warned_missing: Mutex::new(HashSet::new()),
        }
    }

    /// Attach the extension id used by `@@extension_id`.
    pub fn with_extension_id(mut self, id: &str) -> Self {
        self.extension_id = id.to_owned();
        self
    }

    /// `i18n.getMessage(name, substitutions)`. Extra substitutions past nine
    /// are ignored; a missing key yields `""` plus a one-time warning.
    pub fn get_message(&self, name: &str, substitutions: &[String]) -> String {
        if let Some(value) = predefined_value(name, &self.extension_id, &self.ui_locale) {
            return value;
        }
        let Some(entry) = self.map.get(name) else {
            self.warn_missing(name);
            return String::new();
        };
        let subs = &substitutions[..substitutions.len().min(MAX_SUBSTITUTIONS)];
        expand(
            &entry.message,
            |token| {
                if token.len() == 1 {
                    if let Some(index) = token.as_bytes()[0].checked_sub(b'1') {
                        if (index as usize) < MAX_SUBSTITUTIONS {
                            return subs.get(index as usize).cloned();
                        }
                    }
                }
                entry
                    .placeholders
                    .get(token)
                    .map(|content| expand_content(content, subs))
            },
            |token| predefined_value(token, &self.extension_id, &self.ui_locale),
        )
    }

    fn warn_missing(&self, name: &str) {
        let Ok(mut seen) = self.warned_missing.lock() else {
            return;
        };
        if seen.insert(name.to_owned()) {
            eprintln!(
                "hodo-ext-i18n: missing message `{name}` in locale `{}`",
                self.locale
            );
        }
    }
}

/// Predefined `@@…` messages. `@@extension_id` needs the host-provided id;
/// `@@ui_locale` uses the normalized requested locale.
///
/// shortcut: every locale is treated as LTR; detect RTL from the locale script
/// when complex-script/bidi rendering lands (M7.2).
fn predefined_value(name: &str, extension_id: &str, ui_locale: &str) -> Option<String> {
    match name {
        "@@extension_id" => Some(extension_id.to_owned()),
        "@@ui_locale" => Some(ui_locale.to_owned()),
        "@@bidi_dir" => Some("ltr".to_owned()),
        "@@bidi_reversed_dir" => Some("rtl".to_owned()),
        "@@bidi_start_edge" => Some("left".to_owned()),
        "@@bidi_end_edge" => Some("right".to_owned()),
        _ => None,
    }
}

const PREDEFINED: &[&str] = &[
    "bidi_reversed_dir",
    "bidi_start_edge",
    "bidi_end_edge",
    "extension_id",
    "ui_locale",
    "bidi_dir",
];

/// `true` for WebExtension message keys (predefined keys start with `@@`).
pub fn is_valid_message_key(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_KEY_BYTES
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'@')
}

/// `uk-UA` / `uk_UA` / `EN` → `uk-ua` / `uk-ua` / `en`.
pub fn normalize_locale(locale: &str) -> String {
    locale.trim().replace('_', "-").to_lowercase()
}

/// Reject anything that could escape `_locales/<candidate>/`.
fn is_safe_locale_component(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        && !s.starts_with('-')
        && !s.ends_with('-')
}

fn push_candidate(list: &mut Vec<String>, candidate: String) {
    if !list.contains(&candidate) {
        list.push(candidate);
    }
}

/// Load messages with the chain: normalized exact locale → language →
/// `default_locale` → default language. A broken locale file is skipped with a
/// warning; absence of `_locales` yields an empty table (never an error).
///
/// `default_locale` is mandatory as soon as `_locales` exists.
pub fn load_locales(
    ext_root: &Path,
    default_locale: Option<&str>,
    ui_locale: &str,
) -> Result<Messages, I18nError> {
    let locales_dir = ext_root.join("_locales");
    let ui_norm = normalize_locale(ui_locale);
    let default_norm = default_locale.map(normalize_locale);

    if !locales_dir.is_dir() {
        return Ok(Messages::empty(
            default_norm.as_deref().unwrap_or("en"),
            &ui_norm,
        ));
    }
    let Some(default_norm) = default_norm else {
        return Err(I18nError::MissingDefaultLocale);
    };

    let mut messages = Messages::empty(&default_norm, &ui_norm);
    let mut candidates = Vec::new();
    if is_safe_locale_component(&ui_norm) {
        push_candidate(&mut candidates, ui_norm.clone());
        if let Some(language) = ui_norm.split('-').next() {
            push_candidate(&mut candidates, language.to_owned());
        }
    }
    push_candidate(&mut candidates, default_norm.clone());
    if let Some(language) = default_norm.split('-').next() {
        push_candidate(&mut candidates, language.to_owned());
    }

    for candidate in candidates {
        if !is_safe_locale_component(&candidate) {
            continue;
        }
        let path = locales_dir.join(&candidate).join("messages.json");
        match read_messages(&path) {
            Ok(map) => {
                messages.map = map;
                messages.locale = candidate;
                return Ok(messages);
            }
            Err(error) => eprintln!("hodo-ext-i18n: skipping locale `{candidate}`: {error}"),
        }
    }
    eprintln!(
        "hodo-ext-i18n: no usable locale in `{}`; using an empty table",
        locales_dir.display()
    );
    Ok(messages)
}

fn read_messages(path: &Path) -> Result<HashMap<String, MessageEntry>, I18nError> {
    let file = File::open(path).map_err(|e| I18nError::Io(path.to_owned(), e.to_string()))?;
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| I18nError::Io(path.to_owned(), e.to_string()))?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(I18nError::InvalidMessages(
            path.to_owned(),
            "file exceeds 2 MiB".to_owned(),
        ));
    }
    let root: Value = serde_json::from_slice(&bytes)
        .map_err(|e| I18nError::InvalidMessages(path.to_owned(), e.to_string()))?;
    let object = root.as_object().ok_or_else(|| {
        I18nError::InvalidMessages(path.to_owned(), "top level must be an object".to_owned())
    })?;
    if object.len() > MAX_KEYS {
        return Err(I18nError::InvalidMessages(
            path.to_owned(),
            "too many keys".to_owned(),
        ));
    }

    let mut map = HashMap::with_capacity(object.len());
    for (key, value) in object {
        let invalid = |reason: &str| {
            I18nError::InvalidMessages(path.to_owned(), format!("`{key}`: {reason}"))
        };
        if !is_valid_message_key(key) {
            return Err(invalid("invalid key name"));
        }
        let entry = value
            .as_object()
            .ok_or_else(|| invalid("entry must be an object"))?;
        let message = entry
            .get("message")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("missing string `message`"))?;
        if message.len() > MAX_VALUE_BYTES {
            return Err(invalid("message too long"));
        }

        let mut placeholders = HashMap::new();
        if let Some(definitions) = entry.get("placeholders") {
            let definitions = definitions
                .as_object()
                .ok_or_else(|| invalid("`placeholders` must be an object"))?;
            if definitions.len() > MAX_KEYS {
                return Err(invalid("too many placeholders"));
            }
            for (name, definition) in definitions {
                if !is_valid_message_key(name) {
                    return Err(invalid("invalid placeholder name"));
                }
                let content = definition
                    .get("content")
                    .and_then(Value::as_str)
                    .ok_or_else(|| invalid("placeholder has no string `content`"))?;
                if content.len() > MAX_VALUE_BYTES {
                    return Err(invalid("placeholder content too long"));
                }
                placeholders.insert(name.clone(), content.to_owned());
            }
        }
        map.insert(
            key.clone(),
            MessageEntry {
                message: message.to_owned(),
                placeholders,
            },
        );
    }
    Ok(map)
}

/// Expand one template in a single pass: `@@…` predefined tokens, `$$` → `$`,
/// and `$name$` resolved by `resolve`. Inserted text is never rescanned.
fn expand(
    text: &str,
    resolve: impl Fn(&str) -> Option<String>,
    resolve_special: impl Fn(&str) -> Option<String>,
) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'@' && bytes.get(i + 1) == Some(&b'@') {
            if let Some(name) = PREDEFINED.iter().find(|name| {
                let rest = &text[i + 2..];
                rest.starts_with(**name)
                    // A predefined token must end at a boundary. Without this,
                    // `@@bidi_direction` matches the `bidi_dir` prefix and is
                    // mangled into "ltr" + "ection".
                    && rest[name.len()..].chars().next().is_none_or(|ch| {
                        !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
                    })
            }) {
                let end = i + 2 + name.len();
                if let Some(value) = resolve_special(&text[i..end]) {
                    out.push_str(&value);
                }
                i = end;
                continue;
            }
        }
        if bytes[i] == b'$' {
            if bytes.get(i + 1) == Some(&b'$') {
                out.push('$');
                i += 2;
                continue;
            }
            if let Some(offset) = text[i + 1..].find('$') {
                let end = i + 1 + offset + 1;
                let name = &text[i + 1..end - 1];
                if is_valid_message_key(name) {
                    if let Some(value) = resolve(name) {
                        out.push_str(&value);
                    }
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

/// Expand a placeholder's `content`: `$$` → `$`, `$1`–`$9` → substitution.
/// Deliberately does not resolve `$name$` — that would be a second pass.
fn expand_content(content: &str, substitutions: &[String]) -> String {
    let bytes = content.as_bytes();
    let mut out = String::with_capacity(content.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            if bytes.get(i + 1) == Some(&b'$') {
                out.push('$');
                i += 2;
                continue;
            }
            if let Some(index) = bytes.get(i + 1).and_then(|b| b.checked_sub(b'1')) {
                if (index as usize) < MAX_SUBSTITUTIONS {
                    if let Some(value) = substitutions.get(index as usize) {
                        out.push_str(value);
                    }
                    i += 2;
                    continue;
                }
            }
        }
        match content[i..].chars().next() {
            Some(ch) => {
                out.push(ch);
                i += ch.len_utf8();
            }
            None => break,
        }
    }
    out
}

/// Current UI language from the OS, normalized; `en` if unavailable.
pub fn ui_locale() -> String {
    sys_locale::get_locale()
        .map(|locale| normalize_locale(&locale))
        .filter(|locale| is_safe_locale_component(locale))
        .unwrap_or_else(|| "en".to_owned())
}

/// OS-preferred languages, normalized, with `en` always present as fallback.
pub fn get_accept_languages() -> Vec<String> {
    let mut languages: Vec<String> = sys_locale::get_locales()
        .map(|locale| normalize_locale(&locale))
        .filter(|locale| is_safe_locale_component(locale))
        .collect();
    if !languages.iter().any(|locale| locale == "en") {
        languages.push("en".to_owned());
    }
    languages
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TempDir(PathBuf);
    impl TempDir {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "hodo-i18n-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create temp dir");
            Self(path)
        }

        fn write_locale(&self, locale: &str, json: &str) {
            let dir = self.0.join("_locales").join(locale);
            fs::create_dir_all(&dir).expect("create locale dir");
            fs::write(dir.join("messages.json"), json).expect("write messages.json");
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn msg(text: &str) -> String {
        format!(r#"{{"k":{{"message":"{text}"}}}}"#)
    }

    #[test]
    fn normalize_locale_handles_dashes_underscores_and_case() {
        assert_eq!(normalize_locale("uk-UA"), "uk-ua");
        assert_eq!(normalize_locale("uk_UA"), "uk-ua");
        assert_eq!(normalize_locale("EN"), "en");
    }

    #[test]
    fn fallback_exact_then_language_then_default() {
        let dir = TempDir::new();
        dir.write_locale("en", &msg("default"));
        dir.write_locale("uk", &msg("language"));
        dir.write_locale("uk-ua", &msg("exact"));

        let load = |ui: &str| {
            load_locales(&dir.0, Some("en"), ui)
                .expect("locales load")
                .get_message("k", &[])
        };
        assert_eq!(load("uk-UA"), "exact");
        assert_eq!(load("uk_UA"), "exact");
        assert_eq!(load("uk-GB"), "language");
        assert_eq!(load("fr-FR"), "default");
    }

    #[test]
    fn placeholders_dollar_escape_and_single_pass() {
        let dir = TempDir::new();
        dir.write_locale(
            "en",
            r#"{
                "greet": {
                    "message": "Hello $who$! Cost $$5, bonus $one$, second $two$, literal $b$.",
                    "placeholders": {
                        "who": { "content": "$1", "example": "Ada" },
                        "one": { "content": "$1" },
                        "two": { "content": "$2" },
                        "b": { "content": "$2" }
                    }
                },
                "bomb": {
                    "message": "$a$",
                    "placeholders": {
                        "a": { "content": "$b$" },
                        "b": { "content": "$1" }
                    }
                }
            }"#,
        );
        let messages = load_locales(&dir.0, Some("en"), "en").expect("locales load");
        let subs = vec!["Ada".to_owned(), "42".to_owned()];
        assert_eq!(
            messages.get_message("greet", &subs),
            "Hello Ada! Cost $5, bonus Ada, second 42, literal 42."
        );
        // Placeholder content is not rescanned: `$b$` stays literal.
        assert_eq!(messages.get_message("bomb", &subs), "$b$");
    }

    #[test]
    fn predefined_messages_and_inline_expansion() {
        let dir = TempDir::new();
        dir.write_locale(
            "en",
            r#"{"bidi":{"message":"dir=@@bidi_dir loc=@@ui_locale id=@@extension_id"}}"#,
        );
        let messages = load_locales(&dir.0, Some("en"), "uk-UA")
            .expect("locales load")
            .with_extension_id("ext@example.com");

        assert_eq!(messages.get_message("@@bidi_dir", &[]), "ltr");
        assert_eq!(messages.get_message("@@bidi_reversed_dir", &[]), "rtl");
        assert_eq!(messages.get_message("@@bidi_start_edge", &[]), "left");
        assert_eq!(messages.get_message("@@bidi_end_edge", &[]), "right");
        assert_eq!(messages.get_message("@@ui_locale", &[]), "uk-ua");
        assert_eq!(
            messages.get_message("@@extension_id", &[]),
            "ext@example.com"
        );
        assert_eq!(
            messages.get_message("bidi", &[]),
            "dir=ltr loc=uk-ua id=ext@example.com"
        );
    }

    #[test]
    fn unknown_double_at_prefix_stays_literal() {
        let dir = TempDir::new();
        dir.write_locale(
            "en",
            &msg("raw=@@bidi_direction dir=@@bidi_dir tail=@@bidi_directionX"),
        );
        let messages = load_locales(&dir.0, Some("en"), "en").expect("locales load");
        assert_eq!(
            messages.get_message("k", &[]),
            "raw=@@bidi_direction dir=ltr tail=@@bidi_directionX"
        );
    }

    #[test]
    fn missing_key_is_empty_and_warns() {
        let dir = TempDir::new();
        dir.write_locale("en", &msg("ok"));
        let messages = load_locales(&dir.0, Some("en"), "en").expect("locales load");
        assert_eq!(messages.get_message("absent", &[]), "");
    }

    #[test]
    fn broken_json_degrades_to_default_locale() {
        let dir = TempDir::new();
        dir.write_locale("de", "{ this is not json");
        dir.write_locale("en", &msg("english"));
        let messages = load_locales(&dir.0, Some("en"), "de-DE").expect("locales load");
        assert_eq!(messages.locale, "en");
        assert_eq!(messages.get_message("k", &[]), "english");
    }

    #[test]
    fn ten_thousand_keys_load_but_10001_do_not() {
        let dir = TempDir::new();
        let entries = (0..10_000)
            .map(|i| format!(r#""k{i}":{{"message":"v{i}"}}"#))
            .collect::<Vec<_>>()
            .join(",");
        dir.write_locale("en", &format!("{{{entries}}}"));
        let messages = load_locales(&dir.0, Some("en"), "en").expect("locales load");
        assert_eq!(messages.map.len(), 10_000);
        assert_eq!(messages.get_message("k9999", &[]), "v9999");

        dir.write_locale(
            "en",
            &format!("{{{entries},\"extra\":{{\"message\":\"x\"}}}}"),
        );
        let messages = load_locales(&dir.0, Some("en"), "en").expect("locales load");
        assert!(messages.map.is_empty());
    }

    #[test]
    fn invalid_key_names_are_rejected() {
        assert!(is_valid_message_key("appName_2@x"));
        assert!(!is_valid_message_key("bad-key"));
        assert!(!is_valid_message_key(""));
        assert!(!is_valid_message_key("a b"));
        assert!(!is_valid_message_key(&"x".repeat(MAX_KEY_BYTES + 1)));

        let dir = TempDir::new();
        dir.write_locale("en", r#"{"bad-key":{"message":"x"}}"#);
        let messages = load_locales(&dir.0, Some("en"), "en").expect("locales load");
        assert!(messages.map.is_empty());
    }

    #[test]
    fn missing_default_locale_is_an_error_when_locales_exist() {
        let dir = TempDir::new();
        dir.write_locale("en", &msg("ok"));
        assert!(matches!(
            load_locales(&dir.0, None, "en"),
            Err(I18nError::MissingDefaultLocale)
        ));
    }

    #[test]
    fn no_locales_directory_is_not_an_error() {
        let dir = TempDir::new();
        let messages = load_locales(&dir.0, Some("en"), "de").expect("locales load");
        assert!(messages.map.is_empty());
        assert_eq!(messages.get_message("anything", &[]), "");
    }

    #[test]
    fn os_locale_helpers_always_return_useful_values() {
        let locale = ui_locale();
        assert!(!locale.is_empty());
        let languages = get_accept_languages();
        assert!(languages.contains(&"en".to_owned()));
    }
}
