use pumpkin_data::packet::clientbound::PLAY_RESOURCE_PACK_POP;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ClientPacket,
    ser::{NetworkWriteExt, WritingError},
};

#[java_packet(PLAY_RESOURCE_PACK_POP)]
pub struct CPlayResourcePackPop<'a>(pub Option<&'a uuid::Uuid>);

impl ClientPacket for CPlayResourcePackPop<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_bool(self.0.is_some())?;
        if let Some(uuid) = self.0 {
            write.write_uuid(uuid)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::CPlayResourcePackPop;
    use crate::ClientPacket;
    use pumpkin_util::version::JavaMinecraftVersion;

    #[test]
    fn writes_optional_pack_id() {
        let id = uuid::Uuid::from_u128(0x0011_2233_4455_6677_8899_aabb_ccdd_eeff);
        let mut with_id = Vec::new();
        CPlayResourcePackPop(Some(&id))
            .write_packet_data(&mut with_id, &JavaMinecraftVersion::V_26_2)
            .unwrap();
        let mut expected = vec![1];
        expected.extend_from_slice(id.as_bytes());
        assert_eq!(with_id, expected);

        let mut without_id = Vec::new();
        CPlayResourcePackPop(None)
            .write_packet_data(&mut without_id, &JavaMinecraftVersion::V_26_2)
            .unwrap();
        assert_eq!(without_id, [0]);
    }
}
