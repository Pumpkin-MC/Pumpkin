use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::PLACE_RECIPE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(PLACE_RECIPE)]
pub struct SPlaceRecipe {
    pub container_id: i8,
    pub recipe_display_id: VarInt,
    pub use_max_items: bool,
}

impl<'a> ServerPacket<'a> for SPlaceRecipe {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        if *version >= JavaMinecraftVersion::V_1_21_2 {
            let container_id = bytebuf.get_container_id(version)?.0 as i8;
            let recipe_display_id = bytebuf.get_var_int()?;
            let use_max_items = bytebuf.get_bool()?;
            Ok(Self {
                container_id,
                recipe_display_id,
                use_max_items,
            })
        } else if *version >= JavaMinecraftVersion::V_1_13 {
            let container_id = bytebuf.get_i8()?;
            let _recipe_key = bytebuf.get_str_borrowed()?;
            let use_max_items = bytebuf.get_bool()?;
            Ok(Self {
                container_id,
                recipe_display_id: VarInt(0),
                use_max_items,
            })
        } else {
            let container_id = bytebuf.get_i8()?;
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
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_i8(self.container_id)?;
        write.write_var_int(&self.recipe_display_id)?;
        write.write_bool(self.use_max_items)?;
        Ok(())
    }
}
