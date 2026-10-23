use pumpkin_data::packet::clientbound::config::POST_EFFECTS;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;

#[java_packet(POST_EFFECTS)]
pub struct CConfigPostEffects<'a> {
    pub effects: &'a [String],
}

impl<'a> CConfigPostEffects<'a> {
    #[must_use]
    pub const fn new(effects: &'a [String]) -> Self {
        Self { effects }
    }
}

impl ClientPacket for CConfigPostEffects<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_list(self.effects, |w, effect| w.write_string(effect))?;
        Ok(())
    }
}
