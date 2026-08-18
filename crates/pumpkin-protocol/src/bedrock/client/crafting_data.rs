use crate::{
    bedrock::network_item::NetworkItemDescriptor,
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};
use pumpkin_macros::packet;
use std::io::{Error, Write};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ItemDescriptorCount {
    pub item_identifier: String,
    pub metadata_value: i32,
    pub count: i32,
}

impl PacketWrite for ItemDescriptorCount {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        if self.item_identifier.is_empty() {
            VarUInt(0).write(writer)?;
            VarInt(32767).write(writer)?;
        } else {
            VarUInt(1).write(writer)?;
            "name".write(writer)?;
            self.item_identifier.write(writer)?;
            VarInt(self.metadata_value).write(writer)?;
        }
        VarInt(self.count).write(writer)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct RecipeUnlockRequirement {
    pub context: i32,
}

impl PacketWrite for RecipeUnlockRequirement {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarInt(self.context).write(writer)?;
        // Context NONE carries an optional ingredient list; Pumpkin currently
        // sends ALWAYS for its generated recipes.
        false.write(writer)
    }
}

#[derive(Clone, Debug)]
pub struct BedrockShapelessRecipe {
    pub recipe_id: String,
    pub input: Vec<ItemDescriptorCount>,
    pub output: Vec<NetworkItemDescriptor>,
    pub uuid: Uuid,
    pub block: String,
    pub priority: VarInt,
    pub unlock_requirement: RecipeUnlockRequirement,
    pub recipe_network_id: VarUInt,
}

impl PacketWrite for BedrockShapelessRecipe {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.recipe_id.write(writer)?;

        // input slice with VarUInt length prefix
        VarUInt(self.input.len() as u32).write(writer)?;
        for item in &self.input {
            item.write(writer)?;
        }

        // output slice with VarUInt length prefix
        VarUInt(self.output.len() as u32).write(writer)?;
        for item in &self.output {
            item.write_item_instance(writer)?;
        }

        // uuid
        self.uuid.write(writer)?;

        // block
        self.block.write(writer)?;

        // priority
        self.priority.write(writer)?;

        true.write(writer)?;
        self.unlock_requirement.write(writer)?;

        // recipe_network_id
        self.recipe_network_id.write(writer)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct BedrockShapedRecipe {
    pub recipe_id: String,
    pub width: VarInt,
    pub height: VarInt,
    pub input: Vec<ItemDescriptorCount>,
    pub output: Vec<NetworkItemDescriptor>,
    pub uuid: Uuid,
    pub block: String,
    pub priority: VarInt,
    pub assume_symmetry: bool,
    pub unlock_requirement: RecipeUnlockRequirement,
    pub recipe_network_id: VarUInt,
}

impl PacketWrite for BedrockShapedRecipe {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.recipe_id.write(writer)?;
        self.width.write(writer)?;
        self.height.write(writer)?;

        VarUInt(self.input.len() as u32).write(writer)?;
        for item in &self.input {
            item.write(writer)?;
        }

        // output slice with VarUInt length prefix
        VarUInt(self.output.len() as u32).write(writer)?;
        for item in &self.output {
            item.write_item_instance(writer)?;
        }

        // uuid
        self.uuid.write(writer)?;

        // block
        self.block.write(writer)?;

        // priority
        self.priority.write(writer)?;

        // assume_symmetry
        self.assume_symmetry.write(writer)?;

        true.write(writer)?;
        self.unlock_requirement.write(writer)?;

        // recipe_network_id
        self.recipe_network_id.write(writer)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum BedrockRecipe {
    Shapeless(BedrockShapelessRecipe),
    Shaped(BedrockShapedRecipe),
}

impl PacketWrite for BedrockRecipe {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        match self {
            Self::Shapeless(recipe) => {
                VarInt(0).write(writer)?; // type 0: Shapeless
                recipe.write(writer)?;
            }
            Self::Shaped(recipe) => {
                VarInt(1).write(writer)?; // type 1: Shaped
                recipe.write(writer)?;
            }
        }
        Ok(())
    }
}

#[packet(52)]
pub struct CCraftingData {
    pub recipes: Vec<BedrockRecipe>,
    pub clean_recipes: bool,
}

impl PacketWrite for CCraftingData {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let shaped = self
            .recipes
            .iter()
            .filter_map(|recipe| match recipe {
                BedrockRecipe::Shaped(recipe) => Some(recipe),
                BedrockRecipe::Shapeless(_) => None,
            })
            .collect::<Vec<_>>();
        VarUInt(shaped.len() as u32).write(writer)?;
        for recipe in shaped {
            recipe.write(writer)?;
        }

        let shapeless = self
            .recipes
            .iter()
            .filter_map(|recipe| match recipe {
                BedrockRecipe::Shapeless(recipe) => Some(recipe),
                BedrockRecipe::Shaped(_) => None,
            })
            .collect::<Vec<_>>();
        VarUInt(shapeless.len() as u32).write(writer)?;
        for recipe in shapeless {
            recipe.write(writer)?;
        }

        // Multi, user, chemistry, smithing, potion, container and material arrays.
        for _ in 0..9 {
            VarUInt(0).write(writer)?;
        }

        // clean_recipes
        self.clean_recipes.write(writer)?;

        Ok(())
    }
}
