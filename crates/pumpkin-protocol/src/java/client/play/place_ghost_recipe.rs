use pumpkin_data::packet::clientbound::PLAY_PLACE_GHOST_RECIPE;
use pumpkin_macros::java_packet;

use crate::{ClientPacket, ser::NetworkWriteExt};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_PLACE_GHOST_RECIPE)]
pub struct CPlaceGhostRecipe<'a> {
    pub window_id: u8,
    pub recipe_id: &'a str,
}

impl<'a> CPlaceGhostRecipe<'a> {
    #[must_use]
    pub const fn new(window_id: u8, recipe_id: &'a str) -> Self {
        Self {
            window_id,
            recipe_id,
        }
    }
}

impl ClientPacket for CPlaceGhostRecipe<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_u8(self.window_id)?;
        write.write_string(self.recipe_id)?;
        Ok(())
    }
}
