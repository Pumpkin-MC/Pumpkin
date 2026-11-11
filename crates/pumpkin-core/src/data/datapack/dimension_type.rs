//! Datapack `dimension_type` attributes, read once when a level is created.

use pumpkin_data::dimension::{BedRule, BedRuleOption, Dimension};
use pumpkin_nbt::{Nbt, compound::NbtCompound, tag::NbtTag};
use tracing::error;

use super::DatapackManager;

type BoolField = fn(&mut Dimension) -> &mut bool;

/// Boolean `EnvironmentAttributes` of a dimension type: key, vanilla default, field.
const BOOL_ATTRIBUTES: [(&str, bool, BoolField); 7] = [
    ("gameplay/fast_lava", false, |d| &mut d.fast_lava),
    ("gameplay/water_evaporates", false, |d| {
        &mut d.water_evaporates
    }),
    ("gameplay/respawn_anchor_works", false, |d| {
        &mut d.respawn_anchor_works
    }),
    ("gameplay/piglins_zombify", true, |d| &mut d.piglins_zombify),
    ("gameplay/snow_golem_melts", false, |d| {
        &mut d.snow_golem_melts
    }),
    ("gameplay/can_start_raid", true, |d| &mut d.can_start_raid),
    ("gameplay/nether_portal_spawns_piglin", false, |d| {
        &mut d.nether_portal_spawns_piglin
    }),
];

// TODO: gameplay/straw_bed_rule (destroy_on_leave), straw beds use bed_rule
const BED_RULE: &str = "gameplay/bed_rule";

impl DatapackManager {
    /// `dimension` with the attributes of its datapack `dimension_type` entry.
    ///
    /// A datapack entry replaces the whole dimension type, so attributes it leaves out
    /// take their default. Invalid entries keep `dimension` and are logged.
    #[must_use]
    pub fn with_dimension_type_attributes(&self, dimension: Dimension) -> Dimension {
        let Some(bytes) = self
            .get_custom_registry_entry("minecraft:dimension_type", dimension.minecraft_name)
            .and_then(|entry| entry.data)
        else {
            return dimension;
        };
        let mut cursor = std::io::Cursor::new(&bytes[..]);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
            pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
        );
        let Ok(root) = Nbt::read_unnamed(&mut reader).map(|nbt| nbt.root_tag) else {
            error!("Unreadable dimension_type {}", dimension.minecraft_name);
            return dimension;
        };
        let empty = NbtCompound::new();
        let attributes = root.get_compound("attributes").unwrap_or(&empty);
        match with_attributes(dimension.clone(), attributes) {
            Ok(resolved) => resolved,
            Err(key) => {
                error!(
                    "Invalid attribute {key} in dimension_type {}",
                    dimension.minecraft_name
                );
                dimension
            }
        }
    }
}

fn with_attributes(
    mut dimension: Dimension,
    attributes: &NbtCompound,
) -> Result<Dimension, &'static str> {
    for (key, default, field) in BOOL_ATTRIBUTES {
        *field(&mut dimension) = bool_attribute(attributes, key, default)?;
    }
    dimension.bed_rule = bed_rule(attributes)?;
    Ok(dimension)
}

/// Attribute `key`, with or without the `minecraft:` namespace.
fn attribute<'a>(attributes: &'a NbtCompound, key: &str) -> Option<&'a NbtTag> {
    attributes
        .get(&format!("minecraft:{key}"))
        .or_else(|| attributes.get(key))
}

/// `EnvironmentAttributeMap.Entry` of a boolean attribute applied to `default`.
fn bool_attribute(
    attributes: &NbtCompound,
    key: &'static str,
    default: bool,
) -> Result<bool, &'static str> {
    let Some(tag) = attribute(attributes, key) else {
        return Ok(default);
    };
    match tag {
        // bare value: override
        NbtTag::Byte(value) => Ok(*value != 0),
        NbtTag::Compound(entry) => {
            let argument = entry.get_bool("argument").ok_or(key)?;
            match entry.get_string("modifier").ok_or(key)? {
                "override" => Ok(argument),
                "and" => Ok(argument && default),
                "nand" => Ok(!argument || !default),
                "or" => Ok(argument || default),
                "nor" => Ok(!argument && !default),
                "xor" => Ok(argument ^ default),
                "xnor" => Ok(argument == default),
                _ => Err(key),
            }
        }
        _ => Err(key),
    }
}

/// `BedRule.CODEC`, override only; default `CAN_SLEEP_WHEN_DARK`.
fn bed_rule(attributes: &NbtCompound) -> Result<BedRule, &'static str> {
    let Some(tag) = attribute(attributes, BED_RULE) else {
        return Ok(BedRule {
            can_sleep: BedRuleOption::WhenDark,
            can_set_spawn: BedRuleOption::Always,
            destroy_on_use: false,
        });
    };
    let NbtTag::Compound(rule) = tag else {
        return Err(BED_RULE);
    };
    let mut rule: &NbtCompound = rule;
    if let Some(modifier) = rule.get_string("modifier") {
        rule = match (modifier, rule.get_compound("argument")) {
            ("override", Some(argument)) => argument,
            _ => return Err(BED_RULE),
        };
    }
    let option = |name| match rule.get_string(name) {
        Some("always") => Ok(BedRuleOption::Always),
        Some("when_dark") => Ok(BedRuleOption::WhenDark),
        Some("never") => Ok(BedRuleOption::Never),
        _ => Err(BED_RULE),
    };
    Ok(BedRule {
        can_sleep: option("can_sleep")?,
        can_set_spawn: option("can_set_spawn")?,
        destroy_on_use: rule.get_bool("destroy_on_use").unwrap_or(false),
    })
}
