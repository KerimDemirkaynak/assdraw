//! Guards against typos in translation keys: every key used by the UI code must resolve to a
//! real translation (not the key itself) in all seven languages.

use assdraw_rs::i18n::{t, Language};
use std::path::Path;

fn collect_keys(dir: &Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("read_dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_keys(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let src = std::fs::read_to_string(&path).expect("read source");
            for pat in ["t(lang, \"", "t(self.settings.language, \""] {
                let mut rest = src.as_str();
                while let Some(i) = rest.find(pat) {
                    rest = &rest[i + pat.len()..];
                    if let Some(j) = rest.find('"') {
                        out.push(rest[..j].to_string());
                    }
                }
            }
        }
    }
}

#[test]
fn every_used_key_resolves_in_every_language() {
    let mut keys = Vec::new();
    collect_keys(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut keys);
    assert!(keys.len() > 50, "key scan found too few keys: {}", keys.len());
    keys.sort();
    keys.dedup();

    for key in keys {
        let key: &'static str = Box::leak(key.into_boxed_str());
        for lang in Language::ALL {
            assert_ne!(t(lang, key), key, "missing translation for '{key}' in {lang:?}");
        }
    }
}

#[test]
fn language_actually_changes_the_text() {
    assert_eq!(t(Language::En, "menu_mode"), "Mode");
    assert_eq!(t(Language::Tr, "menu_mode"), "Kip");
    assert_eq!(t(Language::De, "menu_mode"), "Modus");
    // Untranslated manual text falls back to English instead of showing the key.
    assert_eq!(t(Language::Fr, "man_title"), t(Language::En, "man_title"));
}
