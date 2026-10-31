//! Build-time composition for Pumpkin plugins, regardless of their source language.

use anyhow::{Context, Result, bail};
use wasm_compose::graph::{Component, CompositionGraph, EncodeOptions};
use wasmparser::{Validator, WasmFeatures};

const INTERFACE: &str = "pumpkin:helpers/registry-keys@0.1.0";

/// Embeds registry helpers and preserves the plugin's exports and remaining imports.
///
/// Plugins without helper imports are returned unchanged. Unsupported helper interfaces,
/// invalid components, and incompatible implementations return an error before output is written.
/// This composes components; it does not promise cross-component inlining or dead-code removal.
pub fn compose(plugin: &[u8], helpers: &[u8]) -> Result<Vec<u8>> {
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    let plugin_component = Component::from_bytes(&mut validator, "plugin", plugin)
        .context("invalid plugin component")?;
    for (_, name, _) in plugin_component
        .imports()
        .filter(|(_, name, _)| name.starts_with("pumpkin:helpers/"))
    {
        if name != INTERFACE {
            bail!("unsupported helper interface: {name}");
        }
    }
    let Some((import, _)) = plugin_component.import_by_name(INTERFACE) else {
        return Ok(plugin.to_vec());
    };
    let helpers = Component::from_bytes(&mut validator, "helpers", helpers)
        .context("invalid helper component")?;
    let (export, _, _) = helpers
        .export_by_name(INTERFACE)
        .context("helper component does not export the requested registry interface")?;
    let mut graph = CompositionGraph::new();
    let helper_id = graph.add_component(helpers)?;
    let plugin_id = graph.add_component(plugin_component)?;
    let helper_instance = graph.instantiate(helper_id)?;
    let plugin_instance = graph.instantiate(plugin_id)?;
    graph
        .connect(helper_instance, Some(export), plugin_instance, import)
        .context("helper component does not satisfy the plugin imports")?;
    let bytes = graph
        .encode(EncodeOptions {
            define_components: true,
            export: Some(plugin_instance),
            validate: false,
        })
        .context("failed to encode composed plugin")?;
    Validator::new_with_features(WasmFeatures::all())
        .validate_all(&bytes)
        .context("composed plugin failed validation")?;
    Ok(bytes)
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "invalid fixtures should fail the test")]
mod tests {
    use super::compose;

    #[test]
    fn plugins_without_helper_imports_are_unchanged() {
        let component = b"\0asm\x0d\0\x01\0";
        assert_eq!(
            compose(component, b"unused").ok().as_deref(),
            Some(component.as_slice())
        );
    }

    #[test]
    fn invalid_plugin_is_rejected() {
        assert!(compose(b"not a component", b"unused").is_err());
    }

    #[test]
    fn unsupported_helper_interface_is_rejected() {
        let plugin = wat::parse_str(
            r#"(component (import "pumpkin:helpers/registry-keys@9.0.0" (instance)))"#,
        )
        .expect("valid component");
        let error = compose(&plugin, b"unused").expect_err("unsupported interface");
        assert!(error.to_string().contains("unsupported helper interface"));
    }

    #[test]
    fn incompatible_helper_is_rejected() {
        let plugin = wat::parse_str(
            r#"(component
                (import "pumpkin:helpers/registry-keys@0.1.0" (instance
                    (export "qualify-key" (func (param "key" string) (result string)))
                ))
            )"#,
        )
        .expect("valid plugin");
        let helpers = wat::parse_str(
            r#"(component
                (instance $helpers)
                (export "pumpkin:helpers/registry-keys@0.1.0" (instance $helpers))
            )"#,
        )
        .expect("valid helper");
        let error = compose(&plugin, &helpers).expect_err("missing qualify-key function");
        assert!(error.to_string().contains("does not satisfy"));
    }
}
