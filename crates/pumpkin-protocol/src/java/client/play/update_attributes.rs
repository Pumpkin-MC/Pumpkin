use std::io::Write;

use pumpkin_data::attribute_id_remap::remap_attribute_id_for_version;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::packet::clientbound::play::UPDATE_ATTRIBUTES;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::codec::var_int::VarInt;
use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};
use crate::{ClientPacket, ServerPacket};

#[derive(Debug, PartialEq, Clone)]
#[java_packet(UPDATE_ATTRIBUTES)]
pub struct CUpdateAttributes {
    pub entity_id: VarInt,
    pub properties: Vec<Property>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Property {
    pub id: VarInt,
    pub value: f64,
    pub modifiers: Vec<AttributeModifier>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AttributeModifier {
    pub id: String,
    pub amount: f64,
    pub operation: i8,
}

impl CUpdateAttributes {
    #[must_use]
    pub const fn new(entity_id: VarInt, properties: Vec<Property>) -> Self {
        Self {
            entity_id,
            properties,
        }
    }
}

impl Property {
    #[must_use]
    pub const fn new(id: VarInt, value: f64, modifiers: Vec<AttributeModifier>) -> Self {
        Self {
            id,
            value,
            modifiers,
        }
    }
}

impl AttributeModifier {
    #[must_use]
    pub const fn new(id: String, amount: f64, operation: i8) -> Self {
        Self {
            id,
            amount,
            operation,
        }
    }

    #[must_use]
    pub fn from_uuid(uuid: uuid::Uuid, amount: f64, operation: i8) -> Self {
        Self {
            id: uuid.to_string(),
            amount,
            operation,
        }
    }

    #[must_use]
    pub fn uuid(&self) -> uuid::Uuid {
        uuid::Uuid::parse_str(&self.id)
            .unwrap_or_else(|_| uuid::Uuid::new_v3(&uuid::Uuid::NAMESPACE_OID, self.id.as_bytes()))
    }
}

#[must_use]
pub fn attribute_id_to_legacy_name(id: u8) -> &'static str {
    match id {
        23 => "generic.maxHealth",
        32 => "zombie.spawnReinforcements",
        19 => "horse.jumpStrength",
        16 => "generic.followRange",
        20 => "generic.knockbackResistance",
        26 => "generic.movementSpeed",
        15 => "generic.flyingSpeed",
        3 => "generic.attackDamage",
        4 => "generic.attackKnockback",
        5 => "generic.attackSpeed",
        2 => "generic.armorToughness",
        1 => "generic.armor",
        21 => "generic.luck",
        _ => Attributes::ALL
            .get(id as usize)
            .map_or("generic.maxHealth", |attr| attr.name),
    }
}

#[must_use]
pub fn attribute_id_to_1_16_name(id: u8) -> &'static str {
    match id {
        1 => "minecraft:generic.armor",
        2 => "minecraft:generic.armor_toughness",
        3 => "minecraft:generic.attack_damage",
        4 => "minecraft:generic.attack_knockback",
        5 => "minecraft:generic.attack_speed",
        7 => "minecraft:player.block_break_speed",
        8 => "minecraft:player.block_interaction_range",
        10 => "minecraft:generic.burning_time",
        12 => "minecraft:generic.explosion_knockback_resistance",
        13 => "minecraft:player.entity_interaction_range",
        14 => "minecraft:generic.fall_damage_multiplier",
        15 => "minecraft:generic.flying_speed",
        16 => "minecraft:generic.follow_range",
        18 => "minecraft:generic.gravity",
        19 => "minecraft:horse.jump_strength",
        20 => "minecraft:generic.knockback_resistance",
        21 => "minecraft:generic.luck",
        22 => "minecraft:generic.max_absorption",
        23 => "minecraft:generic.max_health",
        24 => "minecraft:player.mining_efficiency",
        25 => "minecraft:generic.movement_efficiency",
        26 => "minecraft:generic.movement_speed",
        28 => "minecraft:generic.oxygen_bonus",
        29 => "minecraft:generic.safe_fall_distance",
        30 => "minecraft:generic.scale",
        31 => "minecraft:player.sneaking_speed",
        32 => "minecraft:zombie.spawn_reinforcements",
        33 => "minecraft:generic.step_height",
        34 => "minecraft:player.submerged_mining_speed",
        35 => "minecraft:player.sweeping_damage_ratio",
        37 => "minecraft:generic.water_movement_efficiency",
        _ => Attributes::ALL
            .get(id as usize)
            .map_or("minecraft:generic.max_health", |attr| attr.name),
    }
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn attribute_name_to_id(name: &str) -> Option<u8> {
    match name {
        "generic.maxHealth"
        | "Max Health"
        | "minecraft:generic.max_health"
        | "generic.max_health"
        | "minecraft:max_health"
        | "max_health" => Some(Attributes::MAX_HEALTH.id),
        "zombie.spawnReinforcements"
        | "Spawn Reinforcements Chance"
        | "minecraft:zombie.spawn_reinforcements"
        | "zombie.spawn_reinforcements"
        | "minecraft:spawn_reinforcements"
        | "spawn_reinforcements" => Some(Attributes::SPAWN_REINFORCEMENTS.id),
        "horse.jumpStrength"
        | "Jump Strength"
        | "minecraft:horse.jump_strength"
        | "horse.jump_strength"
        | "minecraft:jump_strength"
        | "jump_strength" => Some(Attributes::JUMP_STRENGTH.id),
        "generic.followRange"
        | "Follow Range"
        | "minecraft:generic.follow_range"
        | "generic.follow_range"
        | "minecraft:follow_range"
        | "follow_range" => Some(Attributes::FOLLOW_RANGE.id),
        "generic.knockbackResistance"
        | "Knockback Resistance"
        | "minecraft:generic.knockback_resistance"
        | "generic.knockback_resistance"
        | "minecraft:knockback_resistance"
        | "knockback_resistance" => Some(Attributes::KNOCKBACK_RESISTANCE.id),
        "generic.movementSpeed"
        | "Movement Speed"
        | "minecraft:generic.movement_speed"
        | "generic.movement_speed"
        | "minecraft:movement_speed"
        | "movement_speed" => Some(Attributes::MOVEMENT_SPEED.id),
        "generic.flyingSpeed"
        | "Flying Speed"
        | "minecraft:generic.flying_speed"
        | "generic.flying_speed"
        | "minecraft:flying_speed"
        | "flying_speed" => Some(Attributes::FLYING_SPEED.id),
        "generic.attackDamage"
        | "Attack Damage"
        | "minecraft:generic.attack_damage"
        | "generic.attack_damage"
        | "minecraft:attack_damage"
        | "attack_damage" => Some(Attributes::ATTACK_DAMAGE.id),
        "generic.attackKnockback"
        | "minecraft:generic.attack_knockback"
        | "generic.attack_knockback"
        | "minecraft:attack_knockback"
        | "attack_knockback" => Some(Attributes::ATTACK_KNOCKBACK.id),
        "generic.attackSpeed"
        | "minecraft:generic.attack_speed"
        | "generic.attack_speed"
        | "minecraft:attack_speed"
        | "attack_speed" => Some(Attributes::ATTACK_SPEED.id),
        "generic.armorToughness"
        | "Armor Toughness"
        | "minecraft:generic.armor_toughness"
        | "generic.armor_toughness"
        | "minecraft:armor_toughness"
        | "armor_toughness" => Some(Attributes::ARMOR_TOUGHNESS.id),
        "generic.armor" | "Armor" | "minecraft:generic.armor" | "minecraft:armor" | "armor" => {
            Some(Attributes::ARMOR.id)
        }
        "generic.luck" | "Luck" | "minecraft:generic.luck" | "minecraft:luck" | "luck" => {
            Some(Attributes::LUCK.id)
        }
        "generic.maxAbsorption"
        | "minecraft:generic.max_absorption"
        | "generic.max_absorption"
        | "minecraft:max_absorption"
        | "max_absorption" => Some(Attributes::MAX_ABSORPTION.id),
        "minecraft:generic.scale" | "generic.scale" | "minecraft:scale" | "scale" => {
            Some(Attributes::SCALE.id)
        }
        "minecraft:generic.step_height"
        | "generic.step_height"
        | "minecraft:step_height"
        | "step_height" => Some(Attributes::STEP_HEIGHT.id),
        "minecraft:generic.gravity" | "generic.gravity" | "minecraft:gravity" | "gravity" => {
            Some(Attributes::GRAVITY.id)
        }
        "minecraft:generic.safe_fall_distance"
        | "generic.safe_fall_distance"
        | "minecraft:safe_fall_distance"
        | "safe_fall_distance" => Some(Attributes::SAFE_FALL_DISTANCE.id),
        "minecraft:generic.fall_damage_multiplier"
        | "generic.fall_damage_multiplier"
        | "minecraft:fall_damage_multiplier"
        | "fall_damage_multiplier" => Some(Attributes::FALL_DAMAGE_MULTIPLIER.id),
        "minecraft:generic.burning_time"
        | "generic.burning_time"
        | "minecraft:burning_time"
        | "burning_time" => Some(Attributes::BURNING_TIME.id),
        "minecraft:generic.explosion_knockback_resistance"
        | "generic.explosion_knockback_resistance"
        | "minecraft:explosion_knockback_resistance"
        | "explosion_knockback_resistance" => Some(Attributes::EXPLOSION_KNOCKBACK_RESISTANCE.id),
        "minecraft:generic.movement_efficiency"
        | "generic.movement_efficiency"
        | "minecraft:movement_efficiency"
        | "movement_efficiency" => Some(Attributes::MOVEMENT_EFFICIENCY.id),
        "minecraft:generic.oxygen_bonus"
        | "generic.oxygen_bonus"
        | "minecraft:oxygen_bonus"
        | "oxygen_bonus" => Some(Attributes::OXYGEN_BONUS.id),
        "minecraft:generic.water_movement_efficiency"
        | "generic.water_movement_efficiency"
        | "minecraft:water_movement_efficiency"
        | "water_movement_efficiency" => Some(Attributes::WATER_MOVEMENT_EFFICIENCY.id),
        "minecraft:player.block_interaction_range"
        | "player.block_interaction_range"
        | "minecraft:block_interaction_range"
        | "block_interaction_range" => Some(Attributes::BLOCK_INTERACTION_RANGE.id),
        "minecraft:player.entity_interaction_range"
        | "player.entity_interaction_range"
        | "minecraft:entity_interaction_range"
        | "entity_interaction_range" => Some(Attributes::ENTITY_INTERACTION_RANGE.id),
        "minecraft:player.block_break_speed"
        | "player.block_break_speed"
        | "minecraft:block_break_speed"
        | "block_break_speed" => Some(Attributes::BLOCK_BREAK_SPEED.id),
        "minecraft:player.submerged_mining_speed"
        | "player.submerged_mining_speed"
        | "minecraft:submerged_mining_speed"
        | "submerged_mining_speed" => Some(Attributes::SUBMERGED_MINING_SPEED.id),
        "minecraft:player.sneaking_speed"
        | "player.sneaking_speed"
        | "minecraft:sneaking_speed"
        | "sneaking_speed" => Some(Attributes::SNEAKING_SPEED.id),
        "minecraft:player.mining_efficiency"
        | "player.mining_efficiency"
        | "minecraft:mining_efficiency"
        | "mining_efficiency" => Some(Attributes::MINING_EFFICIENCY.id),
        "minecraft:player.sweeping_damage_ratio"
        | "player.sweeping_damage_ratio"
        | "minecraft:sweeping_damage_ratio"
        | "sweeping_damage_ratio" => Some(Attributes::SWEEPING_DAMAGE_RATIO.id),
        _ => {
            let trimmed = name.strip_prefix("minecraft:").unwrap_or(name);
            Attributes::ALL
                .iter()
                .find(|attr| {
                    let attr_trimmed = attr.name.strip_prefix("minecraft:").unwrap_or(attr.name);
                    attr.name == name || attr_trimmed == trimmed
                })
                .map(|attr| attr.id)
        }
    }
}

impl ClientPacket for CUpdateAttributes {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i32_be(self.entity_id.0)?;
        } else {
            write.write_var_int(&self.entity_id)?;
        }

