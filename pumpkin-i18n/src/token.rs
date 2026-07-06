use std::{collections::HashMap, sync::Arc};

use crate::SubstitutionRange;

/// A precompiled token in a translation format template.
///
/// During startup, every translation string containing placeholders
/// is parsed into a sequence of [`Token`]s so that runtime substitution
/// does zero parsing work — it simply streams the tokens into a buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    /// A static text fragment to be written verbatim.
    Text(Arc<str>),
    /// A variable slot referencing a parameter by index (0‑based).
    ///
    /// For `%s` placeholders the index is sequential (0, 1, 2, …);
    /// for `%1$s` it is the explicit 1‑based index minus one.
    Var(usize),
}

/// The result of precompiling a format string – `None` when the
/// string contains no placeholders (callers can use the raw string
/// directly in that case).
pub type TokenStream = Arc<[Token]>;

/// Precompile a translation format string into a [`TokenStream`].
///
/// Supported placeholders:
/// * `%%`       → literal `%` (emitted as [`Token::Text`])
/// * `%s`, `%d`, `%f`, … → [`Token::Var`] with sequential index
/// * `%1$s`, `%2$d`, …  → [`Token::Var`] with explicit 1‑based index
/// * `{}`, `{:?}` → [`Token::Var`] with sequential index
/// * `{name}`, `{name:?}` → [`Token::Var`] using the first-seen name order
/// * `{0}`, `{0:?}` → [`Token::Var`] with explicit 0‑based index
/// * `{{`, `}}` → literal braces
///
/// Returns `None` if the string contains no placeholders.
///
/// # Examples
/// ```ignore
/// let tokens = precompile("Hello %s, you have %d messages").unwrap();
/// // → [Text("Hello "), Var(0), Text(", you have "), Var(1), Text(" messages")]
/// ```
#[must_use]
pub fn precompile(template: &str) -> Option<TokenStream> {
    let parsed = parse_template(template);
    parsed.has_tokens.then(|| parsed.tokens.into())
}

/// Returns placeholder argument indexes and byte ranges using the same parser as
/// [`precompile`].
#[must_use]
pub fn placeholder_ranges(template: &str) -> Vec<(usize, SubstitutionRange)> {
    parse_template(template).placeholders
}

struct ParsedTemplate {
    tokens: Vec<Token>,
    placeholders: Vec<(usize, SubstitutionRange)>,
    has_tokens: bool,
}

fn parse_template(template: &str) -> ParsedTemplate {
    let bytes = template.as_bytes();
    let len = bytes.len();

    if !bytes.contains(&b'%') && !bytes.contains(&b'{') && !bytes.contains(&b'}') {
        return ParsedTemplate {
            tokens: Vec::new(),
            placeholders: Vec::new(),
            has_tokens: false,
        };
    }

    let mut tokens: Vec<Token> = Vec::new();
    let mut placeholders = Vec::new();
    let mut named_args: HashMap<&str, usize> = HashMap::new();
    let mut cursor = 0usize;
    let mut text_start = 0usize;
    let mut sequential_idx = 0usize;

    while cursor < len {
        match bytes[cursor] {
            b'%' if !is_backslash_escaped(bytes, cursor) => {
                parse_percent_placeholder(
                    template,
                    &mut tokens,
                    &mut placeholders,
                    &mut cursor,
                    &mut text_start,
                    &mut sequential_idx,
                );
            }
            b'{' if !is_backslash_escaped(bytes, cursor) => {
                parse_open_brace(
                    template,
                    &mut tokens,
                    &mut placeholders,
                    &mut named_args,
                    &mut cursor,
                    &mut text_start,
                    &mut sequential_idx,
                );
            }
            b'}' if cursor + 1 < len && bytes[cursor + 1] == b'}' => {
                push_text(template, &mut tokens, text_start, cursor);
                tokens.push(Token::Text("}".into()));
                cursor += 2;
                text_start = cursor;
            }
            _ => cursor += 1,
        }
    }

    if text_start < len {
        tokens.push(Token::Text(template[text_start..].into()));
    }

    let has_tokens = tokens.len() != 1
        || !matches!(tokens.first(), Some(Token::Text(text)) if text.as_ref() == template);

    ParsedTemplate {
        tokens,
        placeholders,
        has_tokens,
    }
}

