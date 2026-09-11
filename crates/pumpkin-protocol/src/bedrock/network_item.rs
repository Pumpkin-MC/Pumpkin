use std::io::{Error, ErrorKind, Read, Write};
use std::num::NonZero;

use pumpkin_data::item::{BedrockItem, JavaToBedrockItemMapping};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::{Nbt, deserializer::NbtReadHelperBedrock};

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
        (self.id.0 as i16).write(writer)?;
        self.write_stack_data(writer, None)
    }
}

impl PacketRead for NetworkItemDescriptor {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let id = VarInt(i32::from(i16::read(buf)?));
        let stack_size = u16::read(buf)?;
        let aux_value = VarUInt::read(buf)?;

        let has_net_id = bool::read(buf)?;
        if has_net_id {
            let _net_id = VarInt::read(buf)?;
        }

        let block_runtime_id = VarInt(VarUInt::read(buf)?.0 as i32);

        let user_data_len = VarUInt::read(buf)?.0 as usize;
        if user_data_len > 1_048_576 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "user_data_len exceeds 1MB limit",
            ));
        }
        let mut user_data = vec![0u8; user_data_len];
        buf.read_exact(&mut user_data)?;

        let (nbt_data, place_on_blocks, destroy_blocks, shield_blocking_tick) =
            read_user_data(user_data, id.0 == i32::from(BedrockItem::SHIELD.id))?;

        Ok(Self {
            id,
            stack_size,
            aux_value,
            block_runtime_id,
            nbt_data,
            place_on_blocks,
            destroy_blocks,
            shield_blocking_tick,
        })
    }
}

impl NetworkItemDescriptor {
    fn write_stack_data<W: Write>(
        &self,
        writer: &mut W,
        net_id: Option<VarInt>,
    ) -> Result<(), Error> {
        self.stack_size.write(writer)?;
        self.aux_value.write(writer)?;
        net_id.is_some().write(writer)?;
        if let Some(net_id) = net_id {
            net_id.write(writer)?;
        }
        VarUInt(self.block_runtime_id.0 as u32).write(writer)?;
        self.write_user_data(writer)?;
        Ok(())
    }

    /// Writes the 1.26.40 `NetworkItemInstanceDescriptor` used by creative and
    /// crafting packets. Regular item fields use the stack descriptor instead.
    pub fn write_item_instance<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.id.write(writer)?;
        self.stack_size.write(writer)?;
        self.aux_value.write(writer)?;
        self.block_runtime_id.write(writer)?;
        self.write_user_data(writer)
    }

    fn write_user_data<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        if self.id.0 == 0 {
            return VarUInt(0).write(writer);
        }

        let mut buf = Vec::new();
        if self.nbt_data.is_empty() {
            (0i16).write(&mut buf)?;
        } else {
            (-1i16).write(&mut buf)?;
            (1i8).write(&mut buf)?;
            self.nbt_data.clone().write_to_writer_bedrock(&mut buf)?;
        }
        write_user_data_strings(&mut buf, &self.place_on_blocks)?;
        write_user_data_strings(&mut buf, &self.destroy_blocks)?;
        if self.id.0 == i32::from(BedrockItem::SHIELD.id) {
            self.shield_blocking_tick.write(&mut buf)?;
        }
        VarUInt(buf.len() as u32).write(writer)?;
        writer.write_all(&buf)
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
    pub net_id: Option<NonZero<i32>>,
}

impl PacketWrite for ItemStackWrapper {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.id.write(writer)?;
        self.stack_size.write(writer)?;
        self.aux_value.write(writer)?;
        self.net_id.is_some().write(writer)?;
        if let Some(id) = self.net_id {
            VarInt(id.get()).write(writer)?;
        }
        VarUInt(self.block_runtime_id.0 as u32).write(writer)?;

        let descriptor = NetworkItemDescriptor {
            id: VarInt(i32::from(self.id)),
            stack_size: self.stack_size,
            aux_value: self.aux_value,
            block_runtime_id: self.block_runtime_id,
            nbt_data: self.nbt_data.clone(),
            place_on_blocks: self.place_on_blocks.clone(),
            destroy_blocks: self.destroy_blocks.clone(),
            shield_blocking_tick: self.shield_blocking_tick,
        };
        descriptor.write_user_data(writer)
    }
}

impl PacketRead for ItemStackWrapper {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let id = i16::read(buf)?;
        let stack_size = u16::read(buf)?;
        let aux_value = VarUInt::read(buf)?;

        let has_net_id = bool::read(buf)?;
        let net_id = if has_net_id {
            let stack_id = VarInt::read(buf)?;
            NonZero::new(stack_id.0)
        } else {
            None
        };

        let block_runtime_id = VarInt(VarUInt::read(buf)?.0 as i32);

