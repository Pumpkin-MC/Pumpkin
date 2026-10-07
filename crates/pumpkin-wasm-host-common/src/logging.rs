use tracing_serde_structured::{DebugRecord, SerializeValue};

use crate::state::PluginHostState;
use pumpkin_core::plugin::error;

/// Adds the available plugin identity and sanitizes the message.
///
/// Uses the filename until plugin metadata is available.
#[must_use]
pub fn format_plugin_message(state: &PluginHostState, text: &str) -> String {
    let identity = state
        .plugin
        .as_ref()
        .and_then(std::sync::Weak::upgrade)
        .map_or_else(
            || match (&state.name, &state.plugin_file) {
                (Some(name), _) => format!("\"{}\"", error::inline(name)),
                (_, Some(file)) => format!(
                    "file \"{}\"",
                    error::inline(
                        &file
                            .file_name()
                            .map_or(file.as_path(), std::path::Path::new)
                            .display()
                            .to_string()
                    )
                ),
                _ => "being initialized".into(),
            },
            |plugin| {
                format!(
                    "\"{}\" ({})",
                    error::inline(&plugin.name),
                    error::inline(&plugin.version)
                )
            },
        );
    format!("Plugin {identity} reported: {}", error::sanitize(text))
}

pub fn log_tracing(event_bytes: &[u8]) {
    let event: tracing_serde_structured::SerializeEvent<'_> =
        match postcard::from_bytes(event_bytes) {
            Ok(e) => e,
            Err(e) => {
                tracing::error!("[plugin] failed to deserialize tracing event: {e}");
                return;
            }
        };

    let message = match &event.fields {
        tracing_serde_structured::SerializeRecordFields::De(map) => map
            .get(&tracing_serde_structured::CowString::Borrowed("message"))
            .map(|v| match v {
                SerializeValue::Debug(d) => match d {
                    DebugRecord::De(s) => s.as_str().to_string(),
                    DebugRecord::Ser(_) => String::new(),
                },
                SerializeValue::Str(s) => s.as_str().to_string(),
                _ => String::new(),
            })
            .unwrap_or_default(),
        tracing_serde_structured::SerializeRecordFields::Ser(_) => String::new(),
    };

    let target = event.metadata.target.as_str().to_string();
    let module_path = event
        .metadata
        .module_path
        .as_deref()
        .unwrap_or("")
        .to_string();
    let file = event
        .metadata
        .file
        .as_deref()
        .unwrap_or("unknown")
        .to_string();
    let line = event.metadata.line.unwrap_or(0);

    let extra: Vec<(&str, &tracing_serde_structured::SerializeValue)> = match &event.fields {
        tracing_serde_structured::SerializeRecordFields::De(map) => map
            .iter()
            .filter(|(k, _)| k.as_str() != "message")
            .map(|(k, v)| (k.as_str(), v))
            .collect(),
        tracing_serde_structured::SerializeRecordFields::Ser(_) => vec![],
    };

    let fields_str = if extra.is_empty() {
        None
    } else {
        Some(
            extra
                .iter()
                .map(|(k, v)| format!("{k}={}", format_value(v)))
                .collect::<Vec<_>>()
                .join(", "),
        )
    };

    macro_rules! emit {
        ($level:expr) => {
            match &fields_str {
                None => {
                    tracing::event!(
                        $level,
                        plugin.target = %target,
                        plugin.module = %module_path,
                        plugin.file = %file,
                        plugin.line = line,
                        "{message}"
                    );
                }
                Some(fields) => {
                    tracing::event!(
                        $level,
                        plugin.target = %target,
                        plugin.module = %module_path,
                        plugin.file = %file,
                        plugin.line = line,
                        plugin.fields = %fields,
                        "{message}"
                    );
                }
            }
        };
    }

    match event.metadata.level {
        tracing_serde_structured::SerializeLevel::Trace => emit!(tracing::Level::TRACE),
        tracing_serde_structured::SerializeLevel::Debug => emit!(tracing::Level::DEBUG),
        tracing_serde_structured::SerializeLevel::Info => emit!(tracing::Level::INFO),
        tracing_serde_structured::SerializeLevel::Warn => emit!(tracing::Level::WARN),
        tracing_serde_structured::SerializeLevel::Error => emit!(tracing::Level::ERROR),
    }
}

fn format_value(v: &tracing_serde_structured::SerializeValue) -> String {
    match v {
        SerializeValue::Debug(d) => match d {
            DebugRecord::De(s) => s.as_str().to_string(),
            DebugRecord::Ser(args) => format!("{args:?}"),
        },
        SerializeValue::Str(s) => s.as_str().to_string(),
        SerializeValue::F64(f) => f.to_string(),
        SerializeValue::I64(i) => i.to_string(),
        SerializeValue::U64(u) => u.to_string(),
        SerializeValue::Bool(b) => b.to_string(),
        _ => String::from("<unknown>"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_logs_identify_the_source_and_indent_untrusted_lines() {
        let mut state = PluginHostState::new();
        state.plugin_file = Some("probe.wasm".into());
        assert_eq!(
            format_plugin_message(&state, "Rust panic"),
            "Plugin file \"probe.wasm\" reported: Rust panic"
        );
        state.name = Some("probe\nERROR forged\u{1b}[31m".into());
        let text = format_plugin_message(&state, "Rust panic\nERROR forged\r\u{0}");
        assert!(!text.contains("\nERROR"), "{text}");
        assert!(
            text.contains("reported: Rust panic\n    ERROR forged"),
            "{text}"
        );
        assert!(!text.chars().any(|c| c.is_control() && c != '\n'));
        let text = format_plugin_message(&state, &"x".repeat(10_000));
        assert!(text.ends_with("[truncated]"), "{text}");
    }
}
