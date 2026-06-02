use std::io::{Error, Write};
use std::num::NonZeroI32;

use pumpkin_data::item::{BedrockItem, JavaToBedrockItemMapping};
use pumpkin_data::item_stack::ItemStack;
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

    pub shield_blocking_tick: i64,
}

impl PacketWrite for NetworkItemDescriptor {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.write_with_net_id(writer, None)
    }
}

impl NetworkItemDescriptor {
    #[allow(clippy::option_option)]
    fn write_with_net_id<W: Write>(
        &self,
        writer: &mut W,
        net_id: Option<Option<VarInt>>,
    ) -> Result<(), Error> {
        self.id.write(writer)?;
        if self.id.0 != 0 {
            self.stack_size.write(writer)?;
            self.aux_value.write(writer)?;

            if let Some(id) = net_id {
                id.write(writer)?;
            }

            self.block_runtime_id.write(writer)?;

            let mut buf = Vec::new();

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

            if self.id.0 == (BedrockItem::SHIELD.id as i32) {
                self.shield_blocking_tick.write(&mut buf)?;
            }

            VarUInt(buf.len() as u32).write(writer)?;
            writer.write_all(&buf)?;
        }
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct NetworkItemStackDescriptor {
    // I hate mojang
    // https://mojang.github.io/bedrock-protocol-docs/html/cerealizer_NetworkItemStackDescriptor___SerializedData.html
    pub item: NetworkItemDescriptor,
    pub net_id: Option<NonZeroI32>,
}

impl PacketWrite for NetworkItemStackDescriptor {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.item
            .write_with_net_id(writer, Some(self.net_id.map(|id| VarInt(id.get()))))
    }
}

impl From<&ItemStack> for NetworkItemStackDescriptor {
    fn from(stack: &ItemStack) -> Self {
        if stack.is_empty() {
            Self::default()
        } else {
            JavaToBedrockItemMapping::from_java_item_id(stack.get_item().id).map_or(
                Self::default(),
                |mapping| Self {
                    item: NetworkItemDescriptor {
                        id: VarInt::from(mapping.bedrock_item.id),
                        stack_size: stack.item_count as u16,
                        aux_value: VarUInt(mapping.bedrock_data),
                        block_runtime_id: VarInt::from(mapping.bedrock_block_state),
                        nbt_data: Nbt::default(),
                        place_on_blocks: Vec::default(),
                        destroy_blocks: Vec::default(),
                        shield_blocking_tick: 0,
                    },
                    net_id: Some(stack.uid),
                },
            )
        }
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
