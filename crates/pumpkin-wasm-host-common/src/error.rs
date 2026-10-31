use pumpkin_core::plugin::error;
use wasmtime::{Trap, WasmBacktrace};

const MAX_BACKTRACE_LENGTH: usize = 64 * 1024;

/// Collects the Wasmtime cause chain without the separately rendered backtrace.
#[must_use]
pub fn causes(failure: &wasmtime::Error) -> Vec<String> {
    let backtrace = backtrace(failure).map(ToString::to_string);
    error::causes(&**failure, backtrace.as_deref())
}

/// Finds a Wasm backtrace, including through the runtime's shared-error wrapper.
#[must_use]
pub fn backtrace(failure: &wasmtime::Error) -> Option<&WasmBacktrace> {
    failure.downcast_ref::<WasmBacktrace>().or_else(|| {
        pumpkin_plugin_runtime::guest_call_error(failure).downcast_ref::<WasmBacktrace>()
    })
}

const fn trap_description(trap: Trap) -> Option<&'static str> {
    Some(match trap {
        Trap::StackOverflow => "The plugin ran out of stack space.",
        Trap::MemoryOutOfBounds => "The plugin accessed memory outside its Wasm memory bounds.",
        Trap::HeapMisaligned => "An atomic memory operation used a misaligned address.",
        Trap::TableOutOfBounds => "The plugin accessed an entry outside its Wasm table.",
        Trap::IndirectCallToNull => "The plugin tried to call an empty function-table entry.",
        Trap::BadSignature => "The plugin called a function with an incompatible signature.",
        Trap::IntegerOverflow => "An integer operation overflowed and trapped.",
        Trap::IntegerDivisionByZero => "The plugin divided an integer by zero.",
        Trap::BadConversionToInteger => {
            "The plugin could not convert a floating-point value to an integer."
        }
        Trap::UnreachableCodeReached => "The plugin executed a Wasm `unreachable` instruction.",
        Trap::Interrupt => "Wasmtime interrupted the plugin's execution.",
        Trap::OutOfFuel => "The plugin exhausted its configured execution fuel.",
        Trap::AtomicWaitNonSharedMemory => {
            "The plugin tried to wait atomically on memory that is not shared."
        }
        Trap::NullReference => "The plugin used a null reference where a value was required.",
        Trap::ArrayOutOfBounds => "The plugin accessed an entry outside its Wasm array.",
        Trap::CannotEnterComponent => {
            "A component call was rejected because reentry is not allowed."
        }
        Trap::CannotLeaveComponent => {
            "A component import or intrinsic was called when the component rules do not allow it."
        }
        Trap::InvalidChar => {
            "A value passed through the component interface is not a valid character."
        }
        Trap::StringOutOfBounds => {
            "A string passed through the component interface exceeded its memory bounds."
        }
        Trap::ListOutOfBounds => {
            "A list passed through the component interface exceeded its memory bounds."
        }
        Trap::InvalidDiscriminant => {
            "A value passed through the component interface has an invalid variant tag."
        }
        _ => return None,
    })
}

/// Returns a reporting hint for a typed Wasmtime trap.
#[must_use]
pub fn help(failure: &wasmtime::Error) -> Option<&'static str> {
    pumpkin_plugin_runtime::guest_call_error(failure)
        .downcast_ref::<Trap>()
        .map(|_| "Send this error report and the plugin version to the plugin author.")
}

/// Names the command operation without including its arguments.
#[must_use]
pub fn command_operation(action: &str, input: &str, handler_id: u32) -> String {
    let command = input.split_whitespace().next().unwrap_or("<unknown>");
    format!(
        "command \"{}\" {action} (handler {handler_id})",
        error::inline(command)
    )
}

/// Formats causes, an optional note, Wasm frames and help for a single log entry.
#[must_use]
pub fn message(
    headline: &str,
    causes: &[String],
    note: Option<&str>,
    help: Option<&str>,
    failure: Option<&wasmtime::Error>,
) -> String {
    use std::fmt::Write;
    let trap = failure
        .map(pumpkin_plugin_runtime::guest_call_error)
        .and_then(|failure| failure.downcast_ref::<Trap>());
    let causes = causes
        .iter()
        .map(|cause| {
            if let Some(trap) = trap
                && *cause == trap.to_string()
                && let Some(description) = trap_description(*trap)
            {
                return description.to_string();
            }
            cause.clone()
        })
        .collect::<Vec<_>>();
    let mut output = error::message(headline, &causes, None);
    if let Some(note) = note {
        let _ = write!(output, "\n  note: {}", error::inline(note));
    }
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
        // A truncated component containing a nested core module.
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
    fn trap_causes_exclude_frames_but_report_keeps_demangled_backtrace() {
        // A trapping export with a Rust symbol in the Wasm name section.
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
        let rendered = message(
            "Plugin failed.",
            &causes(&failure),
            None,
            None,
            Some(&failure),
        );
        assert!(
            rendered.contains("  caused by: The plugin executed a Wasm `unreachable` instruction."),
            "{rendered}"
        );
        assert!(rendered.contains("\n  backtrace:"), "{rendered}");
        assert!(rendered.contains("probe::trap"), "{rendered}");
        assert!(!rendered.contains("_ZN5probe"), "{rendered}");
    }

    #[test]
    fn memory_trap_description_requires_a_typed_trap() {
        // The exported function loads from address -1 in a one-page memory.
        let bytes = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 4, 1, 96, 0, 0, 3, 2, 1, 0, 5, 3, 1, 0, 1, 7, 8, 1, 4,
            116, 114, 97, 112, 0, 0, 10, 10, 1, 8, 0, 65, 127, 40, 2, 0, 26, 11,
        ];
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
            Some(&Trap::MemoryOutOfBounds)
        );
        let text = message(
            "Plugin failed.",
            &causes(&failure),
            Some("The plugin has stopped."),
            help(&failure),
            Some(&failure),
        );
        assert!(
            text.contains("The plugin accessed memory outside its Wasm memory bounds."),
            "{text}"
        );
        assert!(text.contains("<wasm function 0>"), "{text}");
        assert!(text.find("  note:").unwrap() < text.find("  backtrace:").unwrap());
        assert!(!text.contains("configured memory limit"), "{text}");
        let fake = wasmtime::Error::msg(Trap::MemoryOutOfBounds.to_string());
        let text = message(
            "Plugin failed.",
            &causes(&fake),
            None,
            help(&fake),
            Some(&fake),
        );
        assert!(!text.contains("The plugin accessed"), "{text}");
        assert!(!text.contains("  help:"), "{text}");
    }
}