fn parse_percent_placeholder(
    template: &str,
    tokens: &mut Vec<Token>,
    placeholders: &mut Vec<(usize, SubstitutionRange)>,
    cursor: &mut usize,
    text_start: &mut usize,
    sequential_idx: &mut usize,
) {
    let bytes = template.as_bytes();
    let len = bytes.len();
    let pct = *cursor;

    push_text(template, tokens, *text_start, pct);

    if pct + 1 >= len {
        tokens.push(Token::Text("%".into()));
        *cursor = len;
        *text_start = len;
        return;
    }

    if bytes[pct + 1] == b'%' {
        tokens.push(Token::Text("%".into()));
        *cursor = pct + 2;
        *text_start = *cursor;
        return;
    }

    let mut look = pct + 1;
    let digits_start = look;
    while look < len && bytes[look].is_ascii_digit() {
        look += 1;
    }

    let (arg_idx, end_exclusive) = if look > digits_start && look + 1 < len && bytes[look] == b'$' {
        let idx = template[digits_start..look].parse::<usize>().unwrap_or(1);
        (idx.saturating_sub(1), look + 2)
    } else {
        let idx = *sequential_idx;
        *sequential_idx += 1;
        (idx, pct + 2)
    };

    // Validate the format specifier character (e.g. 's' in %s or %1$s).
    // Non-alphabetic specifiers (typos like %!, %$, or trailing %) are
    // treated as literal percent signs so the error is visible to the user.
    if !bytes[end_exclusive - 1].is_ascii_alphabetic() {
        tokens.push(Token::Text("%".into()));
        *cursor = pct + 1;
        *text_start = *cursor;
        return;
    }

    tokens.push(Token::Var(arg_idx));
    placeholders.push((
        arg_idx,
        SubstitutionRange {
            start: pct,
            end: end_exclusive - 1,
        },
    ));
    *cursor = end_exclusive;
    *text_start = *cursor;
}

fn parse_open_brace<'a>(
    template: &'a str,
    tokens: &mut Vec<Token>,
    placeholders: &mut Vec<(usize, SubstitutionRange)>,
    named_args: &mut HashMap<&'a str, usize>,
    cursor: &mut usize,
    text_start: &mut usize,
    sequential_idx: &mut usize,
) {
    let bytes = template.as_bytes();
    let open = *cursor;

    if open + 1 < bytes.len() && bytes[open + 1] == b'{' {
        push_text(template, tokens, *text_start, open);
        tokens.push(Token::Text("{".into()));
        *cursor = open + 2;
        *text_start = *cursor;
        return;
    }

    let Some(close) = find_closing_brace(bytes, open + 1) else {
        *cursor += 1;
        return;
    };

    let inner = &template[open + 1..close];
    let Some(arg_idx) = brace_arg_index(inner, named_args, sequential_idx) else {
        *cursor += 1;
        return;
    };

    push_text(template, tokens, *text_start, open);
    tokens.push(Token::Var(arg_idx));
    placeholders.push((
        arg_idx,
        SubstitutionRange {
            start: open,
            end: close,
        },
    ));
    *cursor = close + 1;
    *text_start = *cursor;
}

fn brace_arg_index<'a>(
    inner: &'a str,
    named_args: &mut HashMap<&'a str, usize>,
    sequential_idx: &mut usize,
) -> Option<usize> {
    let field = inner.split_once(':').map_or(inner, |(field, _)| field);

    if field.is_empty() {
        let idx = *sequential_idx;
        *sequential_idx += 1;
        return Some(idx);
    }

    if field.bytes().all(|byte| byte.is_ascii_digit()) {
        let idx: usize = field.parse().ok()?;
        *sequential_idx = (*sequential_idx).max(idx + 1);
        return Some(idx);
    }

    if !is_identifier(field) {
        return None;
    }

    if let Some(idx) = named_args.get(field) {
        return Some(*idx);
    }

    let idx = *sequential_idx;
    *sequential_idx += 1;
    named_args.insert(field, idx);
    Some(idx)
}

fn find_closing_brace(bytes: &[u8], start: usize) -> Option<usize> {
    bytes[start..]
        .iter()
        .position(|byte| *byte == b'}')
        .map(|offset| start + offset)
}

fn is_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first == b'_' || first.is_ascii_alphabetic())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

fn is_backslash_escaped(bytes: &[u8], idx: usize) -> bool {
    let mut count = 0;
    let mut pos = idx;
    while pos > 0 && bytes[pos - 1] == b'\\' {
        count += 1;
        pos -= 1;
    }
    // Only escaped when preceded by an odd number of backslashes.
    // e.g. \% → escaped (1 backslash), \\% → not escaped (2 backslashes = literal \)
    count % 2 == 1
}

fn push_text(template: &str, tokens: &mut Vec<Token>, start: usize, end: usize) {
    if end > start {
        tokens.push(Token::Text(template[start..end].into()));
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::format_tokens;

    use super::precompile;

    #[test]
    fn precompile_formats_explicit_sequential_and_literal_percent() {
        let tokens = precompile("%2$s %% %s \\%s end%").unwrap();
        let mut output = String::new();
        format_tokens(&tokens, &["A".to_owned(), "B".to_owned()], &mut output);

        assert_eq!(output, "B % A \\%s end%");
    }

    #[test]
    fn precompile_formats_rust_style_placeholders() {
        let tokens = precompile("Chunk {pos:?} ({stage:?}): {msg}").unwrap();
        let mut output = String::new();
        format_tokens(
            &tokens,
            &["0,0".to_owned(), "Full".to_owned(), "boom".to_owned()],
            &mut output,
        );

        assert_eq!(output, "Chunk 0,0 (Full): boom");
    }

    #[test]
    fn precompile_reuses_named_placeholders_and_unescapes_braces() {
        let tokens = precompile("{} {name} {name} {0} {{ }} %2$s").unwrap();
        let mut output = String::new();
        format_tokens(&tokens, &["A".to_owned(), "B".to_owned()], &mut output);

        assert_eq!(output, "A B B A { } B");
    }
}
