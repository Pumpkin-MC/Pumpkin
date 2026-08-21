use pumpkin_data::packet::serverbound::login::HELLO;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};

#[java_packet(HELLO)]
pub struct SLoginStart {
    pub name: Box<str>, // 16
    pub uuid: uuid::Uuid,
}

impl<'a> ServerPacket<'a> for SLoginStart {
    fn read(read: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let name = read.get_str_bounded(16)?;
        let mut uuid = None;

        if version >= &JavaMinecraftVersion::V_1_19 {
            if version <= &JavaMinecraftVersion::V_1_19_3 {
                let has_signature_data = read.get_bool()?;
                if has_signature_data {
                    let _expires_at = read.get_i64_be()?;
                    let public_key_len = usize::try_from(read.get_var_int()?.0)
                        .map_err(|_| ReadingError::Message("Negative public key length".into()))?;
                    let _public_key = read.read_slice_borrowed(public_key_len)?;
                    let signature_len = usize::try_from(read.get_var_int()?.0)
                        .map_err(|_| ReadingError::Message("Negative signature length".into()))?;
                    let _signature = read.read_slice_borrowed(signature_len)?;
                }
            }

            if version >= &JavaMinecraftVersion::V_1_20_2 {
                uuid = Some(read.get_uuid()?);
            } else if version >= &JavaMinecraftVersion::V_1_19_1 {
                let has_uuid = read.get_bool()?;
                if has_uuid {
                    uuid = Some(read.get_uuid()?);
                }
            }
        }

        let uuid = uuid.unwrap_or_else(|| {
            uuid::Uuid::new_v3(
                &uuid::Uuid::nil(),
                format!("OfflinePlayer:{name}").as_bytes(),
            )
        });

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
        if version >= &JavaMinecraftVersion::V_1_19 {
            if version <= &JavaMinecraftVersion::V_1_19_3 {
                write.write_bool(false)?;
            }
            if version >= &JavaMinecraftVersion::V_1_20_2 {
                write.write_uuid(&self.uuid)?;
            } else if version >= &JavaMinecraftVersion::V_1_19_1 {
                write.write_bool(true)?;
                write.write_uuid(&self.uuid)?;
            }
        }
        Ok(())
    }
}
