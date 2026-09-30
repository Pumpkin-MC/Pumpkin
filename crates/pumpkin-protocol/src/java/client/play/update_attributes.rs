use std::io::Write;

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
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_var_int(&self.entity_id)?;

        write.write_var_int(&VarInt(self.properties.len() as i32))?;

        for prop in &self.properties {
            write.write_var_int(&prop.id)?;

            write.write_f64_be(prop.value)?;

            write.write_var_int(&VarInt(prop.modifiers.len() as i32))?;

            for modifier in &prop.modifiers {
                write.write_string(&modifier.id)?;
                write.write_f64_be(modifier.amount)?;
                write.write_u8(modifier.operation as u8)?;
            }
        }
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CUpdateAttributes {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let entity_id = bytebuf.get_var_int()?;

        let property_count = bytebuf.get_var_int()?.0 as usize;

        let mut properties = Vec::with_capacity(property_count);
        for _ in 0..property_count {
            let id = bytebuf.get_var_int()?;

            let value = bytebuf.get_f64_be()?;

            let modifiers_length = bytebuf.get_var_int()?.0 as usize;

            let mut modifiers = Vec::with_capacity(modifiers_length);
            for _ in 0..modifiers_length {
                let id = bytebuf.get_str()?.to_string();
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

#[cfg(test)]
mod tests {
    use pumpkin_data::attributes::Attributes;
    use pumpkin_data::packet::clientbound::play::UPDATE_ATTRIBUTES;
    use pumpkin_util::version::JavaMinecraftVersion;

    use crate::{ClientPacket, VarInt, packet::MultiVersionJavaPacket, ser::NetworkReadExt};

    use super::{AttributeModifier, CUpdateAttributes, Property};

    fn encoded_armor_attributes(version: JavaMinecraftVersion) -> Vec<u8> {
        let packet = CUpdateAttributes::new(
            VarInt(1),
            vec![Property::new(
                VarInt(i32::from(Attributes::ARMOR.id)),
                0.0,
                vec![AttributeModifier::new(
                    "minecraft:armor.chestplate".to_string(),
                    8.0,
                    0,
                )],
            )],
        );
        let mut buf = Vec::new();
        packet.write_packet_data(&mut buf, &version).unwrap();
        buf
    }

    fn assert_armor_attribute_payload(bytes: &[u8], _version: JavaMinecraftVersion) {
        let mut cursor = bytes;
        let entity_id = cursor.get_var_int().unwrap();
        assert_eq!(entity_id, VarInt(1));

        let count = cursor.get_var_int().unwrap();
        assert_eq!(count, VarInt(1));

        let attr_id = cursor.get_var_int().unwrap();
        let expected_id = u32::from(Attributes::ARMOR.id);
        assert_eq!(attr_id.0, expected_id as i32);

        let base = cursor.get_f64_be().unwrap();
        assert_eq!(base, 0.0);

        let modifier_count = cursor.get_var_int().unwrap();
        assert_eq!(modifier_count, VarInt(1));

        let id = cursor.get_str().unwrap();
        assert_eq!(&*id, "minecraft:armor.chestplate");
        let amount = cursor.get_f64_be().unwrap();
        assert_eq!(amount, 8.0);
        let operation = cursor.get_u8().unwrap();
        assert_eq!(operation, 0);
        assert!(cursor.is_empty());
    }

    #[test]
    fn update_attributes_packet_id_for_26_3() {
        assert_eq!(
            CUpdateAttributes::to_id(pumpkin_data::packet::CURRENT_MC_VERSION),
            UPDATE_ATTRIBUTES.to_id(pumpkin_data::packet::CURRENT_MC_VERSION)
        );
        assert_eq!(
            CUpdateAttributes::to_id(pumpkin_data::packet::CURRENT_MC_VERSION),
            134
        );
    }

    #[test]
    fn armor_attribute_encodes() {
        let version = pumpkin_data::packet::CURRENT_MC_VERSION;
        assert_armor_attribute_payload(&encoded_armor_attributes(version), version);
    }
}
