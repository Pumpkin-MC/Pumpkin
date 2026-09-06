use pumpkin_data::packet::clientbound::play::RESOURCE_PACK_POP;

use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(RESOURCE_PACK_POP)]
pub struct CRemoveResourcePack<'a> {
    pub uuid: Option<&'a uuid::Uuid>,
}

impl<'a> CRemoveResourcePack<'a> {
    #[must_use]
    pub const fn new(uuid: Option<&'a uuid::Uuid>) -> Self {
        Self { uuid }
    }
}

impl ClientPacket for CRemoveResourcePack<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        if let Some(uuid) = self.uuid {
            write.write_bool(true)?;
            write.write_uuid(uuid)?;
        } else {
            write.write_bool(false)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_only_the_superseded_skin_pack() {
        let id = uuid::Uuid::from_u128(0x0011_2233_4455_6677_8899_aabb_ccdd_eeff);
        let mut encoded = Vec::new();
        CRemoveResourcePack::new(Some(&id))
            .write_packet_data(&mut encoded, &JavaMinecraftVersion::V_26_2)
            .unwrap();
        let mut expected = vec![1];
        expected.extend_from_slice(id.as_bytes());
        assert_eq!(encoded, expected);

        let mut all_packs = Vec::new();
        CRemoveResourcePack::new(None)
            .write_packet_data(&mut all_packs, &JavaMinecraftVersion::V_26_2)
            .unwrap();
        assert_eq!(all_packs, [0]);
    }
}
