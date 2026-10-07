use pumpkin_core::plugin::error;
use wasmtime::WasmBacktrace;

const MAX_BACKTRACE_LENGTH: usize = 64 * 1024;

/// Renders Wasmtime's causes without its execution backtrace.
#[must_use]
pub fn causes(failure: &wasmtime::Error) -> Vec<String> {
    let backtrace = backtrace(failure).map(ToString::to_string);
    error::causes(&**failure, backtrace.as_deref())
}

/// Finds frames even when the runtime has shared the guest failure with its driver.
#[must_use]
pub fn backtrace(failure: &wasmtime::Error) -> Option<&WasmBacktrace> {
    failure.downcast_ref::<WasmBacktrace>().or_else(|| {
        pumpkin_plugin_runtime::guest_call_error(failure).downcast_ref::<WasmBacktrace>()
    })
}

/// Formats a failure and its readable Wasm frames in one log event.
#[must_use]
pub fn message(
    headline: &str,
    causes: &[String],
    help: Option<&str>,
    failure: Option<&wasmtime::Error>,
) -> String {
    use std::fmt::Write;
    let mut output = error::message(headline, causes, None);
    if let Some(backtrace) = failure.and_then(backtrace)
        && !backtrace.frames().is_empty()
    {
        let rendered = backtrace.to_string();
        if let Some((_, frames)) = rendered.split_once('\n') {
            let _ = write!(
                output,
                "\n  backtrace:\n    {}",
                error::sanitize_with_limit(frames, MAX_BACKTRACE_LENGTH),
            );
        }
    }
    output.push_str(&error::message("", &[], help));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::{Engine, Instance, Module, Store, Trap};

    #[test]
    fn malformed_component_preserves_nested_parser_causes() {
        let bytes = [
            0, 97, 115, 109, 13, 0, 1, 0, 1, 20, 0, 97, 115, 109, 1, 0, 0, 0, 1, 5,
        ];
        let failure = wasmtime::component::Component::new(&Engine::default(), bytes).unwrap_err();
        let layers = causes(&failure);
        let text = layers.join("\n");
        let first = text
            .find("failed to parse WebAssembly module")
            .unwrap_or_else(|| panic!("{text}"));
        let second = text
            .find("wasm component contains an invalid module section")
            .unwrap_or_else(|| panic!("{text}"));
        let third = text.find("section range").unwrap();
        assert!(first < second && second < third, "{text}");
    }

    #[test]
    fn trap_keeps_its_reason_without_the_execution_backtrace() {
        let mut bytes = vec![
            0, 97, 115, 109, 1, 0, 0, 0, 1, 4, 1, 96, 0, 0, 3, 2, 1, 0, 7, 8, 1, 4, 116, 114, 97,
            112, 0, 0, 10, 5, 1, 3, 0, 0, 11,
        ];
        let name = b"_ZN5probe4trap17h0123456789abcdefE";
        let mut functions = vec![1, 0, u8::try_from(name.len()).unwrap()];
        functions.extend_from_slice(name);
        let mut section = vec![
            4,
            b'n',
            b'a',
            b'm',
            b'e',
            1,
            u8::try_from(functions.len()).unwrap(),
        ];
        section.extend(functions);
        bytes.extend([0, u8::try_from(section.len()).unwrap()]);
        bytes.extend(section);
        let engine = Engine::default();
        let module = Module::new(&engine, bytes).unwrap();
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[]).unwrap();
        let failure = instance
            .get_typed_func::<(), ()>(&mut store, "trap")
            .unwrap()
            .call(&mut store, ())
            .unwrap_err();
        assert_eq!(
            failure.downcast_ref::<Trap>(),
            Some(&Trap::UnreachableCodeReached)
        );
        assert!(failure.downcast_ref::<WasmBacktrace>().is_some());
        let text = causes(&failure).join("\n");
        assert!(text.contains("unreachable"), "{text}");
        assert!(!text.contains("wasm backtrace"), "{text}");
        assert!(format!("{failure:?}").contains("wasm backtrace"));
        let rendered = message("Plugin failed.", &causes(&failure), None, Some(&failure));
        assert!(
            rendered.contains("  caused by: wasm trap: wasm `unreachable` instruction executed"),
            "{rendered}"
        );
        assert!(rendered.contains("\n  backtrace:"), "{rendered}");
        assert!(rendered.contains("probe::trap"), "{rendered}");
        assert!(!rendered.contains("_ZN5probe"), "{rendered}");
    }
}
