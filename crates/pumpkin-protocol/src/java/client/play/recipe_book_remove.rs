use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::RECIPE_BOOK_REMOVE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(RECIPE_BOOK_REMOVE)]
pub struct CRecipeBookRemove<'a> {
    pub recipes: &'a [VarInt],
}

impl<'a> CRecipeBookRemove<'a> {
    #[must_use]
    pub const fn new(recipes: &'a [VarInt]) -> Self {
        Self { recipes }
    }
}

impl ClientPacket for CRecipeBookRemove<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_var_int(&VarInt(self.recipes.len() as i32))?;
        for recipe in self.recipes {
            write.write_var_int(recipe)?;
        }
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CRecipeBookRemove<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let count = bytebuf.get_var_int()?.0 as usize;
        let mut recipes = Vec::with_capacity(count);
        for _ in 0..count {
            recipes.push(bytebuf.get_var_int()?);
        }
        Ok(Self {
            recipes: Box::leak(recipes.into_boxed_slice()),
        })
    }
}
