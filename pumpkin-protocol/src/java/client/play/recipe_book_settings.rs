use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_RECIPE_BOOK_SETTINGS;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

use crate::{ClientPacket, WritingError, ser::NetworkWriteExt};

/// Sent by the server to update the player's recipe book open/filter state.
/// Wire format: 4 TypeSettings pairs (crafting, furnace, blast_furnace, smoker),
/// each pair is (is_open: bool, is_filtering: bool).
#[java_packet(PLAY_RECIPE_BOOK_SETTINGS)]
pub struct CRecipeBookSettings {
    pub crafting_open: bool,
    pub crafting_filtering: bool,
    pub furnace_open: bool,
    pub furnace_filtering: bool,
    pub blast_furnace_open: bool,
    pub blast_furnace_filtering: bool,
    pub smoker_open: bool,
    pub smoker_filtering: bool,
}

impl CRecipeBookSettings {
    #[must_use]
    pub fn default_closed() -> Self {
        Self {
            crafting_open: false,
            crafting_filtering: false,
            furnace_open: false,
            furnace_filtering: false,
            blast_furnace_open: false,
            blast_furnace_filtering: false,
            smoker_open: false,
            smoker_filtering: false,
        }
    }
}

impl ClientPacket for CRecipeBookSettings {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;
        write.write_bool(self.crafting_open)?;
        write.write_bool(self.crafting_filtering)?;
        write.write_bool(self.furnace_open)?;
        write.write_bool(self.furnace_filtering)?;
        write.write_bool(self.blast_furnace_open)?;
        write.write_bool(self.blast_furnace_filtering)?;
        write.write_bool(self.smoker_open)?;
        write.write_bool(self.smoker_filtering)?;
        Ok(())
    }
}
