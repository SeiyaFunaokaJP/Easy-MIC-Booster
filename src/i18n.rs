//! UI strings from `lang/*.json` (embedded at build time). Falls back to English
//! for missing keys and to the key itself as a last resort.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

/// Languages in the order shown in the selector: (code, native name, json).
pub const LANGUAGES: &[(&str, &str, &str)] = &[
    ("ja", "日本語", include_str!("../lang/ja.json")),
    ("en", "English", include_str!("../lang/en.json")),
    ("zh-CN", "简体中文", include_str!("../lang/zh-CN.json")),
    ("zh-TW", "繁體中文", include_str!("../lang/zh-TW.json")),
    ("ko", "한국어", include_str!("../lang/ko.json")),
    ("de", "Deutsch", include_str!("../lang/de.json")),
    ("fr", "Français", include_str!("../lang/fr.json")),
    ("es", "Español", include_str!("../lang/es.json")),
];

type Table = HashMap<String, String>;

struct State {
    code: &'static str,
    table: Table,
    fallback: Table,
}

fn parse(json: &str) -> Table {
    serde_json::from_str(json.trim_start_matches('\u{feff}')).unwrap_or_default()
}

fn state() -> &'static RwLock<State> {
    static S: OnceLock<RwLock<State>> = OnceLock::new();
    S.get_or_init(|| {
        let en = parse(LANGUAGES[1].2);
        RwLock::new(State {
            code: "en",
            table: en.clone(),
            fallback: en,
        })
    })
}

/// Switches the UI language; unknown codes fall back to English.
pub fn set_language(code: &str) {
    let (code, json) = LANGUAGES
        .iter()
        .find(|(c, _, _)| c.eq_ignore_ascii_case(code))
        .map(|(c, _, j)| (*c, *j))
        .unwrap_or((LANGUAGES[1].0, LANGUAGES[1].2));
    let mut s = state().write().unwrap();
    s.code = code;
    s.table = parse(json);
}

pub fn current() -> &'static str {
    state().read().unwrap().code
}

pub fn tr(key: &str) -> String {
    let s = state().read().unwrap();
    s.table
        .get(key)
        .or_else(|| s.fallback.get(key))
        .cloned()
        .unwrap_or_else(|| key.to_string())
}

/// `tr` with `{0}`, `{1}`… placeholders.
pub fn trf(key: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut out = tr(key);
    for (i, a) in args.iter().enumerate() {
        out = out.replace(&format!("{{{i}}}"), &a.to_string());
    }
    out
}

/// Picks the supported language closest to the OS UI language (first run only).
pub fn detect_system_language() -> String {
    let locale = sys_locale::get_locale()
        .unwrap_or_default()
        .replace('_', "-");
    let lower = locale.to_lowercase();
    if lower.starts_with("zh") {
        let traditional = ["-tw", "-hk", "-mo", "hant"]
            .iter()
            .any(|t| lower.contains(t));
        return if traditional { "zh-TW" } else { "zh-CN" }.to_string();
    }
    let primary = lower.split('-').next().unwrap_or("");
    LANGUAGES
        .iter()
        .find(|(c, _, _)| c.eq_ignore_ascii_case(primary))
        .map(|(c, _, _)| c.to_string())
        .unwrap_or_else(|| "en".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_has_every_english_key() {
        let en = parse(LANGUAGES[1].2);
        for (code, _, json) in LANGUAGES {
            let t = parse(json);
            assert!(!t.is_empty(), "{code} failed to parse");
            let missing: Vec<_> = en.keys().filter(|k| !t.contains_key(*k)).collect();
            assert!(missing.is_empty(), "{code} is missing {missing:?}");
        }
    }

    #[test]
    fn formats_placeholders() {
        set_language("en");
        assert_eq!(trf("Device_Default", &[&"Mic"]), "Default Device (Mic)");
    }
}
