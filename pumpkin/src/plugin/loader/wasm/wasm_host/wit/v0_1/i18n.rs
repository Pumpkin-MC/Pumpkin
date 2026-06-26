use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1::pumpkin::plugin::{common::Locale as WitLocale, i18n::Host},
};
use pumpkin_i18n::{Locale as UtilLocale, add_translation_file, get_translation};
use std::str::FromStr;

impl Host for PluginHostState {
    async fn translate(&mut self, key: String, locale: WitLocale) -> wasmtime::Result<String> {
        let util_locale = wit_to_util_locale(locale);
        Ok(get_translation(&key, util_locale))
    }

    async fn load_translations(
        &mut self,
        namespace: String,
        json: String,
        locale: WitLocale,
    ) -> wasmtime::Result<()> {
        let util_locale = wit_to_util_locale(locale);
        add_translation_file(namespace, json, util_locale);
        Ok(())
    }
}

/// Converts a WIT Locale to a pumpkin-util Locale.
///
/// The WIT locale enum uses Rust `PascalCase` variant names (e.g. `EnUs`, `ZhCn`)
/// which are converted to `snake_case` via [`Locale::from_str`]'s built-in
/// [`normalize_locale_code`] normalizer. Falls back to `EnUs` with a warning if
/// the conversion fails — this should not happen as long as the WIT locale
/// variants stay in sync with the [`Locale`] enum.
fn wit_to_util_locale(wit: WitLocale) -> UtilLocale {
    let s = format!("{wit:?}");
    UtilLocale::from_str(&s).unwrap_or_else(|()| {
        tracing::warn!(
            "failed to convert WIT locale '{s}' to pumpkin Locale, falling back to EnUs"
        );
        UtilLocale::EnUs
    })
}
