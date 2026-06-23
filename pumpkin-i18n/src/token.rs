use std::sync::Arc;

/// A precompiled token in a translation format template.
///
/// During startup, every translation string containing `%` placeholders
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
///
/// Returns `None` if the string contains no `%` placeholders.
///
/// # Examples
/// ```ignore
/// let tokens = precompile("Hello %s, you have %d messages").unwrap();
/// // → [Text("Hello "), Var(0), Text(", you have "), Var(1), Text(" messages")]
/// ```
#[must_use]
pub fn precompile(template: &str) -> Option<TokenStream> {
    let bytes = template.as_bytes();
    let len = bytes.len();

    // Quick check: does this string contain any placeholders?
    if !bytes.contains(&b'%') {
        return None;
    }

    let mut tokens: Vec<Token> = Vec::new();
    let mut cursor = 0usize;
    let mut text_start = 0usize;
    let mut sequential_idx = 0usize;

    while cursor < len {
        if bytes[cursor] != b'%' {
            cursor += 1;
            continue;
        }

        if cursor > 0 && bytes[cursor - 1] == b'\\' {
            cursor += 1;
            continue;
        }

        let pct = cursor;
        if pct > text_start {
            tokens.push(Token::Text(template[text_start..pct].into()));
        }

        if pct + 1 >= len {
            tokens.push(Token::Text("%".into()));
            text_start = len;
            break;
        }

        if pct + 1 < len && bytes[pct + 1] == b'%' {
            tokens.push(Token::Text("%".into()));
            cursor = pct + 2;
            text_start = cursor;
            continue;
        }

        let mut look = pct + 1;
        let digits_start = look;
        while look < len && bytes[look].is_ascii_digit() {
            look += 1;
        }

        if look > digits_start && look + 1 < len && bytes[look] == b'$' {
            // Explicit index: %1$s, %2$d, …
            let idx = template[digits_start..look].parse::<usize>().unwrap_or(1);
            tokens.push(Token::Var(idx.saturating_sub(1)));
            cursor = look + 2;
        } else {
            // Sequential index: %s, %d, %f, …
            tokens.push(Token::Var(sequential_idx));
            sequential_idx += 1;
            cursor = pct + 2;
        }

        text_start = cursor;
    }

    if text_start < len {
        tokens.push(Token::Text(template[text_start..].into()));
    }

    if tokens.is_empty() {
        None
    } else {
        Some(tokens.into())
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
}
