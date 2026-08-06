use crate::{
    bedrock::network_item::NetworkItemDescriptor,
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};
use pumpkin_macros::packet;
use std::io::{Error, Write};

#[derive(Clone, Debug)]
pub struct ItemDescriptorCount {
    pub network_id: i16,
    pub metadata_value: i16,
    pub count: i32,
}

impl PacketWrite for ItemDescriptorCount {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        if self.network_id == 0 {
            // Invalid item descriptor (type 0)
            0u8.write(writer)?;
        } else {
            // Default item descriptor (type 1)
            1u8.write(writer)?;
            self.network_id.write(writer)?;
            self.metadata_value.write(writer)?;
        }
        VarInt(self.count).write(writer)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct RecipeUnlockRequirement {
    pub context: u8,
}

impl PacketWrite for RecipeUnlockRequirement {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.context.write(writer)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct BedrockShapelessRecipe {
    pub recipe_id: String,
    pub input: Vec<ItemDescriptorCount>,
    pub output: Vec<NetworkItemDescriptor>,
    pub uuid: [u8; 16],
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
            item.write(writer)?;
        }

        // uuid
        writer.write_all(&self.uuid)?;

        // block
        self.block.write(writer)?;

        // priority
        self.priority.write(writer)?;

        // unlock_requirement
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
    pub uuid: [u8; 16],
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

        // input slice: width * height elements sequentially (no count prefix)
        for item in &self.input {
            item.write(writer)?;
        }

        // output slice with VarUInt length prefix
        VarUInt(self.output.len() as u32).write(writer)?;
        for item in &self.output {
            item.write(writer)?;
        }

        // uuid
        writer.write_all(&self.uuid)?;

        // block
        self.block.write(writer)?;

        // priority
        self.priority.write(writer)?;

        // assume_symmetry
        self.assume_symmetry.write(writer)?;

        // unlock_requirement
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
        // recipes slice with VarUInt prefix
        VarUInt(self.recipes.len() as u32).write(writer)?;
        for recipe in &self.recipes {
            recipe.write(writer)?;
        }

        // potion recipes count prefix (0)
        VarUInt(0).write(writer)?;
        // potion container change recipes count prefix (0)
        VarUInt(0).write(writer)?;
        // material reducers count prefix (0)
        VarUInt(0).write(writer)?;

        // clean_recipes
        self.clean_recipes.write(writer)?;

        Ok(())
    }
}
