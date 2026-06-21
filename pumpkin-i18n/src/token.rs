use std::sync::Arc;

/// A precompiled token in a translation format template.
///
/// During startup, every translation string containing `%` placeholders
/// is parsed into a sequence of [`Token`]s so that runtime substitution
/// does zero parsing work — it simply streams the tokens into a buffer.
#[derive(Clone, Debug)]
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
    let mut sequential_idx = 0usize;

    while cursor < len {
        let pct = if let Some(pos) = bytes[cursor..].iter().position(|&b| b == b'%') {
            cursor + pos
        } else {
            // Remainder is plain text
            if cursor < len {
                tokens.push(Token::Text(template[cursor..].into()));
            }
            break;
        };

        // Emit text before the %
        if pct > cursor {
            tokens.push(Token::Text(template[cursor..pct].into()));
        }

        // Check for %% (escaped literal percent)
        if pct + 1 < len && bytes[pct + 1] == b'%' {
            tokens.push(Token::Text("%".into()));
            cursor = pct + 2;
            continue;
        }

        // We have a placeholder. Scan for a possible digit prefix + '$'.
        let mut num_str = String::new();
        let mut look = pct + 1;
        while look < len && bytes[look].is_ascii_digit() {
            num_str.push(bytes[look] as char);
            look += 1;
        }

        if look < len && bytes[look] == b'$' {
            // Explicit index: %1$s, %2$d, …
            let idx: usize = num_str.parse().unwrap_or(1);
            // 1‑based → 0‑based
            tokens.push(Token::Var(idx.saturating_sub(1)));
            // Skip the format specifier char (s/d/f/…)
            cursor = look + 2.min(len.saturating_sub(look));
        } else {
            // Sequential index: %s, %d, %f, …
            tokens.push(Token::Var(sequential_idx));
            sequential_idx += 1;
            // Skip the format specifier char
            cursor = pct + 2.min(len);
        }
    }

    if tokens.is_empty() {
        None
    } else {
        Some(tokens.into())
    }
}
