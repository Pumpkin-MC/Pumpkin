use std::io::{Error, Write};

use pumpkin_nbt::Nbt;

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};

#[derive(Default, Clone)]
pub struct NetworkItemDescriptor {
    // I hate mojang
    // https://mojang.github.io/bedrock-protocol-docs/html/NetworkItemInstanceDescriptor.html
    pub id: VarInt,
    pub stack_size: u16,
    pub aux_value: VarUInt,
    pub block_runtime_id: VarInt,

    // remainder is expansion of `User Data Buffer` (ItemInstanceUserData)
    pub nbt_data: Nbt,
    pub place_on_blocks: Vec<String>,
    pub destroy_blocks: Vec<String>,
}

impl PacketWrite for NetworkItemDescriptor {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.id.write(writer)?;
        if self.id.0 != 0 {
            self.stack_size.write(writer)?;
            self.aux_value.write(writer)?;
            self.block_runtime_id.write(writer)?;

            let mut buf = Vec::new();
            {
                if self.nbt_data.is_empty() {
                    (0i16).write(&mut buf)?;
                } else {
                    (-1i16).write(&mut buf)?;
                    (1i8).write(&mut buf)?;

                    self.nbt_data.clone().write_to_writer_bedrock(&mut buf)?;
                }

                (self.place_on_blocks.len() as u32).write(&mut buf)?;
                self.place_on_blocks.write(&mut buf)?;

                (self.destroy_blocks.len() as u32).write(&mut buf)?;
                self.destroy_blocks.write(&mut buf)?;
            }
            VarUInt(buf.len() as u32).write(writer)?;
            writer.write_all(&buf)?
        }
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct NetworkItemStackDescriptor {
    // I hate mojang
    // https://mojang.github.io/bedrock-protocol-docs/html/cerealizer_NetworkItemStackDescriptor___SerializedData.html
    pub item: NetworkItemDescriptor,
    pub net_id: i32,
}

impl PacketWrite for NetworkItemStackDescriptor {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.item.id.write(writer)?;
        if self.item.id.0 != 0 {
            self.item.stack_size.write(writer)?;
            self.item.aux_value.write(writer)?;

            if self.net_id == 0 {
                None
            } else {
                Some(VarInt(self.net_id))
            }
            .write(writer)?; // the only difference from NetworkItemDescriptor

            self.item.block_runtime_id.write(writer)?;

            let mut buf = Vec::new();
            {
                if self.item.nbt_data.is_empty() {
                    (0i16).write(&mut buf)?;
                } else {
                    (-1i16).write(&mut buf)?;
                    (1i8).write(&mut buf)?;

                    self.item
                        .nbt_data
                        .clone()
                        .write_to_writer_bedrock(&mut buf)?;
                }

                (self.item.place_on_blocks.len() as u32).write(&mut buf)?;
                self.item.place_on_blocks.write(&mut buf)?;

                (self.item.destroy_blocks.len() as u32).write(&mut buf)?;
                self.item.destroy_blocks.write(&mut buf)?;
            }
            VarUInt(buf.len() as u32).write(writer)?;
            writer.write_all(&buf)?
        }
        Ok(())
    }
}

#[derive(PacketWrite, Clone)]
pub struct FullContainerName {
    pub container_name: ContainerName,
    pub dynamic_id: Option<u32>,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ContainerName {
    AnvilInput,
    AnvilMaterial,
    AnvilResultPreview,
    SmithingTableInput,
    SmithingTableMaterial,
    SmithingTableResultPreview,
    Armor,
    LevelEntity,
    BeaconPayment,
    BrewingStandInput,
    BrewingStandResult,
    BrewingStandFuel,
    CombinedHotBarAndInventory,
    CraftingInput,
    CraftingOutputPreview,
    RecipeConstruction,
    RecipeNature,
    RecipeItems,
    RecipeSearch,
    RecipeSearchBar,
    RecipeEquipment,
    RecipeBook,
    EnchantingInput,
    EnchantingMaterial,
    FurnaceFuel,
    FurnaceIngredient,
    FurnaceResult,
    HorseEquip,
    HotBar,
    Inventory,
    ShulkerBox,
    TradeIngredient1,
    TradeIngredient2,
    TradeResultPreview,
    Offhand,
    CompoundCreatorInput,
    CompoundCreatorOutputPreview,
    ElementConstructorOutputPreview,
    MaterialReducerInput,
    MaterialReducerOutput,
    LabTableInput,
    LoomInput,
    LoomDye,
    LoomMaterial,
    LoomResultPreview,
    BlastFurnaceIngredient,
    SmokerIngredient,
    Trade2Ingredient1,
    Trade2Ingredient2,
    Trade2ResultPreview,
    GrindstoneInput,
    GrindstoneAdditional,
    GrindstoneResultPreview,
    StonecutterInput,
    StonecutterResultPreview,
    CartographyInput,
    CartographyAdditional,
    CartographyResultPreview,
    Barrel,
    Cursor,
    CreatedOutput,
    SmithingTableTemplate,
    CrafterLevelEntity,
    Dynamic,
    RecipeFood,
    RecipeBlocks,
    RecipeFurnaceItems,
}

impl PacketWrite for ContainerName {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as u8).write(writer)?;
        Ok(())
    }
}
