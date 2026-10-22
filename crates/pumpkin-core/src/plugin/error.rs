use std::{error::Error, fmt::Debug, fmt::Write};

const MAX_TEXT_LENGTH: usize = 2048;

/// Removes control characters except newlines and indents continuation lines.
/// Output is limited to 2 KiB, excluding the truncation marker.
#[must_use]
pub fn sanitize(text: &str) -> String {
    sanitize_with_limit(text, MAX_TEXT_LENGTH)
}

/// Sanitizes text using a caller-supplied byte limit.
///
/// The truncation marker is not included in `limit`.
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

/// Formats untrusted text for a single log line.
#[must_use]
pub fn inline(text: &str) -> String {
    sanitize(text).replace('\n', " ")
}

/// Collects sanitized messages from the outer error through its source chain.
///
/// Consecutive duplicates and any message equal to `backtrace` are omitted.
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

/// Formats a headline, labeled causes and optional help.
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

/// Logs sanitized `Debug` output under the supplied headline at DEBUG level.
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
    fn indents_untrusted_lines_and_removes_control_characters() {
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
