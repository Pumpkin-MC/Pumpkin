use pumpkin_data::packet::clientbound::PLAY_SELECT_ADVANCEMENTS_TAB;
use pumpkin_macros::java_packet;
use pumpkin_util::identifier::Identifier;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SELECT_ADVANCEMENTS_TAB)]
pub struct CSelectAdvancementsTab {
    pub tab_id: Option<Identifier>,
}

impl CSelectAdvancementsTab {
    #[must_use]
    pub const fn new(tab_id: Option<Identifier>) -> Self {
        Self { tab_id }
    }
}

impl ClientPacket for CSelectAdvancementsTab {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_option(&self.tab_id, |w, id| w.write_string(&id.to_string()))?;
        Ok(())
    }
}
