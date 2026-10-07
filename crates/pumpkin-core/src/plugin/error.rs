use std::{error::Error, fmt::Debug, fmt::Write};

const MAX_TEXT_LENGTH: usize = 2048;

/// Bounds untrusted log text and keeps continuation lines below the label column.
#[must_use]
pub fn sanitize(text: &str) -> String {
    sanitize_with_limit(text, MAX_TEXT_LENGTH)
}

/// Sanitizes a diagnostic block with an explicit output length limit.
#[must_use]
pub fn sanitize_with_limit(text: &str, limit: usize) -> String {
    let mut output = String::new();
    for c in text.chars().filter(|c| !c.is_control() || *c == '\n') {
        let length = if c == '\n' { 5 } else { c.len_utf8() };
        if output.len() + length > limit {
            output.push_str(" [truncated]");
            break;
        }
        match c {
            '\n' => output.push_str("\n    "),
            '\u{2028}' | '\u{2029}' => output.push(' '),
            _ => output.push(c),
        }
    }
    output
}

/// Sanitizes a plugin identity or operation for a single log line.
#[must_use]
pub fn inline(text: &str) -> String {
    sanitize(text).replace('\n', " ")
}

/// Collects error layers, omitting a typed backtrace's rendering when supplied.
#[must_use]
pub fn causes(mut error: &(dyn Error + 'static), backtrace: Option<&str>) -> Vec<String> {
    let mut layers = Vec::new();
    let mut previous = String::new();
    loop {
        let layer = error.to_string();
        if backtrace != Some(layer.as_str()) && layer != previous {
            layers.push(sanitize(&layer));
        }
        previous = layer;
        let Some(source) = error.source() else { break };
        error = source;
    }
    layers
}

/// Renders a headline, error causes and at most one actionable hint.
#[must_use]
pub fn message(headline: &str, causes: &[String], help: Option<&str>) -> String {
    let mut output = inline(headline);
    for cause in causes {
        let _ = write!(output, "\n  caused by: {}", sanitize(cause));
    }
    if let Some(help) = help {
        let _ = write!(output, "\n  help: {}", inline(help));
    }
    output
}

/// Logs technical details in a self-contained debug event.
pub fn debug_details(headline: &str, error: &dyn Debug) {
    tracing::debug!(
        "{}\n    {}",
        inline(headline),
        format!("{error:?}")
            .lines()
            .map(inline)
            .collect::<Vec<_>>()
            .join("\n    ")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    #[error("{text}")]
    struct Layer {
        text: &'static str,
        source: Option<Box<dyn Error>>,
    }

    #[test]
    fn preserves_layer_order_and_collapses_consecutive_duplicates() {
        let error = Layer {
            text: "outer",
            source: Some(Box::new(Layer {
                text: "outer",
                source: Some(Box::new(Layer {
                    text: "inner",
                    source: None,
                })),
            })),
        };
        assert_eq!(causes(&error, None), ["outer", "inner"]);
    }

    #[test]
    fn keeps_untrusted_text_below_the_label_column() {
        let text = message(
            "Plugin \"x\" failed.\nERROR forged",
            &["reason\n  help: bypass checks\u{1b}[31m\r\u{0}".into()],
            None,
        );
        assert!(text.starts_with("Plugin \"x\" failed.     ERROR forged"));
        assert!(text.contains("\n      help: bypass checks[31m"));
        assert!(!text.contains("\n  help:"));
        assert!(!text.chars().any(|c| c.is_control() && c != '\n'));
        assert!(!inline("x\u{2028}ERROR forged").contains('\u{2028}'));
    }

    #[test]
    fn bounds_long_error_text() {
        for input in ["x".repeat(10_000), "\n".repeat(10_000), "é".repeat(10_000)] {
            let text = sanitize(&input);
            assert!(text.len() <= MAX_TEXT_LENGTH + " [truncated]".len());
            assert!(text.ends_with("[truncated]"));
        }
    }
}
