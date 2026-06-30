use std::borrow::Cow;

use pumpkin_i18n::{Locale, SubstitutionRange, Token, placeholder_ranges, resolve_translation};

use crate::text::{TextComponentBase, TextContent, style::Style};

pub use crate::translation::{translate_format, translate_plain};

/// Reorders substitution placeholders within a translation string.
///
/// # Arguments
/// * `translation`: The raw translation string containing placeholders.
/// * `with`: Substitution components to insert into the placeholders.
///
/// # Returns
/// A tuple containing the reordered components and their substitution ranges.
#[must_use]
pub fn reorder_substitutions(
    translation: &str,
    with: &[TextComponentBase],
) -> (Vec<TextComponentBase>, Vec<SubstitutionRange>) {
    let placeholders = placeholder_ranges(translation);
    let mut substitutions = Vec::with_capacity(placeholders.len());
    let mut ranges = Vec::with_capacity(placeholders.len());

    for (arg_idx, range) in placeholders {
        substitutions.push(
            with.get(arg_idx)
                .cloned()
                .unwrap_or_else(empty_text_component),
        );
        ranges.push(range);
    }

    (substitutions, ranges)
}

/// Resolves a translation into formatted console output.
///
/// # Arguments
/// * `namespaced_key`: The fully qualified `namespace:key`.
/// * `locale`: The requested locale.
/// * `with`: Substitution components used to replace placeholders.
///
/// # Returns
/// The resolved and formatted translation string.
pub fn translation_to_pretty<P: AsRef<str>>(
    namespaced_key: P,
    locale: Locale,
    with: &[TextComponentBase],
) -> String {
    format_translation_components(
        namespaced_key.as_ref(),
        locale,
        with,
        TextComponentBase::to_pretty_console,
    )
}

/// Resolves a translation into plain text.
///
/// # Arguments
/// * `namespaced_key`: The fully qualified `namespace:key`.
/// * `locale`: The requested locale.
/// * `with`: Substitution components used to replace placeholders.
///
/// # Returns
/// The resolved translation as plain text.
pub fn get_translation_text<P: AsRef<str>>(
    namespaced_key: P,
    locale: Locale,
    with: &[TextComponentBase],
) -> String {
    format_translation_components(namespaced_key.as_ref(), locale, with, |component| {
        component.get_text(locale)
    })
}

pub(crate) fn resolve_translation_components(
    namespaced_key: &str,
    locale: Locale,
    with: &[TextComponentBase],
) -> (String, Vec<TextComponentBase>) {
    let resolved = resolve_translation(namespaced_key, locale);
    if with.is_empty() {
        return (resolved.value_or_raw(namespaced_key), Vec::new());
    }

    let Some(tokens) = resolved.tokens() else {
        return (resolved.value_or_raw(namespaced_key), Vec::new());
    };

    let mut parent = String::new();
    let mut extra = Vec::new();
    let mut writing_parent = true;

    for token in tokens {
        match token {
            Token::Text(text) if writing_parent => parent.push_str(text),
            Token::Text(text) if !text.is_empty() => extra.push(text_component(text.as_ref())),
            Token::Text(_) => {}
            Token::Var(idx) => {
                writing_parent = false;
                if let Some(component) = with.get(*idx) {
                    extra.push(component.clone());
                }
            }
        }
    }

    (parent, extra)
}

fn format_translation_components<F>(
    namespaced_key: &str,
    locale: Locale,
    with: &[TextComponentBase],
    mut render_component: F,
) -> String
where
    F: FnMut(TextComponentBase) -> String,
{
    let resolved = resolve_translation(namespaced_key, locale);
    if with.is_empty() {
        return resolved.value_or_raw(namespaced_key);
    }

    let Some(tokens) = resolved.tokens() else {
        return resolved.value_or_raw(namespaced_key);
    };

    let mut result = String::with_capacity(resolved.as_str().len());
    for token in tokens {
        match token {
            Token::Text(text) => result.push_str(text),
            Token::Var(idx) => {
                if let Some(component) = with.get(*idx) {
                    result.push_str(&render_component(component.clone()));
                }
            }
        }
    }
    result
}

fn empty_text_component() -> TextComponentBase {
    text_component("")
}

fn text_component(text: &str) -> TextComponentBase {
    TextComponentBase {
        content: Box::new(TextContent::Text {
            text: Cow::Owned(text.to_owned()),
        }),
        style: Box::new(Style::default()),
        extra: vec![],
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_i18n::{Locale, add_translation};

    use crate::text::TextComponent;

    use super::{get_translation_text, reorder_substitutions, translation_to_pretty};

    #[test]
    fn formats_explicit_placeholders_and_literal_percent() {
        add_translation(
            "test_util_translation",
            "ordered",
            "%2$s then %1$s %% done",
            Locale::EnUs,
        );

        let args = vec![TextComponent::text("A").0, TextComponent::text("B").0];

        assert_eq!(
            get_translation_text("test_util_translation:ordered", Locale::EnUs, &args),
            "B then A % done"
        );
        assert_eq!(
            translation_to_pretty("test_util_translation:ordered", Locale::EnUs, &args),
            "B then A % done"
        );
    }

    #[test]
    fn reorder_substitutions_handles_missing_args() {
        let (substitutions, ranges) =
            reorder_substitutions("%s %2$s %% {name:?}", &[TextComponent::text("A").0]);

        assert_eq!(ranges.len(), 3);
        assert_eq!(substitutions[0].clone().get_text(Locale::EnUs), "A");
        assert_eq!(substitutions[1].clone().get_text(Locale::EnUs), "");
        assert_eq!(substitutions[2].clone().get_text(Locale::EnUs), "");
    }
}
