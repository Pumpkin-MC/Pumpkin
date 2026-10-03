use std::collections::HashMap;
use std::fs;
use std::path::Path;

use pumpkin_data::damage::{DamageEffects, DamageScaling, DamageType, DeathMessageType};
use pumpkin_data::registry::RegistryEntryData;
use pumpkin_nbt::{Nbt, NbtCompound};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageScalingWrapper {
    Never,
    WhenCausedByLivingNonPlayer,
    Always,
}

impl From<DamageScalingWrapper> for DamageScaling {
    fn from(wrapper: DamageScalingWrapper) -> Self {
        match wrapper {
            DamageScalingWrapper::Never => Self::Never,
            DamageScalingWrapper::WhenCausedByLivingNonPlayer => Self::WhenCausedByLivingNonPlayer,
            DamageScalingWrapper::Always => Self::Always,
        }
    }
}

impl DamageScalingWrapper {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::WhenCausedByLivingNonPlayer => "when_caused_by_living_non_player",
            Self::Always => "always",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageEffectsWrapper {
    Hurt,
    Thorns,
    Drowning,
    Burning,
    Poking,
    Freezing,
}

impl From<DamageEffectsWrapper> for DamageEffects {
    fn from(wrapper: DamageEffectsWrapper) -> Self {
        match wrapper {
            DamageEffectsWrapper::Hurt => Self::Hurt,
            DamageEffectsWrapper::Thorns => Self::Thorns,
            DamageEffectsWrapper::Drowning => Self::Drowning,
            DamageEffectsWrapper::Burning => Self::Burning,
            DamageEffectsWrapper::Poking => Self::Poking,
            DamageEffectsWrapper::Freezing => Self::Freezing,
        }
    }
}

impl DamageEffectsWrapper {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Hurt => "hurt",
            Self::Thorns => "thorns",
            Self::Drowning => "drowning",
            Self::Burning => "burning",
            Self::Poking => "poking",
            Self::Freezing => "freezing",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeathMessageTypeWrapper {
    Default,
    FallVariants,
    IntentionalGameDesign,
}

impl From<DeathMessageTypeWrapper> for DeathMessageType {
    fn from(wrapper: DeathMessageTypeWrapper) -> Self {
        match wrapper {
            DeathMessageTypeWrapper::Default => Self::Default,
            DeathMessageTypeWrapper::FallVariants => Self::FallVariants,
            DeathMessageTypeWrapper::IntentionalGameDesign => Self::IntentionalGameDesign,
        }
    }
}

impl DeathMessageTypeWrapper {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::FallVariants => "fall_variants",
            Self::IntentionalGameDesign => "intentional_game_design",
        }
    }
}

/// A parsed damage type definition from a datapack JSON file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DamageTypeDefinition {
    pub message_id: String,
    pub scaling: DamageScalingWrapper,
    #[serde(default)]
    pub exhaustion: f32,
    #[serde(default)]
    pub effects: Option<DamageEffectsWrapper>,
    #[serde(default)]
    pub death_message_type: Option<DeathMessageTypeWrapper>,
}

/// A registered damage type in the runtime registry.
#[derive(Clone, Debug)]
pub struct DamageTypeEntry {
    pub id: String,
    pub numeric_id: u8,
    pub definition: DamageTypeDefinition,
    pub damage_type: DamageType,
    pub nbt_data: Box<[u8]>,
}

impl DamageTypeEntry {
    #[must_use]
    pub fn to_registry_entry(&self) -> RegistryEntryData {
        RegistryEntryData {
            entry_id: self.id.clone(),
            data: Some(self.nbt_data.clone()),
        }
    }
}

pub type DamageTypeRegistry = HashMap<String, DamageTypeEntry>;

/// Encodes a `DamageTypeDefinition` into a `pumpkin_data::damage::DamageType`.
#[must_use]
pub fn to_damage_type(numeric_id: u8, def: &DamageTypeDefinition) -> DamageType {
    DamageType {
        death_message_type: def
            .death_message_type
            .map_or(DeathMessageType::Default, Into::into),
        exhaustion: def.exhaustion,
        effects: def.effects.map(Into::into),
        message_id: Box::leak(def.message_id.clone().into_boxed_str()),
        scaling: def.scaling.into(),
        id: numeric_id,
    }
}

