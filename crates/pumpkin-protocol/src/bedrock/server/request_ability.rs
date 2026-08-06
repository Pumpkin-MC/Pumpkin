use crate::{codec::var_int::VarInt, serial::PacketRead};
use pumpkin_macros::packet;
use std::io::{Error, Read};

#[derive(Clone, Debug)]
pub enum AbilityValue {
    Bool(bool),
    Float(f32),
}

impl PacketRead for AbilityValue {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let val_type = u8::read(buf)?;
        let bool_val = bool::read(buf)?;
        let float_val = f32::read(buf)?;
        match val_type {
            1 => Ok(Self::Bool(bool_val)),
            2 => Ok(Self::Float(float_val)),
            _ => Err(Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid ability value type: {val_type}"),
            )),
        }
    }
}

#[packet(102)]
pub struct SRequestAbility {
    pub ability: VarInt,
    pub value: AbilityValue,
}

impl PacketRead for SRequestAbility {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let ability = VarInt::read(buf)?;
        let value = AbilityValue::read(buf)?;
        Ok(Self { ability, value })
    }
}
