use std::borrow::Cow;

use pumpkin_i18n::{Locale, Token, resolve_translation};

use crate::text::{TextComponentBase, TextContent, style::Style};

pub use crate::translation::{translate_format, translate_plain};

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
                let component = with.get(*idx).cloned().unwrap_or_else(empty_text_component);
                extra.push(component);
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

    let Some(tokens) = resolved.tokens() else {
        return resolved.value_or_raw(namespaced_key);
    };

    // Even when `with` is empty, tokens may contain escaped-percent
    // Token::Text (e.g. "Progress: 100%%") that need unescaping.
    if with.is_empty() {
        let mut output = String::with_capacity(resolved.as_str().len());
        for token in tokens {
            if let Token::Text(text) = token {
                output.push_str(text);
            }
        }
        return output;
    }

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
    use pumpkin_i18n::{Locale, set_translation};

    use crate::text::TextComponent;

    use super::{get_translation_text, translation_to_pretty};

    #[test]
    fn formats_explicit_placeholders_and_literal_percent() {
        set_translation(
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
}
