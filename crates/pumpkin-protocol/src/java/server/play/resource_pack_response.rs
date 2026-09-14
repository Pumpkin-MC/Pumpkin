use pumpkin_data::packet::serverbound::play::RESOURCE_PACK;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[derive(Debug, PartialEq, Eq)]
pub enum PlayResourcePackResult {
    DownloadSuccess,
    Declined,
    DownloadFail,
    Accepted,
    Downloaded,
    InvalidUrl,
    ReloadFailed,
    Discarded,
    Unknown(i32),
}

#[java_packet(RESOURCE_PACK)]
pub struct SPlayResourcePack {
    pub uuid: uuid::Uuid,
    pub result: VarInt,
}

impl SPlayResourcePack {
    #[must_use]
    pub const fn response_result(&self) -> PlayResourcePackResult {
        match self.result.0 {
            0 => PlayResourcePackResult::DownloadSuccess,
            1 => PlayResourcePackResult::Declined,
            2 => PlayResourcePackResult::DownloadFail,
            3 => PlayResourcePackResult::Accepted,
            4 => PlayResourcePackResult::Downloaded,
            5 => PlayResourcePackResult::InvalidUrl,
            6 => PlayResourcePackResult::ReloadFailed,
            7 => PlayResourcePackResult::Discarded,
            v => PlayResourcePackResult::Unknown(v),
        }
    }
}

impl<'a> ServerPacket<'a> for SPlayResourcePack {
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

impl crate::ClientPacket for SPlayResourcePack {
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
