use std::io::{Error, ErrorKind, Read};

use crate::{codec::var_uint::VarUInt, serial::PacketRead};
use pumpkin_macros::packet;

#[packet(8)]
pub struct SResourcePackResponse {
    pub response: u8,
    pub download_size: u16,
    pub pack_ids: Vec<String>,
}

impl PacketRead for SResourcePackResponse {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let encoded_status = VarUInt::read(reader)?.0;
        let response = encoded_status
            .checked_add(1)
            .and_then(|v| u8::try_from(v).ok())
            .ok_or_else(|| {
                Error::new(ErrorKind::InvalidData, "resource pack status is too large")
            })?;
        let _status_name = String::read(reader)?;

        let pack_ids = if response == Self::STATUS_SEND_PACKS {
            let count = VarUInt::read(reader)?.0;
            if count > 1024 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "too many resource pack identifiers",
                ));
            }
            (0..count)
                .map(|_| String::read(reader))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };
        let download_size = u16::try_from(pack_ids.len()).map_err(|_| {
            Error::new(ErrorKind::InvalidData, "too many resource pack identifiers")
        })?;

        Ok(Self {
            response,
            download_size,
            pack_ids,
        })
    }
}

impl SResourcePackResponse {
    pub const STATUS_REFUSED: u8 = 1;
    pub const STATUS_SEND_PACKS: u8 = 2;
    pub const STATUS_HAVE_ALL_PACKS: u8 = 3;
    pub const STATUS_COMPLETED: u8 = 4;
}
