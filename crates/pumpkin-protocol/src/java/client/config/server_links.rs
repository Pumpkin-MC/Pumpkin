use crate::Link;
use pumpkin_data::packet::clientbound::CONFIG_SERVER_LINKS;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIG_SERVER_LINKS)]
pub struct CConfigServerLinks<'a> {
    pub links: &'a [Link<'a>],
}

impl<'a> CConfigServerLinks<'a> {
    #[must_use]
    pub const fn new(links: &'a [Link<'a>]) -> Self {
        Self { links }
    }
}

impl ClientPacket for CConfigServerLinks<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&crate::VarInt(self.links.len() as i32))?;
        for link in self.links {
            link.write(&mut write)?;
        }
        Ok(())
    }
}
