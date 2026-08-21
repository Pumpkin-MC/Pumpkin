use pumpkin_data::packet::clientbound::play::UPDATE_ATTRIBUTES;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::codec::var_int::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

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
}

use pumpkin_data::attribute_id_remap::remap_attribute_id_for_version;

impl ClientPacket for CUpdateAttributes {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        write.write_var_int(&VarInt(self.properties.len() as i32))?;
        for prop in &self.properties {
            let remapped_id = remap_attribute_id_for_version(prop.id.0 as u32, *version);
            write.write_var_int(&VarInt(remapped_id as i32))?;
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
