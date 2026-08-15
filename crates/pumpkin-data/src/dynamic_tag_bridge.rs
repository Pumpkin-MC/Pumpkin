use std::sync::OnceLock;

/// A function that checks dynamic (datapack) tags.
/// Arguments: (registry_key_str, element_key_str, tag_name_str) -> Option<bool>
pub type DynamicTagChecker = Box<dyn Fn(&str, &str, &str) -> Option<bool> + Send + Sync>;

static DYNAMIC_CHECKER: OnceLock<DynamicTagChecker> = OnceLock::new();

/// Set the global dynamic tag checker.
/// Called once during `Server::new()` to bridge static `Taggable` checks
/// with the datapack `TagRegistry`.
pub fn set_dynamic_tag_checker(checker: DynamicTagChecker) {
    let _ = DYNAMIC_CHECKER.set(checker);
}

/// Check if a dynamic tag exists for the given element.
/// Returns `None` if no dynamic checker is configured,
/// `Some(true/false)` otherwise.
pub fn check_dynamic_tag(registry: &str, element_key: &str, tag_name: &str) -> Option<bool> {
    DYNAMIC_CHECKER
        .get()
        .and_then(|f| f(registry, element_key, tag_name))
}

/// Get the registered tag name from the Tag type.
/// This is a placeholder; the actual name is set at compile time.
pub fn tag_name(registry: &str, values: &[&str], ids: &[u16]) -> Option<&'static str> {
    // This function exists for future dynamic resolution.
    // Currently, the Tag type carries its name via the codegen.
    let _ = (registry, values, ids);
    None
}
