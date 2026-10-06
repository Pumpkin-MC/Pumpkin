use pumpkin_core::plugin::error;
use wasmtime::WasmBacktrace;

/// Renders Wasmtime's causes without its execution backtrace.
#[must_use]
pub fn causes(failure: &wasmtime::Error) -> Vec<String> {
    let backtrace = failure
        .downcast_ref::<WasmBacktrace>()
        .map(ToString::to_string);
    error::causes(&**failure, backtrace.as_deref())
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
        let bytes = [
            0, 97, 115, 109, 1, 0, 0, 0, 1, 4, 1, 96, 0, 0, 3, 2, 1, 0, 7, 8, 1, 4, 116, 114, 97,
            112, 0, 0, 10, 5, 1, 3, 0, 0, 11,
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
            Some(&Trap::UnreachableCodeReached)
        );
        assert!(failure.downcast_ref::<WasmBacktrace>().is_some());
        let text = causes(&failure).join("\n");
        assert!(text.contains("unreachable"), "{text}");
        assert!(!text.contains("wasm backtrace"), "{text}");
        assert!(format!("{failure:?}").contains("wasm backtrace"));
    }
}
