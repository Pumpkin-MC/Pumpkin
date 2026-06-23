use std::borrow::Cow;

use pumpkin_i18n::{Locale, SubstitutionRange, Token, resolve_translation};

use crate::text::{TextComponentBase, TextContent, style::Style};

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
    with: Vec<TextComponentBase>,
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
pub fn translation_to_pretty<P: Into<Cow<'static, str>>>(
    namespaced_key: P,
    locale: Locale,
    with: Vec<TextComponentBase>,
) -> String {
    format_translation_components(
        namespaced_key.into(),
        locale,
        &with,
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
pub fn get_translation_text<P: Into<Cow<'static, str>>>(
    namespaced_key: P,
    locale: Locale,
    with: Vec<TextComponentBase>,
) -> String {
    format_translation_components(namespaced_key.into(), locale, &with, |component| {
        component.get_text(locale)
    })
}

pub(crate) fn resolve_translation_components(
    namespaced_key: &str,
    locale: Locale,
    with: Vec<TextComponentBase>,
) -> (String, Vec<TextComponentBase>) {
    let resolved = resolve_translation(namespaced_key, locale);
    if with.is_empty() {
        return (resolved_text(namespaced_key, &resolved), Vec::new());
    }

    let Some(tokens) = resolved.tokens() else {
        return (resolved_text(namespaced_key, &resolved), Vec::new());
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
    namespaced_key: Cow<'static, str>,
    locale: Locale,
    with: &[TextComponentBase],
    mut render_component: F,
) -> String
where
    F: FnMut(TextComponentBase) -> String,
{
    let resolved = resolve_translation(namespaced_key.as_ref(), locale);
    if with.is_empty() {
        return resolved_text(namespaced_key.as_ref(), &resolved);
    }

    let Some(tokens) = resolved.tokens() else {
        return resolved_text(namespaced_key.as_ref(), &resolved);
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

fn resolved_text(raw_key: &str, resolved: &pumpkin_i18n::ResolvedTranslation) -> String {
    if resolved.is_missing() {
        raw_key.to_owned()
    } else {
        resolved.as_str().to_owned()
    }
}

fn placeholder_ranges(translation: &str) -> Vec<(usize, SubstitutionRange)> {
    let bytes = translation.as_bytes();
    let mut ranges = Vec::new();
    let mut cursor = 0usize;
    let mut sequential_idx = 0usize;

    while cursor < bytes.len() {
        if bytes[cursor] != b'%' {
            cursor += 1;
            continue;
        }

        if cursor > 0 && bytes[cursor - 1] == b'\\' {
            cursor += 1;
            continue;
        }

        if cursor + 1 >= bytes.len() {
            break;
        }

        if bytes[cursor + 1] == b'%' {
            cursor += 2;
            continue;
        }

        let mut look = cursor + 1;
        let digits_start = look;
        while look < bytes.len() && bytes[look].is_ascii_digit() {
            look += 1;
        }

        if look > digits_start && look + 1 < bytes.len() && bytes[look] == b'$' {
            let arg_idx = translation[digits_start..look]
                .parse::<usize>()
                .unwrap_or(1)
                .saturating_sub(1);
            ranges.push((
                arg_idx,
                SubstitutionRange {
                    start: cursor,
                    end: look + 1,
                },
            ));
            cursor = look + 2;
            continue;
        }

        ranges.push((
            sequential_idx,
            SubstitutionRange {
                start: cursor,
                end: cursor + 1,
            },
        ));
        sequential_idx += 1;
        cursor += 2;
    }

    ranges
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
            get_translation_text("test_util_translation:ordered", Locale::EnUs, args.clone()),
            "B then A % done"
        );
        assert_eq!(
            translation_to_pretty("test_util_translation:ordered", Locale::EnUs, args),
            "B then A % done"
        );
    }

    #[test]
    fn reorder_substitutions_handles_missing_args() {
        let (substitutions, ranges) =
            reorder_substitutions("%s %2$s %%", vec![TextComponent::text("A").0]);

        assert_eq!(ranges.len(), 2);
        assert_eq!(substitutions[0].clone().get_text(Locale::EnUs), "A");
        assert_eq!(substitutions[1].clone().get_text(Locale::EnUs), "");
    }
}
