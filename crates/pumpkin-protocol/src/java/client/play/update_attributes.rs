use pumpkin_data::{
    attribute_id_remap::remap_attribute_id_for_version, packet::clientbound::PLAY_UPDATE_ATTRIBUTES,
};
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::codec::var_int::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[derive(Debug, PartialEq, Clone)]
#[java_packet(PLAY_UPDATE_ATTRIBUTES)]
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
}

impl ClientPacket for CUpdateAttributes {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        let properties: Vec<_> = self
            .properties
            .iter()
            .filter_map(|property| {
                remap_attribute_id_for_version(property.id.0 as u16, *version)
                    .map(|id| (property, id))
            })
            .collect();

        write.write_var_int(&VarInt(properties.len() as i32))?;
        for (prop, attribute_id) in properties {
            write.write_var_int(&VarInt(i32::from(attribute_id)))?;
            write.write_f64(prop.value)?;
            write.write_var_int(&VarInt(prop.modifiers.len() as i32))?;
            for modifier in &prop.modifiers {
                write.write_string(&modifier.id)?;
                write.write_f64(modifier.amount)?;
                write.write_u8(modifier.operation as u8)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::version::JavaMinecraftVersion;

    use crate::{
        ClientPacket,
        codec::var_int::VarInt,
        java::client::play::{CUpdateAttributes, Property},
    };

    fn packet_with_attribute(attribute_id: i32) -> CUpdateAttributes {
        CUpdateAttributes::new(
            VarInt(1),
            vec![Property::new(VarInt(attribute_id), 4.0, Vec::new())],
        )
    }

    #[test]
    fn remaps_attack_speed_for_1_21_11() {
        let mut bytes = Vec::new();
        packet_with_attribute(5)
            .write_packet_data(&mut bytes, &JavaMinecraftVersion::V_1_21_11)
            .unwrap();

        assert_eq!(&bytes[..3], &[1, 1, 4]);
    }

    #[test]
    fn keeps_attack_speed_id_for_26_2() {
        let mut bytes = Vec::new();
        packet_with_attribute(5)
            .write_packet_data(&mut bytes, &JavaMinecraftVersion::V_26_2)
            .unwrap();

        assert_eq!(&bytes[..3], &[1, 1, 5]);
    }

    #[test]
    fn skips_attributes_missing_from_target_version() {
        let mut bytes = Vec::new();
        packet_with_attribute(0)
            .write_packet_data(&mut bytes, &JavaMinecraftVersion::V_1_21_11)
            .unwrap();

        assert_eq!(bytes, [1, 0]);
    }
}
