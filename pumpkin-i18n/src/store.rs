use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use tracing::{error, warn};

use crate::locale::Locale;

// Include auto-generated translation loading code from build.rs
include!(concat!(env!("OUT_DIR"), "/generated_store.rs"));

/// Global translation store, populated at startup by `load_all_translations()`.
///
/// Indexed by `locale as usize` for O(1) lookup within a `Mutex`.
pub static TRANSLATIONS: LazyLock<Mutex<[HashMap<String, String>; Locale::COUNT]>> =
    LazyLock::new(|| Mutex::new(load_all_translations()));

/// Adds or overrides a single translation entry.
///
/// # Arguments
/// * `namespace`: The namespace of the translation key.
/// * `key`: The translation key without namespace.
/// * `translation`: The localized translation string.
/// * `locale`: The locale the translation belongs to.
pub fn add_translation<P: Into<String>>(namespace: P, key: P, translation: P, locale: Locale) {
    let mut translations = TRANSLATIONS.lock().unwrap();
    let namespaced_key = format!("{}:{}", namespace.into(), key.into()).to_ascii_lowercase();
    translations[locale as usize].insert(namespaced_key, translation.into());
}

/// Loads translations from a JSON string and registers them under a namespace.
///
/// # Arguments
/// * `namespace`: The namespace applied to all loaded keys.
/// * `file_path`: A JSON string containing a flat key-value translation map.
/// * `locale`: The locale the translations belong to.
pub fn add_translation_file<P: Into<String>>(namespace: P, file_path: P, locale: Locale) {
    let translations_map: HashMap<String, String> =
        serde_json::from_str(&file_path.into()).unwrap_or(HashMap::new());
    if translations_map.is_empty() {
        // TODO: Handle the case where the file is empty or not found properly
        return;
    }

    let mut translations = TRANSLATIONS.lock().unwrap();
    let namespace = namespace.into();
    for (key, translation) in translations_map {
        let namespaced_key = format!("{namespace}:{key}").to_ascii_lowercase();
        translations[locale as usize].insert(namespaced_key, translation);
    }
}

/// Retrieves a translation for the given key and locale.
///
/// # Fallback strategy
/// 1. **Requested locale** — silent, no log.
/// 2. **`EnUs`** — logs [`warn!`] when the key was not found in step 1.
/// 3. **Raw key** — logs [`error!`] when neither locale contains the key.
///
/// # Arguments
/// * `key`: The fully qualified `namespace:key`.
/// * `locale`: The requested locale.
///
/// # Returns
/// The localized translation, the English fallback, or the raw key.
pub fn get_translation(key: &str, locale: Locale) -> String {
    let translations = TRANSLATIONS.lock().unwrap();
    let key_lower = key.to_ascii_lowercase();

    // Tier 1 – requested locale (silent)
    if let Some(value) = translations[locale as usize].get(&key_lower) {
        return value.clone();
    }

    // Tier 2 – EnUs fallback
    if let Some(value) = translations[Locale::EnUs as usize].get(&key_lower) {
        warn!("translation key not found – falling back to English");
        return value.clone();
    }

    // Tier 3 – raw key
    error!("translation key not found in any locale – returning raw key");
    key.to_owned()
}