        if *version >= JavaMinecraftVersion::V_1_17 {
            write.write_var_int(&VarInt(self.properties.len() as i32))?;
        } else {
            write.write_i32_be(self.properties.len() as i32)?;
        }

        for prop in &self.properties {
            if *version >= JavaMinecraftVersion::V_1_20_5 {
                let remapped_id = remap_attribute_id_for_version(prop.id.0 as u32, *version);
                write.write_var_int(&VarInt(remapped_id as i32))?;
            } else if *version >= JavaMinecraftVersion::V_1_16 {
                let name = attribute_id_to_1_16_name(prop.id.0 as u8);
                write.write_string(name)?;
            } else {
                let name = attribute_id_to_legacy_name(prop.id.0 as u8);
                write.write_string(name)?;
            }

            write.write_f64_be(prop.value)?;

            if *version <= JavaMinecraftVersion::V_1_7_6 {
                write.write_i16_be(prop.modifiers.len() as i16)?;
            } else {
                write.write_var_int(&VarInt(prop.modifiers.len() as i32))?;
            }

            for modifier in &prop.modifiers {
                if *version >= JavaMinecraftVersion::V_1_21 {
                    write.write_string(&modifier.id)?;
                } else {
                    let uuid = modifier.uuid();
                    write.write_uuid(&uuid)?;
                }
                write.write_f64_be(modifier.amount)?;
                write.write_u8(modifier.operation as u8)?;
            }
        }
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CUpdateAttributes {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let entity_id = if *version <= JavaMinecraftVersion::V_1_7_6 {
            VarInt(bytebuf.get_i32_be()?)
        } else {
            bytebuf.get_var_int()?
        };

        let property_count = if *version >= JavaMinecraftVersion::V_1_17 {
            bytebuf.get_var_int()?.0 as usize
        } else {
            bytebuf.get_i32_be()? as usize
        };

        let mut properties = Vec::with_capacity(property_count);
        for _ in 0..property_count {
            let id = if *version >= JavaMinecraftVersion::V_1_20_5 {
                bytebuf.get_var_int()?
            } else {
                let name = bytebuf.get_str()?;
                VarInt(i32::from(attribute_name_to_id(&name).unwrap_or(0)))
            };

            let value = bytebuf.get_f64_be()?;

            let modifiers_length = if *version <= JavaMinecraftVersion::V_1_7_6 {
                bytebuf.get_i16_be()? as usize
            } else {
                bytebuf.get_var_int()?.0 as usize
            };

            let mut modifiers = Vec::with_capacity(modifiers_length);
            for _ in 0..modifiers_length {
                let id = if *version >= JavaMinecraftVersion::V_1_21 {
                    bytebuf.get_str()?.to_string()
                } else {
                    let uuid = bytebuf.get_uuid()?;
                    uuid.to_string()
                };
                let amount = bytebuf.get_f64_be()?;
                let operation = bytebuf.get_u8()? as i8;
                modifiers.push(AttributeModifier::new(id, amount, operation));
            }

            properties.push(Property::new(id, value, modifiers));
        }

        Ok(Self {
            entity_id,
            properties,
        })
    }
}
