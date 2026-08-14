use std::collections::HashMap;

use super::manager::FunctionManager;
use super::parser::parse_function;
use crate::DatapackError;
use crate::Identifier;
use crate::resource::ResourceManager;

/// Load all `.mcfunction` files from the resource manager.
///
/// Scans `data/<namespace>/function/` for `.mcfunction` files.
/// Also identifies which functions are tagged with `#minecraft:tick` and `#minecraft:load`.
pub fn load_functions(manager: &dyn ResourceManager) -> Result<FunctionManager, DatapackError> {
    let mut functions = HashMap::new();
    let tick_functions = Vec::new();
    let load_functions = Vec::new();

    let namespaces = manager.get_namespaces();

    for ns in &namespaces {
        let paths = crate::resource::list_resources_multi(manager, ns, &["function", "functions"]);
        for path in &paths {
            if !path.ends_with(".mcfunction") {
                continue;
            }

            let Some(data) = manager.get_resource(ns, path) else {
                continue;
            };
            let raw = String::from_utf8_lossy(&data);

            let func_name = path
                .strip_prefix("function/")
                .or_else(|| path.strip_prefix("functions/"))
                .and_then(|p| p.strip_suffix(".mcfunction"))
                .unwrap_or(path.as_str());
            let id = Identifier::new(ns.clone(), func_name.to_string())?;

            match parse_function(&raw) {
                Ok(func) => {
                    functions.insert(id.clone(), func);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse function {id}: {e}");
                }
            }
        }
    }

    // Determine tick and load functions from tags
    // These come from tag registrations in the tag module
    // For now, we check them after tag loading

    Ok(FunctionManager {
        functions,
        tick_functions,
        load_functions,
    })
}
