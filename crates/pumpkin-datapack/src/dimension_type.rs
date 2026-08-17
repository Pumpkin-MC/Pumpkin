pub use pumpkin_world::dimension_type::{DimensionType, SkyBox, UnresolvedIntProvider};

use pumpkin_codecs::{DataResult, Decode, json_ops::JsonOps};
use pumpkin_util::identifier::Identifier;
use serde_json::Value;

use crate::{
    DatapackError,
    resource::{ResourceManager, list_resources_multi},
};

/// Loads the effective `dimension_type` registry from the selected datapack stack.
pub fn load_dimension_types(
    manager: &dyn ResourceManager,
) -> Result<Vec<(Identifier, DimensionType)>, DatapackError> {
    let mut entries = Vec::new();

    for namespace in manager.get_namespaces() {
        for path in list_resources_multi(manager, &namespace, &["dimension_type"]) {
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
                .strip_prefix("dimension_type/")
                .and_then(|path| path.strip_suffix(".json"))
            else {
                continue;
            };

            let identifier = Identifier::new(namespace.clone(), relative.to_string())?;
            let raw: Value = serde_json::from_slice(&data)?;
            let dimension_type = match DimensionType::parse(raw, &JsonOps) {
                DataResult::Success { result, .. } => result,
                DataResult::Error { message, .. } => {
                    return Err(DatapackError::Codec(format!(
                        "failed to decode dimension type {identifier}: {message}"
                    )));
                }
            };
            validate_dimension_type(&dimension_type)?;
            entries.push((identifier, dimension_type));
        }
    }

    Ok(entries)
}

