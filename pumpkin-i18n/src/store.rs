use std::{collections::HashMap, sync::Arc, sync::LazyLock};

use tracing::warn;

use crate::{
    engine::{ResolvedTranslation, TranslationEngine},
    locale::Locale,
};

// Include auto-generated translation loading code from build.rs
include!(concat!(env!("OUT_DIR"), "/generated_store.rs"));

static ENGINE: LazyLock<TranslationEngine> = LazyLock::new(|| {
    let translations = load_all_translations();
    TranslationEngine::build(&translations)
});

/// Returns the global translation engine.
#[must_use]
pub fn translation_engine() -> &'static TranslationEngine {
    &ENGINE
}

/// Resolves a translation using the global engine.
///
/// Key normalisation (lowercasing) is handled inside the engine,
/// so keys of any casing are accepted here.
#[must_use]
pub fn resolve_translation(key: &str, locale: Locale) -> Arc<ResolvedTranslation> {
    translation_engine().resolve(locale as usize, key)
}

/// Adds or overrides a single translation entry.
///
/// # Arguments
/// * `namespace`: The namespace of the translation key (e.g. `"pumpkin"`).
///   Must **not** already contain a namespace prefix or colon.
/// * `key`: The bare translation key (e.g. `"commands.help.header"`).
///   Must **not** already contain a namespace prefix or colon.
/// * `translation`: The localized translation string.
/// * `locale`: The locale the translation belongs to.
///
/// The full key is assembled as `"{namespace}:{key}"` via
/// [`crate::namespaced_key`]. Passing an already-namespaced key
/// will produce a double-prefixed key such as `"pumpkin:pumpkin:foo"`.
pub fn set_translation<N, K, V>(namespace: N, key: K, translation: V, locale: Locale)
where
    N: AsRef<str>,
    K: AsRef<str>,
    V: Into<String>,
{
    let namespaced_key = crate::namespaced_key(namespace.as_ref(), key.as_ref());
    let translation = translation.into();

    translation_engine().set_translation(locale as usize, &namespaced_key, &translation);
}

/// Loads translations from a JSON string and registers them under a namespace.
///
/// # Arguments
/// * `namespace`: The namespace applied to all loaded keys.
/// * `json_content`: A JSON string containing a flat key-value translation map.
/// * `locale`: The locale the translations belong to.
pub fn load_translations<N, J>(namespace: N, json_content: J, locale: Locale)
where
    N: AsRef<str>,
    J: AsRef<str>,
{
    let translations_map: HashMap<String, String> =
        match serde_json::from_str(json_content.as_ref()) {
            Ok(map) => map,
            Err(error) => {
                warn!("failed to parse translation json: {error}");
                return;
            }
        };

    if translations_map.is_empty() {
        warn!(
            "no translations found in JSON string for namespace '{}' — the file may be empty or malformed",
            namespace.as_ref()
        );
        return;
    }

    let namespace = namespace.as_ref();
    let entries = translations_map
        .into_iter()
        .map(|(key, translation)| {
            (
                crate::namespaced_key(namespace, &key).to_ascii_lowercase(),
                translation,
            )
        })
        .collect::<Vec<_>>();

    translation_engine().extend_translations(locale as usize, entries);
}

/// Retrieves a translation for the given key and locale.
///
/// # Fallback strategy
/// 1. **Requested locale** — silent, no log.
/// 2. **`EnUs`** — logs [`debug!`] when the key was not found in step 1.
/// 3. **Raw key** — logs [`error!`] when neither locale contains the key.
///
/// # Arguments
/// * `key`: The fully qualified `namespace:key`.
/// * `locale`: The requested locale.
///
/// # Returns
/// The localized translation, the English fallback, or the raw key.
#[must_use]
pub fn get_translation(key: &str, locale: Locale) -> String {
    resolve_translation(key, locale).value_or_raw(key)
}

/// Formats a translation with already-rendered string arguments.
#[must_use]
pub fn format_translation(key: &str, locale: Locale, args: &[String]) -> String {
    let resolved = resolve_translation(key, locale);
    // Always use the token-aware write_to path when tokens are present,
    // even when args is empty.  A template like "Progress: 100%%" contains
    // an escaped-percent Token::Text — returning the raw template without
    // unescaping would leave "100%%" visible.
    let Some(tokens) = resolved.tokens() else {
        return resolved.value_or_raw(key);
    };

    let mut output = String::with_capacity(resolved.as_str().len());
    format_tokens(tokens, args, &mut output);
    output
}

// Re-export to avoid a circular dependency (format_tokens lives in engine).
use crate::engine::format_tokens;

#[cfg(test)]
mod tests {
    use crate::Locale;

    use super::{format_translation, get_translation, set_translation};

    #[test]
    fn runtime_translation_overrides_are_visible_on_hot_path() {
        set_translation("test_runtime_store", "hello", "Hello %s", Locale::EnUs);

        assert_eq!(
            get_translation("TEST_RUNTIME_STORE:HELLO", Locale::ZhCn),
            "Hello %s"
        );
        assert_eq!(
            format_translation(
                "test_runtime_store:hello",
                Locale::ZhCn,
                &["Pumpkin".to_owned()]
            ),
            "Hello Pumpkin"
        );
    }

    #[test]
    fn missing_translation_returns_original_key() {
        assert_eq!(
            get_translation("Test_Runtime_Store:DefinitelyMissing", Locale::EnUs),
            "Test_Runtime_Store:DefinitelyMissing"
        );
    }

    #[test]
    fn cross_locale_cache_invalidation_stale_fallback() {
        use crate::store::load_translations;

        // ZhCn misses → falls back to EnUs → cached in ZhCn cache.
        assert!(get_translation("cross:foo", Locale::ZhCn).contains("cross:foo"));

        // Now load value into EnUs.  ZhCn must see the EnUs fallback, not
        // the old Missing from its own cache.
        load_translations("cross", r#"{"foo":"Pumpkin rocks"}"#, Locale::EnUs);
        assert_eq!(get_translation("cross:foo", Locale::ZhCn), "Pumpkin rocks");
    }

    #[test]
    fn format_translation_unescapes_percent_with_empty_args() {
        use crate::store::load_translations;

        load_translations(
            "percent_test",
            r#"{"progress":"Loading: 100%% complete"}"#,
            Locale::EnUs,
        );
        // No args provided → only Text tokens → must still unescape %%.
        assert_eq!(
            format_translation("percent_test:progress", Locale::EnUs, &[]),
            "Loading: 100% complete"
        );
    }
}
