pub use pumpkin_world::time::{AttributeTrack, Easing, KeyFrame, TimeMarker, Timeline};

use pumpkin_codecs::{DataResult, Decode, json_ops::JsonOps};
use pumpkin_util::identifier::Identifier;
use serde_json::Value;

use crate::{
    DatapackError,
    resource::{ResourceManager, list_resources_multi},
};

/// Loads the effective `timeline` registry from the selected datapack stack.
pub fn load_timelines(
    manager: &dyn ResourceManager,
) -> Result<Vec<(Identifier, Timeline)>, DatapackError> {
    let mut entries = Vec::new();

    for namespace in manager.get_namespaces() {
        for path in list_resources_multi(manager, &namespace, &["timeline"]) {
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
                .strip_prefix("timeline/")
                .and_then(|path| path.strip_suffix(".json"))
            else {
                continue;
            };

            let identifier = Identifier::new(namespace.clone(), relative.to_string())?;
            let raw: Value = serde_json::from_slice(&data)?;
            let timeline = match Timeline::parse(raw, &JsonOps) {
                DataResult::Success { result, .. } => result,
                DataResult::Error { message, .. } => {
                    return Err(DatapackError::Codec(format!(
                        "failed to decode timeline {identifier}: {message}"
                    )));
                }
            };

            match timeline.validate_structure() {
                DataResult::Success { .. } => {}
                DataResult::Error { message, .. } => {
                    return Err(DatapackError::Validation(vec![format!(
                        "timeline {identifier}: {message}"
                    )]));
                }
            }

            entries.push((identifier, timeline));
        }
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::{Easing, load_timelines};
    use crate::{
        pack::resource::{PackResources, PathPackResources},
        resource::manager::MultiPackResourceManager,
    };
    use std::{fs, sync::Arc};

    #[test]
    fn loads_timeline_schema() {
        let temp = tempfile::tempdir().expect("temporary datapack directory");
        let dir = temp.path().join("data/test/timeline");
        fs::create_dir_all(&dir).expect("timeline directory");
        fs::write(
            dir.join("day.json"),
            r#"{
                "clock": "minecraft:overworld",
                "period_ticks": 24000,
                "time_markers": {
                    "test:day": {"ticks": 1000, "show_in_commands": true},
                    "test:wakeup": 0
                },
                "tracks": {
                    "minecraft:visual/moon_angle": {
                        "ease": {"cubic_bezier": [0.362, 0.241, 0.638, 0.759]},
                        "keyframes": [
                            {"ticks": 6000, "value": 540.0},
                            {"ticks": 6000, "value": 180.0}
                        ]
                    }
                }
            }"#,
        )
        .expect("timeline");

        let packs: Vec<Arc<dyn PackResources>> =
            vec![Arc::new(PathPackResources::new(temp.path().to_path_buf()))];
        let manager = MultiPackResourceManager::new(&packs);
        let timelines = load_timelines(&manager).expect("timeline must decode");

        assert_eq!(timelines.len(), 1);
        assert_eq!(timelines[0].0.to_string(), "test:day");
        assert_eq!(timelines[0].1.period_ticks, Some(24000));
        assert_eq!(timelines[0].1.time_markers.len(), 2);
        assert!(matches!(
            timelines[0]
                .1
                .tracks
                .values()
                .next()
                .map(|track| track.ease),
            Some(Easing::CubicBezier(_))
        ));
    }

    #[test]
    fn rejects_more_than_two_keyframes_at_same_tick() {
        let temp = tempfile::tempdir().expect("temporary datapack directory");
        let dir = temp.path().join("data/test/timeline");
        fs::create_dir_all(&dir).expect("timeline directory");
        fs::write(
            dir.join("invalid.json"),
            r#"{
                "clock": "minecraft:overworld",
                "period_ticks": 24000,
                "tracks": {
                    "minecraft:visual/moon_angle": {
                        "keyframes": [
                            {"ticks": 6000, "value": 0.0},
                            {"ticks": 6000, "value": 1.0},
                            {"ticks": 6000, "value": 2.0}
                        ]
                    }
                }
            }"#,
        )
        .expect("timeline");

        let packs: Vec<Arc<dyn PackResources>> =
            vec![Arc::new(PathPackResources::new(temp.path().to_path_buf()))];
        let manager = MultiPackResourceManager::new(&packs);
        assert!(load_timelines(&manager).is_err());
    }
}
