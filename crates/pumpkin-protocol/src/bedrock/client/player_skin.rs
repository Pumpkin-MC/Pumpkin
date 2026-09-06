use std::io::{Error, Write};

use pumpkin_macros::packet;
use uuid::Uuid;

use crate::serial::PacketWrite;

use super::Skin;

/// Broadcasts an accepted Bedrock player skin change.
#[packet(93)]
pub struct CPlayerSkin<'a> {
    pub uuid: Uuid,
    pub skin: &'a Skin,
    pub new_skin_name: &'a str,
    pub old_skin_name: &'a str,
}

impl PacketWrite for CPlayerSkin<'_> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.uuid.write(writer)?;
        self.skin.write(writer)?;
        self.new_skin_name.write(writer)?;
        self.old_skin_name.write(writer)
    }
}
