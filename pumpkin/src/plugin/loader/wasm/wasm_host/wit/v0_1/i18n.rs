use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1::pumpkin::plugin::{common::Locale as WitLocale, i18n::Host},
};
use pumpkin_util::text::{TextComponentBase, TextContent, style::Style};
use pumpkin_util::translation::{
    Locale as UtilLocale, add_translation_file, get_translation, get_translation_text,
};
use std::borrow::Cow;
use std::str::FromStr;

impl Host for PluginHostState {
    async fn translate(&mut self, key: String, locale: WitLocale) -> wasmtime::Result<String> {
        let util_locale = wit_to_util_locale(locale);
        Ok(get_translation(&key, util_locale))
    }

    async fn translate_text(
        &mut self,
        key: String,
        locale: WitLocale,
        substitutions: Vec<String>,
    ) -> wasmtime::Result<String> {
        let util_locale = wit_to_util_locale(locale);
        if substitutions.is_empty() {
            return Ok(get_translation(&key, util_locale));
        }
        let with: Vec<TextComponentBase> = substitutions
            .into_iter()
            .map(|s| TextComponentBase {
                content: Box::new(TextContent::Text {
                    text: Cow::Owned(s),
                }),
                style: Box::new(Style::default()),
                extra: vec![],
            })
            .collect();
        Ok(get_translation_text(key, util_locale, with))
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
/// WIT enums debug-print as CamelCase variant names (`"EnUs"`, `"ZhCn"`).
/// We convert each uppercase letter boundary to an underscore and lowercase
/// the whole string (`"EnUs"` → `"en_us"`), matching `Locale::from_str`.
fn wit_to_util_locale(wit: WitLocale) -> UtilLocale {
    let raw = format!("{wit:?}");
    let mut result = String::with_capacity(raw.len() + 4);
    for (i, c) in raw.char_indices() {
        if c.is_ascii_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    UtilLocale::from_str(&result).unwrap_or(UtilLocale::EnUs)
}
