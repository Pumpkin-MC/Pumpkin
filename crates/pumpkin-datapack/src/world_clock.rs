pub use pumpkin_world::time::WorldClock;

use pumpkin_codecs::{DataResult, Decode, json_ops::JsonOps};
use pumpkin_util::identifier::Identifier;
use serde_json::Value;

use crate::{
    DatapackError,
    resource::{ResourceManager, list_resources_multi},
};

/// Loads the effective `world_clock` registry from the selected datapack stack.
pub fn load_world_clocks(
    manager: &dyn ResourceManager,
) -> Result<Vec<(Identifier, WorldClock)>, DatapackError> {
    let mut entries = Vec::new();

    for namespace in manager.get_namespaces() {
        for path in list_resources_multi(manager, &namespace, &["world_clock"]) {
            if !std::path::Path::new(&path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                continue;
            }

            let Some(data) = manager.get_resource(&namespace, &path) else {
                continue;
            };
            let Some(relative) = path
                .strip_prefix("world_clock/")
                .and_then(|path| path.strip_suffix(".json"))
            else {
                continue;
            };

            let identifier = Identifier::new(namespace.clone(), relative.to_string())?;
            let raw: Value = serde_json::from_slice(&data)?;
            let world_clock = match WorldClock::parse(raw, &JsonOps) {
                DataResult::Success { result, .. } => result,
                DataResult::Error { message, .. } => {
                    return Err(DatapackError::Codec(format!(
                        "failed to decode world clock {identifier}: {message}"
                    )));
                }
            };
            entries.push((identifier, world_clock));
        }
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::load_world_clocks;
    use crate::{
        pack::resource::{PackResources, PathPackResources},
        resource::manager::MultiPackResourceManager,
    };
    use std::{fs, sync::Arc};

    #[test]
    fn loads_world_clocks_and_rejects_fields() {
        let temp = tempfile::tempdir().expect("temporary datapack directory");
        let dir = temp.path().join("data/test/world_clock");
        fs::create_dir_all(&dir).expect("world clock directory");
        fs::write(dir.join("moon.json"), "{}").expect("world clock");

        let packs: Vec<Arc<dyn PackResources>> =
            vec![Arc::new(PathPackResources::new(temp.path().to_path_buf()))];
        let manager = MultiPackResourceManager::new(&packs);
        let clocks = load_world_clocks(&manager).expect("world clocks must decode");

        assert_eq!(clocks.len(), 1);
        assert_eq!(clocks[0].0.to_string(), "test:moon");

        fs::write(dir.join("invalid.json"), r#"{"rate":1}"#).expect("invalid world clock");
        let manager = MultiPackResourceManager::new(&packs);
        assert!(load_world_clocks(&manager).is_err());
    }
}
