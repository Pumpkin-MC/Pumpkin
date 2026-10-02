use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::PLACE_RECIPE;
use pumpkin_macros::java_packet;

use crate::VarInt;

#[java_packet(PLACE_RECIPE)]
pub struct SPlaceRecipe {
    pub container_id: i8,
    pub recipe_display_id: VarInt,
    pub use_max_items: bool,
}

impl<'a> ServerPacket<'a> for SPlaceRecipe {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        {
            let container_id = bytebuf.get_container_id()?.0 as i8;
            let recipe_display_id = bytebuf.get_var_int()?;
            let use_max_items = bytebuf.get_bool()?;
            Ok(Self {
                container_id,
                recipe_display_id,
                use_max_items,
            })
        }
    }
}

impl crate::ClientPacket for SPlaceRecipe {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_i8(self.container_id)?;
        write.write_var_int(&self.recipe_display_id)?;
        write.write_bool(self.use_max_items)?;
        Ok(())
    }
}
