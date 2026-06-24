use pumpkin_i18n::{format_translation, get_translation, server_global_locale};

use crate::text::TextComponent;

const PUMPKIN_TRANSLATION_NAMESPACE: &str = "pumpkin";

#[must_use]
pub fn localized_log(key: &str) -> String {
    let mut namespaced = String::with_capacity(PUMPKIN_TRANSLATION_NAMESPACE.len() + key.len() + 1);
    namespaced.push_str(PUMPKIN_TRANSLATION_NAMESPACE);
    namespaced.push(':');
    namespaced.push_str(key);
    get_translation(&namespaced, server_global_locale())
}

#[must_use]
pub fn localized_log_format(key: &str, args: &[String]) -> String {
    let mut namespaced = String::with_capacity(PUMPKIN_TRANSLATION_NAMESPACE.len() + key.len() + 1);
    namespaced.push_str(PUMPKIN_TRANSLATION_NAMESPACE);
    namespaced.push(':');
    namespaced.push_str(key);
    format_translation(&namespaced, server_global_locale(), args)
}

#[must_use]
pub fn localized_text<W>(key: &'static str, with: W) -> TextComponent
where
    W: Into<Vec<TextComponent>>,
{
    TextComponent::custom(
        PUMPKIN_TRANSLATION_NAMESPACE,
        key,
        server_global_locale(),
        with,
    )
}
