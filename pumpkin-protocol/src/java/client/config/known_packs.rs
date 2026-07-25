use pumpkin_data::packet::clientbound::CONFIG_SELECT_KNOWN_PACKS;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::KnownPack;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIG_SELECT_KNOWN_PACKS)]
pub struct CKnownPacks<'a> {
    pub known_packs: &'a [KnownPack<'a>],
}

impl<'a> CKnownPacks<'a> {
    #[must_use]
    pub const fn new(known_packs: &'a [KnownPack]) -> Self {
        Self { known_packs }
    }
}

impl ClientPacket for CKnownPacks<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&crate::VarInt(self.known_packs.len() as i32))?;
        for pack in self.known_packs {
            pack.write(&mut write)?;
        }
        Ok(())
    }
}
