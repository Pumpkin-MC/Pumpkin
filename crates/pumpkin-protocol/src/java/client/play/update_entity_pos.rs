use pumpkin_data::packet::clientbound::play::MOVE_ENTITY_POS;
use pumpkin_macros::java_packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};

/// 26.3 packs `on_ground` in bit 0 of a `VarInt` ahead of the delta. Remaining bits are the
/// `VecDelta` step count; Pumpkin only writes 0 (one linear i16 triple).
pub(super) fn read_on_ground_and_linear_delta(
    bytebuf: &mut impl NetworkReadExt,
) -> Result<(bool, Vector3<i16>), ReadingError> {
    let properties = bytebuf.get_var_int()?.0;
    if properties >> 1 != 0 {
        return Err(ReadingError::Message(
            "entity move delta is not a single linear step".into(),
        ));
    }
    let on_ground = properties & 1 != 0;
    Ok((
        on_ground,
        Vector3::new(
            bytebuf.get_i16_be()?,
            bytebuf.get_i16_be()?,
            bytebuf.get_i16_be()?,
        ),
    ))
}

#[java_packet(MOVE_ENTITY_POS)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CUpdateEntityPos {
    pub entity_id: VarInt,
    pub delta: Vector3<i16>,
    pub on_ground: bool,
}

impl CUpdateEntityPos {
    #[must_use]
    pub const fn new(entity_id: VarInt, delta: Vector3<i16>, on_ground: bool) -> Self {
        Self {
            entity_id,
            delta,
            on_ground,
        }
    }
}

impl ClientPacket for CUpdateEntityPos {
    fn write_packet_data(&self, mut write: impl std::io::Write) -> Result<(), WritingError> {
        write.write_var_int(&self.entity_id)?;
        // Since 26.3 the on ground flag and the delta step count are packed into a var int in
        // front of the delta, 0 steps being a single linear delta.
        write.write_var_int(&VarInt(i32::from(self.on_ground)))?;
        write.write_i16_be(self.delta.x)?;
        write.write_i16_be(self.delta.y)?;
        write.write_i16_be(self.delta.z)?;

        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CUpdateEntityPos {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let entity_id = bytebuf.get_var_int()?;
        let (on_ground, delta) = read_on_ground_and_linear_delta(bytebuf)?;
        Ok(Self {
            entity_id,
            delta,
            on_ground,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let packet = CUpdateEntityPos::new(VarInt(9), Vector3::new(4096, -128, 256), true);
        let mut buf = Vec::new();
        packet.write_packet_data(&mut buf).unwrap();
        let read = CUpdateEntityPos::read(&mut buf.as_slice()).unwrap();
        assert_eq!(read, packet);
    }

    #[test]
    fn stepped_delta_is_rejected() {
        let mut buf = Vec::new();
        buf.write_var_int(&VarInt(9)).unwrap();
        // on ground false, one step
        buf.write_var_int(&VarInt(2)).unwrap();
        buf.write_i16_be(1).unwrap();
        buf.write_i16_be(2).unwrap();
        buf.write_i16_be(3).unwrap();
        assert!(CUpdateEntityPos::read(&mut buf.as_slice()).is_err());
    }
}
