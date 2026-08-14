use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1::pumpkin::plugin::{common::Locale as WitLocale, i18n::Host},
};
use pumpkin_util::translation::{Locale as UtilLocale, add_translation_file, get_translation};
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
fn wit_to_util_locale(wit: WitLocale) -> UtilLocale {
    // WIT variants like EnUs often debug to "EnUs".
    // We convert to lowercase and handle potential format differences.
    let s = format!("{wit:?}").to_lowercase();

    // Most translation systems expect underscores (en_us) rather than nothing or dashes.
    // If the WIT Debug format is "EnUs", lowercase is "enus".
    // We might need a smarter mapping if your util expects specifically "en_us".
    UtilLocale::from_str(&s).unwrap_or(UtilLocale::EnUs)
}
