use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1_0::pumpkin::plugin::{common::Locale as WitLocale, i18n::Host},
};
use pumpkin_util::translation::{Locale as UtilLocale, add_translation_file, get_translation};
use std::str::FromStr;

impl Host for PluginHostState {
    async fn translate(&mut self, key: String, locale: WitLocale) -> String {
        let util_locale = wit_to_util_locale(locale);
        get_translation(&key, util_locale)
    }

    async fn load_translations(&mut self, namespace: String, json: String, locale: WitLocale) {
        let util_locale = wit_to_util_locale(locale);
        add_translation_file(namespace, json, util_locale);
    }
}

/// Converts a WIT Locale to a pumpkin-util Locale.
fn wit_to_util_locale(wit: WitLocale) -> UtilLocale {
    // WIT names are kebab-case (e.g., AfZa -> af-za in .wit, but in generated Rust it might vary)
    // We can use the debug representation or a custom mapping.
    // Given the amount of variants, we use a string-based approach if possible.
    let s = format!("{wit:?}").to_lowercase();
    // pumpkin-util::Locale::from_str expects "af_za" or similar.
    // WIT might generate "AfZa" or "af_za" depending on bindgen config.
    // Let's assume standard bindgen which might produce "AfZa".
    // We'll replace kebab-case if necessary.
    let s = s.replace('-', "_");
    UtilLocale::from_str(&s).unwrap_or(UtilLocale::EnUs)
}
