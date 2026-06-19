use std::borrow::Cow;

use pumpkin_i18n::{
    Locale, get_translation, get_translation_entry, reorder_substitutions, reorder_with_entry,
};

use crate::text::{TextComponentBase, TextContent, style::Style};

/// A default empty `TextComponentBase` used as a fallback when a substitution
/// placeholder has no matching value.
pub(crate) fn empty_component() -> TextComponentBase {
    TextComponentBase {
        content: Box::new(TextContent::Text { text: "".into() }),
        style: Box::new(Style::default()),
        extra: vec![],
    }
}

/// Resolves a translation into formatted console output.
///
/// Uses pre‑computed substitution ranges for zero‑allocation fast path.
///
/// # Arguments
/// * `namespaced_key`: The fully qualified `namespace:key`.
/// * `locale`: The requested locale.
/// * `with`: Substitution components used to replace placeholders.
///
/// # Returns
/// The resolved and formatted translation string.
pub fn translation_to_pretty<P: Into<Cow<'static, str>>>(
    namespaced_key: P,
    locale: Locale,
    with: Vec<TextComponentBase>,
) -> String {
    let key = namespaced_key.into();
    let translation = get_translation(&key, locale);

    if with.is_empty() || !translation.contains('%') {
        return translation.to_string();
    }

    // Use pre‑computed ranges for the fast path
    let entry = get_translation_entry(&key, locale);
    if !entry.ranges.is_empty() {
        let translation_str = &*entry.text;
        let (substitutions, ranges) = reorder_with_entry(&entry, with, empty_component());

        let mut result = String::with_capacity(translation_str.len());
        let mut pos = 0;

        for (idx, range) in ranges.iter().enumerate() {
            let sub_idx = idx.min(substitutions.len().saturating_sub(1));
            let substitution = substitutions[sub_idx].clone().to_pretty_console();

            result.push_str(&translation_str[pos..range.start]);
            result.push_str(&substitution);
            pos = range.end + 1;
        }

        result.push_str(&translation_str[pos..]);
        return result;
    }

    // Fallback: use legacy reorder_substitutions
    let translation_str = translation.to_string();
    let (substitutions, indices) = reorder_substitutions(&translation_str, with, empty_component());
    let mut result = String::new();
    let mut pos = 0;

    for (idx, &range) in indices.iter().enumerate() {
        let sub_idx = idx.clamp(0, substitutions.len() - 1);
        let substitution = substitutions[sub_idx].clone().to_pretty_console();

        result.push_str(&translation_str[pos..range.start]);
        result.push_str(&substitution);
        pos = range.end + 1;
    }

    result.push_str(&translation_str[pos..]);
    result
}

/// Resolves a translation into plain text.
///
/// Uses pre‑computed substitution ranges for zero‑allocation fast path.
///
/// # Arguments
/// * `namespaced_key`: The fully qualified `namespace:key`.
/// * `locale`: The requested locale.
/// * `with`: Substitution components used to replace placeholders.
///
/// # Returns
/// The resolved translation as plain text.
pub fn get_translation_text<P: Into<Cow<'static, str>>>(
    namespaced_key: P,
    locale: Locale,
    with: Vec<TextComponentBase>,
) -> String {
    let key = namespaced_key.into();
    let translation = get_translation(&key, locale);

    if with.is_empty() || !translation.contains('%') {
        return translation.to_string();
    }

    // Use pre‑computed ranges for the fast path
    let entry = get_translation_entry(&key, locale);
    if !entry.ranges.is_empty() {
        let translation_str = &*entry.text;
        let (substitutions, ranges) = reorder_with_entry(&entry, with, empty_component());

        let mut result = String::with_capacity(translation_str.len());
        let mut pos = 0;

        for (idx, range) in ranges.iter().enumerate() {
            let sub_idx = idx.min(substitutions.len().saturating_sub(1));
            let substitution = substitutions[sub_idx].clone().get_text(locale);

            result.push_str(&translation_str[pos..range.start]);
            result.push_str(&substitution);
            pos = range.end + 1;
        }

        result.push_str(&translation_str[pos..]);
        return result;
    }

    // Fallback: use legacy reorder_substitutions
    let translation_str = translation.to_string();
    let (substitutions, indices) = reorder_substitutions(&translation_str, with, empty_component());
    let mut result = String::new();
    let mut pos = 0;

    for (idx, &range) in indices.iter().enumerate() {
        let sub_idx = idx.clamp(0, substitutions.len() - 1);
        let substitution = substitutions[sub_idx].clone().get_text(locale);

        result.push_str(&translation_str[pos..range.start]);
        result.push_str(&substitution);
        pos = range.end + 1;
    }

    result.push_str(&translation_str[pos..]);
    result
}
