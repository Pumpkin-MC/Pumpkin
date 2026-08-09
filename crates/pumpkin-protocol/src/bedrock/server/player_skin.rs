use std::io::{Error, Read};

use pumpkin_macros::packet;
use uuid::Uuid;

use crate::{bedrock::client::Skin, serial::PacketRead};

/// Sent when a Bedrock player changes their skin while connected.
#[packet(93)]
pub struct SPlayerSkin {
    pub uuid: Uuid,
    pub skin: Skin,
    pub new_skin_name: String,
    pub old_skin_name: String,
}

impl PacketRead for SPlayerSkin {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            uuid: Uuid::read(reader)?,
            skin: Skin::read(reader)?,
            new_skin_name: String::read(reader)?,
            old_skin_name: String::read(reader)?,
        })
    }
}
