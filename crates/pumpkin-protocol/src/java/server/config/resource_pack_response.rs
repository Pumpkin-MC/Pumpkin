use pumpkin_data::packet::serverbound::config::RESOURCE_PACK;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourcePackResponseResult {
    DownloadSuccess,
    DownloadFail,
    Downloaded,
    Accepted,
    Declined,
    InvalidUrl,
    ReloadFailed,
    Discarded,
    Unknown(i32),
}

/// Sent by the client to inform the server of the status of a requested resource pack.
///
/// This allows the server to know if the player is using the required textures
/// or if the download failed.
#[java_packet(RESOURCE_PACK)]
pub struct SConfigResourcePack {
    /// The unique identifier of the resource pack this response refers to.
    pub uuid: uuid::Uuid,
    /// The status code of the operation, mapped to [`ResourcePackResponseResult`].
    pub result: VarInt,
}

impl<'a> ServerPacket<'a> for SConfigResourcePack {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let uuid = if *version >= JavaMinecraftVersion::V_1_20_3 {
            bytebuf.get_uuid()?
        } else {
            uuid::Uuid::nil()
        };
        if *version < JavaMinecraftVersion::V_1_10 {
            let _hash = bytebuf.get_str_bounded_borrowed(40)?;
        }
        let result = bytebuf.get_var_int()?;
        Ok(Self { uuid, result })
    }
}

impl crate::ClientPacket for SConfigResourcePack {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_uuid(&self.uuid)?;
        write.write_var_int(&self.result)?;
        Ok(())
    }
}

impl SConfigResourcePack {
    #[must_use]
    pub const fn response_result(&self) -> ResourcePackResponseResult {
        ResourcePackResponseResult::from_id(self.result.0)
    }
}

impl ResourcePackResponseResult {
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            0 => Self::DownloadSuccess,
            1 => Self::Declined,
            2 => Self::DownloadFail,
            3 => Self::Accepted,
            4 => Self::Downloaded,
            5 => Self::InvalidUrl,
            6 => Self::ReloadFailed,
            7 => Self::Discarded,
            x => Self::Unknown(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_responses_omit_pack_uuid_before_1_20_3() {
        let mut payload = [0].as_slice();
        let response =
            SConfigResourcePack::read(&mut payload, &JavaMinecraftVersion::V_1_20_2).unwrap();
        assert_eq!(response.uuid, uuid::Uuid::nil());
        assert_eq!(
            response.response_result(),
            ResourcePackResponseResult::DownloadSuccess
        );
        assert!(payload.is_empty());
    }

    #[test]
    fn configuration_responses_identify_each_modern_pack() {
        let id = uuid::Uuid::from_u128(1);
        let mut bytes = id.as_bytes().to_vec();
        bytes.push(1);
        let mut payload = bytes.as_slice();
        let response =
            SConfigResourcePack::read(&mut payload, &JavaMinecraftVersion::V_26_2).unwrap();
        assert_eq!(response.uuid, id);
        assert_eq!(
            response.response_result(),
            ResourcePackResponseResult::Declined
        );
        assert!(payload.is_empty());
    }
}
