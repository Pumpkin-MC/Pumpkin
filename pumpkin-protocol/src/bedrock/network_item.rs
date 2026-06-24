use std::io::{Error, Read, Write};
use std::num::NonZeroI32;

use pumpkin_data::item::{BedrockItem, JavaToBedrockItemMapping};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::Nbt;

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::{PacketRead, PacketWrite},
};

#[derive(Default, Clone, Debug)]
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

impl PacketRead for NetworkItemDescriptor {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let id = VarInt::read(buf)?;
        if id.0 == 0 {
            return Ok(Self::default());
        }
        let stack_size = u16::read(buf)?;
        let aux_value = VarUInt::read(buf)?;

        let has_net_id = bool::read(buf)?;
        if has_net_id {
            let _net_id = VarInt::read(buf)?;
        }

        let block_runtime_id = VarInt::read(buf)?;

        let user_data_len = VarUInt::read(buf)?.0;
        let mut user_data = vec![0u8; user_data_len as usize];
        buf.read_exact(&mut user_data)?;

        Ok(Self {
            id,
            stack_size,
            aux_value,
            block_runtime_id,
            ..Default::default()
        })
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

impl From<&ItemStack> for NetworkItemDescriptor {
    fn from(stack: &ItemStack) -> Self {
        if stack.is_empty() {
            Self::default()
        } else {
            JavaToBedrockItemMapping::from_java_item_id(stack.get_item().id).map_or(
                Self::default(),
                |mapping| Self {
                    id: VarInt::from(mapping.bedrock_item.id),
                    stack_size: stack.item_count as u16,
                    aux_value: VarUInt(mapping.bedrock_data),
                    block_runtime_id: VarInt::from(mapping.bedrock_block_state),
                    nbt_data: Nbt::default(),
                    place_on_blocks: Vec::default(),
                    destroy_blocks: Vec::default(),
                    shield_blocking_tick: 0,
                },
            )
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct ItemStackWrapper {
    pub id: i16,
    pub stack_size: u16,
    pub aux_value: VarUInt,
    pub block_runtime_id: VarInt,
    pub nbt_data: Nbt,
    pub place_on_blocks: Vec<String>,
    pub destroy_blocks: Vec<String>,
    pub shield_blocking_tick: i64,
    pub net_id: Option<NonZeroI32>,
}

impl PacketWrite for ItemStackWrapper {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.id.write(writer)?;
        if self.id != 0 {
            self.stack_size.write(writer)?;
            self.aux_value.write(writer)?;

            self.net_id.is_some().write(writer)?;
            if let Some(id) = self.net_id {
                VarUInt(0).write(writer)?; // variant
                VarInt(id.get()).write(writer)?;
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

            if self.id == BedrockItem::SHIELD.id {
                self.shield_blocking_tick.write(&mut buf)?;
            }

            VarUInt(buf.len() as u32).write(writer)?;
            writer.write_all(&buf)?;
        }
        Ok(())
    }
}

impl PacketRead for ItemStackWrapper {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let id = i16::read(buf)?;
        if id == 0 {
            return Ok(Self::default());
        }
        let stack_size = u16::read(buf)?;
        let aux_value = VarUInt::read(buf)?;

        let has_net_id = bool::read(buf)?;
        let net_id = if has_net_id {
            let _variant = VarUInt::read(buf)?;
            let stack_id = VarInt::read(buf)?;
            NonZeroI32::new(stack_id.0)
        } else {
            None
        };

        let block_runtime_id = VarInt::read(buf)?;

        let user_data_len = VarUInt::read(buf)?.0;
        let mut user_data = vec![0u8; user_data_len as usize];
        buf.read_exact(&mut user_data)?;

        Ok(Self {
            id,
            stack_size,
            aux_value,
            block_runtime_id,
            net_id,
            ..Default::default()
        })
    }
}

impl From<&ItemStack> for ItemStackWrapper {
    fn from(stack: &ItemStack) -> Self {
        if stack.is_empty() {
            Self::default()
        } else {
            JavaToBedrockItemMapping::from_java_item_id(stack.get_item().id).map_or(
                Self::default(),
                |mapping| Self {
                    id: mapping.bedrock_item.id,
                    stack_size: stack.item_count as u16,
                    aux_value: VarUInt(mapping.bedrock_data),
                    block_runtime_id: VarInt::from(mapping.bedrock_block_state),
                    nbt_data: Nbt::default(),
                    place_on_blocks: Vec::default(),
                    destroy_blocks: Vec::default(),
                    shield_blocking_tick: 0,
                    net_id: Some(stack.uid),
                },
            )
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct NetworkItemStackDescriptor {
    pub id: i16,
    pub stack_size: u16,
    pub aux_value: VarUInt,
    pub block_runtime_id: VarUInt,
    pub extra_data: Vec<u8>,
    pub net_id_variant: Option<VarUInt>,
    pub net_id: Option<NonZeroI32>,
}

impl PacketWrite for NetworkItemStackDescriptor {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.id.write(writer)?;

        self.stack_size.write(writer)?;
        self.aux_value.write(writer)?;

        self.net_id.is_some().write(writer)?;
        if let Some(id) = self.net_id {
            let variant = self.net_id_variant.unwrap_or(VarUInt(0));
            variant.write(writer)?;
            VarInt(id.get()).write(writer)?;
        }

        self.block_runtime_id.write(writer)?;

        VarUInt(self.extra_data.len() as u32).write(writer)?;
        writer.write_all(&self.extra_data)?;

        Ok(())
    }
}

impl PacketRead for NetworkItemStackDescriptor {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let id = i16::read(buf)?;

        let stack_size = u16::read(buf)?;
        let aux_value = VarUInt::read(buf)?;

        let has_net_id = bool::read(buf)?;
        let (net_id_variant, net_id) = if has_net_id {
            let variant = VarUInt::read(buf)?;
            let stack_id = VarInt::read(buf)?;
            (Some(variant), NonZeroI32::new(stack_id.0))
        } else {
            (None, None)
        };

        let block_runtime_id = VarUInt::read(buf)?;

        let extra_data_len = VarUInt::read(buf)?.0;
        let mut extra_data = vec![0u8; extra_data_len as usize];
        buf.read_exact(&mut extra_data)?;

        Ok(Self {
            id,
            stack_size,
            aux_value,
            block_runtime_id,
            extra_data,
            net_id_variant,
            net_id,
        })
    }
}

impl From<&ItemStack> for NetworkItemStackDescriptor {
    fn from(stack: &ItemStack) -> Self {
        if stack.is_empty() {
            Self::default()
        } else {
            JavaToBedrockItemMapping::from_java_item_id(stack.get_item().id).map_or(
                Self::default(),
                |mapping| {
                    let extra_data = vec![0u8, 0u8];

                    Self {
                        id: mapping.bedrock_item.id,
                        stack_size: stack.item_count as u16,
                        aux_value: VarUInt(mapping.bedrock_data),
                        block_runtime_id: VarUInt(mapping.bedrock_block_state as u32),
                        extra_data,
                        net_id_variant: Some(VarUInt(0)),
                        net_id: Some(stack.uid),
                    }
                },
            )
        }
    }
}

#[derive(PacketWrite, PacketRead, Clone, Debug, PartialEq, Eq)]
pub struct FullContainerName {
    pub container_name: ContainerName,
    pub dynamic_id: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl TryFrom<u8> for ContainerName {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::AnvilInput),
            1 => Ok(Self::AnvilMaterial),
            2 => Ok(Self::AnvilResultPreview),
            3 => Ok(Self::SmithingTableInput),
            4 => Ok(Self::SmithingTableMaterial),
            5 => Ok(Self::SmithingTableResultPreview),
            6 => Ok(Self::Armor),
            7 => Ok(Self::LevelEntity),
            8 => Ok(Self::BeaconPayment),
            9 => Ok(Self::BrewingStandInput),
            10 => Ok(Self::BrewingStandResult),
            11 => Ok(Self::BrewingStandFuel),
            12 => Ok(Self::CombinedHotBarAndInventory),
            13 => Ok(Self::CraftingInput),
            14 => Ok(Self::CraftingOutputPreview),
            15 => Ok(Self::RecipeConstruction),
            16 => Ok(Self::RecipeNature),
            17 => Ok(Self::RecipeItems),
            18 => Ok(Self::RecipeSearch),
            19 => Ok(Self::RecipeSearchBar),
            20 => Ok(Self::RecipeEquipment),
            21 => Ok(Self::RecipeBook),
            22 => Ok(Self::EnchantingInput),
            23 => Ok(Self::EnchantingMaterial),
            24 => Ok(Self::FurnaceFuel),
            25 => Ok(Self::FurnaceIngredient),
            26 => Ok(Self::FurnaceResult),
            27 => Ok(Self::HorseEquip),
            28 => Ok(Self::HotBar),
            29 => Ok(Self::Inventory),
            30 => Ok(Self::ShulkerBox),
            31 => Ok(Self::TradeIngredient1),
            32 => Ok(Self::TradeIngredient2),
            33 => Ok(Self::TradeResultPreview),
            34 => Ok(Self::Offhand),
            35 => Ok(Self::CompoundCreatorInput),
            36 => Ok(Self::CompoundCreatorOutputPreview),
            37 => Ok(Self::ElementConstructorOutputPreview),
            38 => Ok(Self::MaterialReducerInput),
            39 => Ok(Self::MaterialReducerOutput),
            40 => Ok(Self::LabTableInput),
            41 => Ok(Self::LoomInput),
            42 => Ok(Self::LoomDye),
            43 => Ok(Self::LoomMaterial),
            44 => Ok(Self::LoomResultPreview),
            45 => Ok(Self::BlastFurnaceIngredient),
            46 => Ok(Self::SmokerIngredient),
            47 => Ok(Self::Trade2Ingredient1),
            48 => Ok(Self::Trade2Ingredient2),
            49 => Ok(Self::Trade2ResultPreview),
            50 => Ok(Self::GrindstoneInput),
            51 => Ok(Self::GrindstoneAdditional),
            52 => Ok(Self::GrindstoneResultPreview),
            53 => Ok(Self::StonecutterInput),
            54 => Ok(Self::StonecutterResultPreview),
            55 => Ok(Self::CartographyInput),
            56 => Ok(Self::CartographyAdditional),
            57 => Ok(Self::CartographyResultPreview),
            58 => Ok(Self::Barrel),
            59 => Ok(Self::Cursor),
            60 => Ok(Self::CreatedOutput),
            61 => Ok(Self::SmithingTableTemplate),
            62 => Ok(Self::CrafterLevelEntity),
            63 => Ok(Self::Dynamic),
            64 => Ok(Self::RecipeFood),
            65 => Ok(Self::RecipeBlocks),
            66 => Ok(Self::RecipeFurnaceItems),
            _ => Err(Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid ContainerName ID: {value}"),
            )),
        }
    }
}

impl PacketWrite for ContainerName {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as u8).write(writer)?;
        Ok(())
    }
}

impl PacketRead for ContainerName {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let value = u8::read(buf)?;
        Self::try_from(value)
    }
}

#[derive(Debug, Clone)]
pub struct NetworkItemStack {
    pub id: VarInt,
    pub count: u16,
    pub aux_value: VarUInt,
    pub block_runtime_id: VarInt,
    pub extra_data: Vec<u8>,
}

impl PacketRead for NetworkItemStack {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let id = VarInt::read(buf)?;
        if id.0 == 0 {
            return Ok(Self {
                id,
                count: 0,
                aux_value: VarUInt(0),
                block_runtime_id: VarInt(0),
                extra_data: Vec::new(),
            });
        }
        let count = u16::read(buf)?;
        let aux_value = VarUInt::read(buf)?;
        let block_runtime_id = VarInt::read(buf)?;

        let extra_data_len = VarUInt::read(buf)?.0;
        let mut extra_data = vec![0u8; extra_data_len as usize];
        buf.read_exact(&mut extra_data)?;

        Ok(Self {
            id,
            count,
            aux_value,
            block_runtime_id,
            extra_data,
        })
    }
}
