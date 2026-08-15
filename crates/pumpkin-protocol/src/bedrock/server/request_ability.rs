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

#[derive(PacketRead)]
#[packet(184)]
pub struct SRequestAbility {
    pub ability: VarInt,
    pub value: AbilityValue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_ability_uses_v2168_packet_id_and_payload() {
        assert_eq!(<SRequestAbility as crate::Packet>::PACKET_ID, 184);

        let packet = SRequestAbility::read(&mut [2, 1, 1, 0, 0, 0, 0].as_slice()).unwrap();
        assert_eq!(packet.ability, VarInt(1));
        assert!(matches!(packet.value, AbilityValue::Bool(true)));
    }
}