        let user_data_len = VarUInt::read(buf)?.0 as usize;
        if user_data_len > 1_048_576 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "user_data_len exceeds 1MB limit",
            ));
        }
        let mut user_data = vec![0u8; user_data_len];
        buf.read_exact(&mut user_data)?;

        let (nbt_data, place_on_blocks, destroy_blocks, shield_blocking_tick) =
            read_user_data(user_data, id == BedrockItem::SHIELD.id)?;

        Ok(Self {
            id,
            stack_size,
            aux_value,
            block_runtime_id,
            nbt_data,
            place_on_blocks,
            destroy_blocks,
            shield_blocking_tick,
            net_id,
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
    pub net_id: Option<NonZero<i32>>,
}

impl PacketWrite for NetworkItemStackDescriptor {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.id.write(writer)?;

        self.stack_size.write(writer)?;
        self.aux_value.write(writer)?;

        self.net_id.is_some().write(writer)?;
        if let Some(id) = self.net_id {
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
        let net_id = if has_net_id {
            let stack_id = VarInt::read(buf)?;
            NonZero::new(stack_id.0)
        } else {
            None
        };

        let block_runtime_id = VarUInt::read(buf)?;

        let extra_data_len = VarUInt::read(buf)?.0 as usize;
        if extra_data_len > 1_048_576 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "extra_data_len exceeds 1MB limit",
            ));
        }
        let mut extra_data = vec![0u8; extra_data_len];
        buf.read_exact(&mut extra_data)?;

        Ok(Self {
            id,
            stack_size,
            aux_value,
            block_runtime_id,
            extra_data,
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
                    // Empty NBT followed by empty can-place and can-destroy lists.
                    let extra_data = vec![0; 10];

                    Self {
                        id: mapping.bedrock_item.id,
                        stack_size: stack.item_count as u16,
                        aux_value: VarUInt(mapping.bedrock_data),
                        block_runtime_id: VarUInt(mapping.bedrock_block_state as u32),
                        extra_data,
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
        let count = u16::read(buf)?;
        let aux_value = VarUInt::read(buf)?;
        let block_runtime_id = VarInt::read(buf)?;

        let extra_data_len = VarUInt::read(buf)?.0 as usize;
        if extra_data_len > 1_048_576 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "extra_data_len exceeds 1MB limit",
            ));
        }
        let mut extra_data = vec![0u8; extra_data_len];
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

fn write_user_data_strings<W: Write>(writer: &mut W, values: &[String]) -> Result<(), Error> {
    (values.len() as i32).write(writer)?;
    for value in values {
        let bytes = value.as_bytes();
        let len = u16::try_from(bytes.len())
            .map_err(|_| Error::new(std::io::ErrorKind::InvalidInput, "item string too long"))?;
        writer.write_all(&len.to_be_bytes())?;
        writer.write_all(bytes)?;
    }
    Ok(())
}

fn read_user_data_strings<R: Read>(reader: &mut R) -> Result<Vec<String>, Error> {
    let len = i32::read(reader)?;
    if !(0..=1024).contains(&len) {
        return Err(Error::new(
            std::io::ErrorKind::InvalidData,
            "item string array length out of bounds",
        ));
    }
    let mut values = Vec::with_capacity((len as usize).min(32));
    for _ in 0..len {
        let mut length = [0; 2];
        reader.read_exact(&mut length)?;
        let str_len = usize::from(u16::from_be_bytes(length));
        if str_len > 32767 {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "item string too long",
            ));
        }
        let mut bytes = vec![0; str_len];
        reader.read_exact(&mut bytes)?;
        values.push(
            String::from_utf8(bytes)
                .map_err(|error| Error::new(std::io::ErrorKind::InvalidData, error))?,
        );
    }
    Ok(values)
}

fn read_user_data(
    user_data: Vec<u8>,
    is_shield: bool,
) -> Result<(Nbt, Vec<String>, Vec<String>, i64), Error> {
    if user_data.is_empty() {
        return Ok((Nbt::default(), Vec::new(), Vec::new(), 0));
    }

    let mut cursor = std::io::Cursor::new(user_data);
    let nbt_version = i16::read(&mut cursor)?;
    let nbt_data = if nbt_version == -1 {
        let _version = i8::read(&mut cursor)?;
        let mut nbt_reader = NbtReadHelperBedrock::new(&mut cursor);
        Nbt::read(&mut nbt_reader)
            .map_err(|error| Error::new(std::io::ErrorKind::InvalidData, error))?
    } else {
        Nbt::default()
    };
    let place_on_blocks = read_user_data_strings(&mut cursor)?;
    let destroy_blocks = read_user_data_strings(&mut cursor)?;
    let shield_blocking_tick = if is_shield {
        i64::read(&mut cursor)?
    } else {
        0
    };
    Ok((
        nbt_data,
        place_on_blocks,
        destroy_blocks,
        shield_blocking_tick,
    ))
}