fn validate_dimension_type(dimension_type: &DimensionType) -> Result<(), DatapackError> {
    let mut errors = Vec::new();

    if !(-2032..=2016).contains(&dimension_type.min_y) || dimension_type.min_y % 16 != 0 {
        errors.push(format!(
            "dimension type min_y must be between -2032 and 2016 and divisible by 16, got {}",
            dimension_type.min_y
        ));
    }

    if !(16..=4064).contains(&dimension_type.height) || dimension_type.height % 16 != 0 {
        errors.push(format!(
            "dimension type height must be between 16 and 4064 and divisible by 16, got {}",
            dimension_type.height
        ));
    }

    if dimension_type.logical_height <= 0 || dimension_type.logical_height > dimension_type.height {
        errors.push(format!(
            "dimension type logical_height must be positive and no greater than height ({}), got {}",
            dimension_type.height, dimension_type.logical_height
        ));
    }

    if dimension_type.monster_spawn_block_light_limit > 15 {
        errors.push(format!(
            "dimension type monster_spawn_block_light_limit must be at most 15, got {}",
            dimension_type.monster_spawn_block_light_limit
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(DatapackError::Validation(errors))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::{DimensionType, load_dimension_types};
    use crate::{
        pack::resource::{PackResources, PathPackResources, VanillaPackResources},
        resource::manager::MultiPackResourceManager,
    };
    use pumpkin_codecs::{Decode, Encode, json_ops::JsonOps};
    use std::{fs, sync::Arc};

    fn dimension_json(ambient_light: f32) -> String {
        format!(
            r##"{{
                "ambient_light": {ambient_light},
                "attributes": {{}},
                "coordinate_scale": 1.0,
                "has_ceiling": false,
                "has_ender_dragon_fight": false,
                "has_skylight": true,
                "height": 384,
                "infiniburn": "#minecraft:infiniburn_overworld",
                "logical_height": 384,
                "min_y": -64,
                "monster_spawn_block_light_limit": 0,
                "monster_spawn_light_level": {{
                    "type": "minecraft:uniform",
                    "max_inclusive": 7,
                    "min_inclusive": 0
                }},
                "timelines": "#minecraft:in_overworld"
            }}"##
        )
    }

    #[test]
    fn vanilla_dimension_types_are_loaded_from_pack_resources() {
        let packs: Vec<Arc<dyn PackResources>> = vec![Arc::new(VanillaPackResources)];
        let manager = MultiPackResourceManager::new(&packs);
        let entries = load_dimension_types(&manager).expect("vanilla dimension types must decode");

        let overworld = entries
            .iter()
            .find(|(id, _)| id.to_string() == "minecraft:overworld")
            .map(|(_, dimension_type)| dimension_type)
            .expect("vanilla overworld dimension type must exist");

        assert_eq!(overworld.min_y, -64);
        assert_eq!(overworld.height, 384);
        assert!(overworld.has_skylight);
        assert!(matches!(
            overworld.infiniburn,
            pumpkin_registry::RegistryResolvableSet::Tag(ref value)
                if value.to_string() == "minecraft:infiniburn_overworld"
        ));
        assert_eq!(
            overworld
                .default_clock
                .as_ref()
                .map(|clock| clock.identifier().to_string()),
            Some("minecraft:overworld".to_string())
        );
        assert!(overworld.attributes.keys().any(|attribute| {
            attribute.identifier()
                == &pumpkin_util::identifier::Identifier::parse_static(
                    "minecraft:gameplay/bed_rule",
                )
        }));

        let the_end = entries
            .iter()
            .find(|(id, _)| id.to_string() == "minecraft:the_end")
            .map(|(_, dimension_type)| dimension_type)
            .expect("vanilla end dimension type must exist");
        assert_eq!(the_end.skybox, Some(super::SkyBox::End));

        let the_nether = entries
            .iter()
            .find(|(id, _)| id.to_string() == "minecraft:the_nether")
            .map(|(_, dimension_type)| dimension_type)
            .expect("vanilla nether dimension type must exist");
        assert_eq!(the_nether.skybox, Some(super::SkyBox::None));
        assert_eq!(
            the_nether.cardinal_light,
            Some(pumpkin_world::cardinal_lighting::NETHER)
        );
        assert!(matches!(
            the_nether.timelines,
            Some(pumpkin_registry::RegistryResolvableSet::Tag(ref value))
                if value.to_string() == "minecraft:in_nether"
        ));
        assert!(the_nether.default_clock.is_none());
    }

    #[test]
    fn higher_priority_pack_overrides_and_adds_dimension_types() {
        let temp = tempfile::tempdir().expect("temporary datapack directory");
        let minecraft = temp.path().join("data/minecraft/dimension_type");
        let custom = temp.path().join("data/test/dimension_type");
        fs::create_dir_all(&minecraft).expect("minecraft dimension directory");
        fs::create_dir_all(&custom).expect("custom dimension directory");
        fs::write(minecraft.join("overworld.json"), dimension_json(0.75))
            .expect("overworld override");
        fs::write(custom.join("moon.json"), dimension_json(0.5)).expect("custom dimension");

        let packs: Vec<Arc<dyn PackResources>> = vec![
            Arc::new(VanillaPackResources),
            Arc::new(PathPackResources::new(temp.path().to_path_buf())),
        ];
        let manager = MultiPackResourceManager::new(&packs);
        let entries = load_dimension_types(&manager).expect("stacked dimension types must decode");

        let overworld = entries
            .iter()
            .find(|(id, _)| id.to_string() == "minecraft:overworld")
            .map(|(_, dimension_type)| dimension_type)
            .expect("overridden overworld must exist");
        assert_eq!(overworld.ambient_light, 0.75);

        assert!(entries.iter().any(|(id, _)| id.to_string() == "test:moon"));
    }

    #[test]
    fn infiniburn_accepts_a_list_in_26_2() {
        let json = r#"{
            "ambient_light": 0.0,
            "attributes": {},
            "coordinate_scale": 1.0,
            "has_ceiling": false,
            "has_ender_dragon_fight": false,
            "has_skylight": true,
            "height": 384,
            "infiniburn": ["minecraft:netherrack", "minecraft:magma_block"],
            "logical_height": 384,
            "min_y": -64,
            "monster_spawn_block_light_limit": 0,
            "monster_spawn_light_level": 7
        }"#;

        let raw: serde_json::Value = serde_json::from_str(json).expect("valid dimension JSON");
        let dimension_type = DimensionType::parse(raw, &JsonOps)
            .into_result()
            .expect("dimension type must decode");

        assert!(matches!(
            dimension_type.infiniburn,
            pumpkin_registry::RegistryResolvableSet::List(ref values) if values.len() == 2
        ));
    }

    #[test]
    fn dimension_type_round_trips_through_json_ops() {
        let raw: serde_json::Value =
            serde_json::from_str(&dimension_json(0.25)).expect("valid dimension JSON");
        let decoded = DimensionType::parse(raw.clone(), &JsonOps)
            .into_result()
            .expect("dimension type must decode");
        let encoded = decoded
            .encode_start(&JsonOps)
            .into_result()
            .expect("dimension type must encode");

        assert_eq!(encoded, raw);
    }
}