/// Encodes the damage type definition into binary NBT payload.
#[must_use]
pub fn to_registry_nbt_bytes(def: &DamageTypeDefinition) -> Box<[u8]> {
    let mut nbt = NbtCompound::new();
    nbt.put_string("message_id", def.message_id.clone());
    nbt.put_string("scaling", def.scaling.as_str().to_string());
    nbt.put_double("exhaustion", f64::from(def.exhaustion));

    if let Some(effects) = def.effects {
        nbt.put_string("effects", effects.as_str().to_string());
    }

    if let Some(dmt) = def.death_message_type
        && dmt != DeathMessageTypeWrapper::Default
    {
        nbt.put_string("death_message_type", dmt.as_str().to_string());
    }

    Nbt::from(nbt).write_unnamed().to_vec().into_boxed_slice()
}

/// Encodes the damage type definition into a `RegistryEntryData` with binary NBT payload.
#[must_use]
pub fn to_registry_entry(entry_id: String, def: &DamageTypeDefinition) -> RegistryEntryData {
    RegistryEntryData {
        entry_id,
        data: Some(to_registry_nbt_bytes(def)),
    }
}

/// Loads all damage type JSON definitions from a directory into the definition registry.
pub fn load_damage_types_from_dir<S: std::hash::BuildHasher>(
    namespace: &str,
    dir: &Path,
    registry: &mut HashMap<String, DamageTypeDefinition, S>,
) -> usize {
    if !dir.is_dir() {
        return 0;
    }
    let before = registry.len();
    load_damage_types_recursive(namespace, dir, dir, registry);
    registry.len() - before
}

