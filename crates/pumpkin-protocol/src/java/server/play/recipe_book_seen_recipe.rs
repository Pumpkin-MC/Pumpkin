use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::RECIPE_BOOK_SEEN_RECIPE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(RECIPE_BOOK_SEEN_RECIPE)]
pub struct SRecipeBookSeenRecipe {
    pub recipe_display_id: VarInt,
}

impl<'a> ServerPacket<'a> for SRecipeBookSeenRecipe {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        if *version >= JavaMinecraftVersion::V_1_21_2 {
            Ok(Self {
                recipe_display_id: bytebuf.get_var_int()?,
            })
        } else {
            let _recipe = bytebuf.get_str_borrowed()?;
            Ok(Self {
                recipe_display_id: VarInt(0),
            })
        }
    }
}

impl crate::ClientPacket for SRecipeBookSeenRecipe {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.recipe_display_id)?;
        Ok(())
    }
}
