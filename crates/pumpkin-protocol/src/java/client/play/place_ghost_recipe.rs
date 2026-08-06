use std::io::Write;

use pumpkin_data::item::Item;
use pumpkin_data::packet::clientbound::PLAY_PLACE_GHOST_RECIPE;
use pumpkin_data::recipes::CraftingRecipeTypes;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::codec::recipe::OwnedCraftingRecipe;
use crate::ser::{NetworkWriteExt, WritingError};
use crate::{ClientPacket, VarInt};

use super::recipe_book_add::{write_dynamic_recipe_display, write_recipe_display};

/// Which recipe to show as a ghost overlay -- either a vanilla static recipe or a
/// data-pack (dynamic) one, matching the two sources `handle_place_recipe` already
/// resolves a target recipe from.
pub enum GhostRecipeSource<'a> {
    Static(&'a CraftingRecipeTypes),
    Dynamic(&'a OwnedCraftingRecipe),
}

/// Tells the client to render a recipe's ingredients as a greyed-out "ghost"
/// overlay in the crafting grid, because the player doesn't have everything
/// needed to actually place it.
#[java_packet(PLAY_PLACE_GHOST_RECIPE)]
pub struct CPlaceGhostRecipe<'a> {
    pub sync_id: VarInt,
    pub recipe: GhostRecipeSource<'a>,
}

impl<'a> CPlaceGhostRecipe<'a> {
    #[must_use]
    pub const fn new(sync_id: VarInt, recipe: GhostRecipeSource<'a>) -> Self {
        Self { sync_id, recipe }
    }
}

impl ClientPacket for CPlaceGhostRecipe<'_> {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_var_int(&self.sync_id)?;

        let crafting_table = Item::from_registry_key("crafting_table")
            .ok_or_else(|| WritingError::Message("crafting_table item must exist".into()))?;

        match self.recipe {
            GhostRecipeSource::Static(recipe) => {
                let furnace = Item::from_registry_key("furnace")
                    .ok_or_else(|| WritingError::Message("furnace item must exist".into()))?;
                let blast_furnace = Item::from_registry_key("blast_furnace")
                    .ok_or_else(|| WritingError::Message("blast_furnace item must exist".into()))?;
                let smoker = Item::from_registry_key("smoker")
                    .ok_or_else(|| WritingError::Message("smoker item must exist".into()))?;
                let campfire = Item::from_registry_key("campfire")
                    .ok_or_else(|| WritingError::Message("campfire item must exist".into()))?;
                write_recipe_display(
                    &mut write,
                    *version,
                    crafting_table,
                    furnace,
                    blast_furnace,
                    smoker,
                    campfire,
                    Some(recipe),
                    None,
                )?;
            }
            GhostRecipeSource::Dynamic(recipe) => {
                write_dynamic_recipe_display(&mut write, *version, crafting_table, recipe)?;
            }
        }

        Ok(())
    }
}
