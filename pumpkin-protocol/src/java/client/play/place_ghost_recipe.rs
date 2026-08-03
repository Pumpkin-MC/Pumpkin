use std::io::Write;

use pumpkin_data::{
    item::Item, packet::clientbound::PLAY_PLACE_GHOST_RECIPE, recipes::CraftingRecipeTypes,
};
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ClientPacket, VarInt, WritingError, codec::recipe::OwnedCraftingRecipe, ser::NetworkWriteExt,
};

use super::recipe_book_add::{
    write_crafting_recipe_display, write_dynamic_crafting_recipe_display,
};

/// The crafting-recipe display sent to the client as a ghost recipe.
#[derive(Clone, Copy)]
pub enum GhostRecipe<'a> {
    Vanilla(&'a CraftingRecipeTypes),
    Dynamic(&'a OwnedCraftingRecipe),
}

/// Shows a recipe in a container without moving real ingredients into its grid.
#[java_packet(PLAY_PLACE_GHOST_RECIPE)]
pub struct CPlaceGhostRecipe<'a> {
    pub container_id: i32,
    pub recipe: GhostRecipe<'a>,
}

impl<'a> CPlaceGhostRecipe<'a> {
    #[must_use]
    pub const fn new(container_id: i32, recipe: GhostRecipe<'a>) -> Self {
        Self {
            container_id,
            recipe,
        }
    }
}

impl ClientPacket for CPlaceGhostRecipe<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;
        write.write_var_int(&VarInt(self.container_id))?;

        let crafting_table = Item::from_registry_key("crafting_table")
            .ok_or_else(|| WritingError::Message("crafting_table item must exist".into()))?;
        match self.recipe {
            GhostRecipe::Vanilla(recipe) => {
                if !write_crafting_recipe_display(&mut write, recipe, crafting_table, *version)? {
                    return Err(WritingError::Message(
                        "special crafting recipes cannot be displayed as ghost recipes".into(),
                    ));
                }
            }
            GhostRecipe::Dynamic(recipe) => {
                write_dynamic_crafting_recipe_display(
                    &mut write,
                    recipe,
                    crafting_table,
                    *version,
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        codec::recipe::{OwnedRecipeIngredient, OwnedRecipeResult},
        ser::NetworkReadExt,
    };
    use pumpkin_data::recipes::RecipeCategoryTypes;

    #[test]
    fn dynamic_shapeless_packet_starts_with_container_and_display_type() {
        let recipe = OwnedCraftingRecipe::Shapeless {
            category: RecipeCategoryTypes::Misc,
            group: None,
            ingredients: vec![OwnedRecipeIngredient::Simple("minecraft:stick".into())],
            result: OwnedRecipeResult {
                item_id: "minecraft:campfire".into(),
                count: 1,
            },
        };

        let mut encoded = Vec::new();
        CPlaceGhostRecipe::new(7, GhostRecipe::Dynamic(&recipe))
            .write_packet_data(&mut encoded, &JavaMinecraftVersion::V_26_2)
            .unwrap();

        let mut encoded = encoded.as_slice();
        assert_eq!(encoded.get_var_int().unwrap().0, 7);
        assert_eq!(encoded.get_var_int().unwrap().0, 0); // shapeless RecipeDisplay
    }
}
