//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/update_attributes.rs`.

use crate::codec::var_int::VarInt;
use crate::java::client::play::CUpdateAttributes;
use crate::java::legacy::LegacyWrite;
use crate::ser::{NetworkWriteExt, WritingError};
use pumpkin_data::attributes::Attributes;
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Write;

impl LegacyWrite for CUpdateAttributes {
    fn write_legacy(
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
                write.write_var_int(&prop.id)?;
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
