use pumpkin_data::packet::serverbound::LOGIN_HELLO;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(LOGIN_HELLO)]
pub struct SLoginStart {
    pub name: Box<str>, // 16
    pub uuid: uuid::Uuid,
}

impl<'a> ServerPacket<'a> for SLoginStart {
    fn read(read: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let name = read.get_str_bounded(16)?;
        let uuid = if version >= &JavaMinecraftVersion::V_1_20_2 {
            read.get_uuid()?
        } else if version >= &JavaMinecraftVersion::V_1_19_3 {
            let has_uuid = read.get_bool()?;
            if has_uuid {
                read.get_uuid()?
            } else {
                uuid::Uuid::new_v3(
                    &uuid::Uuid::nil(),
                    format!("OfflinePlayer:{name}").as_bytes(),
                )
            }
        } else {
            uuid::Uuid::new_v3(
                &uuid::Uuid::nil(),
                format!("OfflinePlayer:{name}").as_bytes(),
            )
        };
        Ok(Self { name, uuid })
    }
}

impl crate::ClientPacket for SLoginStart {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_string_bounded(&self.name, 16)?;
        if version >= &JavaMinecraftVersion::V_1_20_2 {
            write.write_uuid(&self.uuid)?;
        } else if version >= &JavaMinecraftVersion::V_1_19_3 {
            write.write_bool(true)?;
            write.write_uuid(&self.uuid)?;
        }
        Ok(())
    }
}
