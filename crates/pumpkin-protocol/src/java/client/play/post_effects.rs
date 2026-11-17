use pumpkin_data::packet::clientbound::play::POST_EFFECTS;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;

#[java_packet(POST_EFFECTS)]
pub struct CPostEffects<'a> {
    pub effects: &'a [String],
}

impl<'a> CPostEffects<'a> {
    #[must_use]
    pub const fn new(effects: &'a [String]) -> Self {
        Self { effects }
    }
}

impl ClientPacket for CPostEffects<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        if *version >= JavaMinecraftVersion::V_26_3 {
            write.write_list(self.effects, |w, effect| w.write_string(effect))?;
        }
        Ok(())
    }
}