fn load_damage_types_recursive<S: std::hash::BuildHasher>(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    registry: &mut HashMap<String, DamageTypeDefinition, S>,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_damage_types_recursive(namespace, base_dir, &path, registry);
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            && let Ok(rel_path) = path.strip_prefix(base_dir)
        {
            let mut stem_path = rel_path.to_string_lossy().to_string();
            if let Some(stem) = stem_path.strip_suffix(".json") {
                stem_path = stem.to_string();
            }
            let stem_path = stem_path.replace('\\', "/");
            let damage_type_id = format!("{namespace}:{stem_path}");

            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(definition) = serde_json::from_str::<DamageTypeDefinition>(&content)
            {
                registry.insert(damage_type_id, definition);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_damage_type_json() {
        let json = r#"{
            "message_id": "bleed",
            "scaling": "when_caused_by_living_non_player",
            "exhaustion": 0.25,
            "effects": "burning",
            "death_message_type": "intentional_game_design"
        }"#;

        let def: DamageTypeDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.message_id, "bleed");
        assert_eq!(
            def.scaling,
            DamageScalingWrapper::WhenCausedByLivingNonPlayer
        );
        assert!((def.exhaustion - 0.25).abs() < 1e-6);
        assert_eq!(def.effects, Some(DamageEffectsWrapper::Burning));
        assert_eq!(
            def.death_message_type,
            Some(DeathMessageTypeWrapper::IntentionalGameDesign)
        );

        let dt = to_damage_type(52, &def);
        assert_eq!(dt.id, 52);
        assert_eq!(dt.message_id, "bleed");
        assert_eq!(dt.scaling, DamageScaling::WhenCausedByLivingNonPlayer);
        assert_eq!(dt.effects, Some(DamageEffects::Burning));
        assert_eq!(
            dt.death_message_type,
            DeathMessageType::IntentionalGameDesign
        );
    }

    #[test]
    fn parse_minimal_damage_type_json() {
        let json = r#"{
            "message_id": "simple",
            "scaling": "never"
        }"#;

        let def: DamageTypeDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.message_id, "simple");
        assert_eq!(def.scaling, DamageScalingWrapper::Never);
        assert_eq!(def.exhaustion, 0.0);
        assert_eq!(def.effects, None);
        assert_eq!(def.death_message_type, None);

        let dt = to_damage_type(51, &def);
        assert_eq!(dt.id, 51);
        assert_eq!(dt.message_id, "simple");
        assert_eq!(dt.scaling, DamageScaling::Never);
        assert_eq!(dt.effects, None);
        assert_eq!(dt.death_message_type, DeathMessageType::Default);
    }

    #[test]
    fn to_registry_entry_encodes_valid_nbt() {
        let def = DamageTypeDefinition {
            message_id: "frost".to_string(),
            scaling: DamageScalingWrapper::Always,
            exhaustion: 0.1,
            effects: Some(DamageEffectsWrapper::Freezing),
            death_message_type: Some(DeathMessageTypeWrapper::FallVariants),
        };

        let entry = to_registry_entry("custom:frost".to_string(), &def);
        assert_eq!(entry.entry_id, "custom:frost");
        assert!(entry.data.is_some());

        let data = entry.data.unwrap();
        let mut cursor = std::io::Cursor::new(&data[..]);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
            pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
        );
        let nbt = Nbt::read_unnamed(&mut reader).unwrap().root_tag;
        assert_eq!(nbt.get_string("message_id"), Some("frost"));
        assert_eq!(nbt.get_string("scaling"), Some("always"));
        assert_eq!(nbt.get_string("effects"), Some("freezing"));
        assert_eq!(nbt.get_string("death_message_type"), Some("fall_variants"));
    }

    #[test]
    fn load_damage_types_from_directory_recursively() {
        let temp_dir =
            std::env::temp_dir().join(format!("pumpkin_test_dt_{}", uuid::Uuid::new_v4()));
        let sub_dir = temp_dir.join("sub");
        fs::create_dir_all(&sub_dir).unwrap();

        let dt1_json = r#"{
            "message_id": "bleeding",
            "scaling": "never",
            "exhaustion": 0.5
        }"#;
        fs::write(temp_dir.join("bleed.json"), dt1_json).unwrap();

        let dt2_json = r#"{
            "message_id": "radiation",
            "scaling": "always",
            "exhaustion": 1.0,
            "effects": "poking"
        }"#;
        fs::write(sub_dir.join("rad.json"), dt2_json).unwrap();

        let mut registry = HashMap::new();
        let count = load_damage_types_from_dir("custom", &temp_dir, &mut registry);
        assert_eq!(count, 2);
        assert!(registry.contains_key("custom:bleed"));
        assert!(registry.contains_key("custom:sub/rad"));

        let rad_def = &registry["custom:sub/rad"];
        assert_eq!(rad_def.message_id, "radiation");
        assert_eq!(rad_def.scaling, DamageScalingWrapper::Always);
        assert_eq!(rad_def.effects, Some(DamageEffectsWrapper::Poking));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn merge_damage_type_entries_preserves_vanilla_and_appends_custom() {
        use crate::data::datapack::merge_damage_type_entries;

        let vanilla = vec![
            RegistryEntryData {
                entry_id: "arrow".to_string(),
                data: Some(vec![1, 2, 3].into_boxed_slice()),
            },
            RegistryEntryData {
                entry_id: "cactus".to_string(),
                data: Some(vec![4, 5, 6].into_boxed_slice()),
            },
        ];

        let mut custom = HashMap::new();
        let bleed_def = DamageTypeDefinition {
            message_id: "bleed".to_string(),
            scaling: DamageScalingWrapper::Never,
            exhaustion: 0.1,
            effects: None,
            death_message_type: None,
        };
        let bleed_entry = DamageTypeEntry {
            id: "custom:bleed".to_string(),
            numeric_id: 51,
            definition: bleed_def.clone(),
            damage_type: to_damage_type(51, &bleed_def),
            nbt_data: vec![7, 8, 9].into_boxed_slice(),
        };
        custom.insert("custom:bleed".to_string(), bleed_entry);

        // Overriding vanilla "arrow"
        let arrow_def = DamageTypeDefinition {
            message_id: "custom_arrow".to_string(),
            scaling: DamageScalingWrapper::Always,
            exhaustion: 0.5,
            effects: None,
            death_message_type: None,
        };
        let arrow_override = DamageTypeEntry {
            id: "minecraft:arrow".to_string(),
            numeric_id: 0,
            definition: arrow_def.clone(),
            damage_type: to_damage_type(0, &arrow_def),
            nbt_data: vec![99, 99].into_boxed_slice(),
        };
        custom.insert("minecraft:arrow".to_string(), arrow_override);

        let merged = merge_damage_type_entries(&vanilla, &custom);
        assert_eq!(merged.len(), 3);
        // Arrow replaced in place at index 0
        assert_eq!(merged[0].entry_id, "arrow");
        assert_eq!(merged[0].data.as_deref(), Some(&[99, 99][..]));
        // Cactus preserved at index 1
        assert_eq!(merged[1].entry_id, "cactus");
        assert_eq!(merged[1].data.as_deref(), Some(&[4, 5, 6][..]));
        // Custom bleed appended at index 2
        assert_eq!(merged[2].entry_id, "custom:bleed");
        assert_eq!(merged[2].data.as_deref(), Some(&[7, 8, 9][..]));
    }
}
