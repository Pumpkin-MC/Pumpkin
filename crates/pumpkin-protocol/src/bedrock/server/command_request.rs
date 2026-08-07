use pumpkin_macros::packet;
use std::borrow::Cow;
use uuid::Uuid;

use crate::serial::{PacketRead, PacketReadSlice, read_str_slice};

#[derive(Debug)]
#[packet(77)]
pub struct SCommandRequest<'a> {
    pub command: Cow<'a, str>,
    pub command_type: Cow<'a, str>,
    pub command_uuid: Uuid,
    pub request_id: Cow<'a, str>,
    pub player_actor_unique_id: i64,
    pub is_internal_source: bool,
    pub version: Cow<'a, str>,
}

impl<'a> PacketReadSlice<'a> for SCommandRequest<'a> {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, std::io::Error> {
        let command = Cow::Borrowed(read_str_slice(buf)?);
        let command_type = Cow::Borrowed(read_str_slice(buf)?);
        let command_uuid = Uuid::read_slice(buf)?;
        let request_id = Cow::Borrowed(read_str_slice(buf)?);
        let player_actor_unique_id = i64::read_slice(buf)?;
        let is_internal_source = bool::read_slice(buf)?;
        let version = Cow::Borrowed(read_str_slice(buf)?);
        Ok(Self {
            command,
            command_type,
            command_uuid,
            request_id,
            player_actor_unique_id,
            is_internal_source,
            version,
        })
    }
}

impl PacketRead for SCommandRequest<'static> {
    fn read<R: std::io::Read>(reader: &mut R) -> Result<Self, std::io::Error> {
        let command = Cow::Owned(String::read(reader)?);
        let command_type = Cow::Owned(String::read(reader)?);
        let command_uuid = Uuid::read(reader)?;
        let request_id = Cow::Owned(String::read(reader)?);
        let player_actor_unique_id = i64::read(reader)?;
        let is_internal_source = bool::read(reader)?;
        let version = Cow::Owned(String::read(reader)?);

        Ok(Self {
            command,
            command_type,
            command_uuid,
            request_id,
            player_actor_unique_id,
            is_internal_source,
            version,
        })
    }
}
