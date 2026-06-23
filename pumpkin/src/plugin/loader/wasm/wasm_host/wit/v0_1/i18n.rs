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
fn wit_to_util_locale(wit: WitLocale) -> UtilLocale {
    let s = format!("{wit:?}");
    UtilLocale::from_str(&s).unwrap_or(UtilLocale::EnUs)
}
