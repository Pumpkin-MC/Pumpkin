use crate::VarInt;
use crate::codec::data_component::{DataComponentCodec, deserialize, serialize};
use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    CustomDataImpl, CustomNameImpl, DataComponentImpl, ItemNameImpl,
};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::text::TextComponent;
use pumpkin_util::version::JavaMinecraftVersion;
use std::borrow::Cow;
use std::io::Cursor;

#[derive(Clone)]
pub struct ItemStackSerializer<'a>(pub Cow<'a, ItemStack>);

/// Upper bound on the number of entries in a component patch. The readers reject
/// a longer patch, and the item stack writers refuse to emit one.
const MAX_COMPONENTS: i32 = 256;

fn item_component_counts(stack: &ItemStack) -> Result<(VarInt, VarInt), WritingError> {
    let mut to_add: usize = 0;
    let mut to_remove: usize = 0;

    for (_, data) in &stack.patch {
        if data.is_none() {
            to_remove += 1;
        } else {
            to_add += 1;
        }
    }

    let to_add = i32::try_from(to_add)
        .map_err(|_| WritingError::Message("Item stack patch is too large".into()))?;
    let to_remove = i32::try_from(to_remove)
        .map_err(|_| WritingError::Message("Item stack patch is too large".into()))?;

    let total = to_add.saturating_add(to_remove);
    if total > MAX_COMPONENTS {
        return Err(WritingError::Message(format!(
            "Item stack patch names {total} components, more than the {MAX_COMPONENTS} a reader accepts"
        )));
    }

    Ok((VarInt(to_add), VarInt(to_remove)))
}

fn serialize_item_stack_with_id(
    stack: &ItemStack,
    item_id: u16,
    version: JavaMinecraftVersion,
    write: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    if version >= JavaMinecraftVersion::V_1_20_5 {
        if stack.is_empty() {
            write.put_var_int(&VarInt(0))
        } else {
            let (to_add, to_remove) = item_component_counts(stack)?;
            write.put_var_int(&VarInt::from(stack.item_count))?;
            write.put_var_int(&VarInt::from(item_id))?;
            write.put_var_int(&to_add)?;
            write.put_var_int(&to_remove)?;

            for (id, data) in &stack.patch {
                if let Some(data) = data {
                    write.put_var_int(&VarInt(i32::from(id.to_id())))?;
                    serialize(*id, data.as_ref(), write)?;
                }
            }

            for (id, data) in &stack.patch {
                if data.is_none() {
                    write.put_var_int(&VarInt(i32::from(id.to_id())))?;
                }
            }

            Ok(())
        }
    } else if version >= JavaMinecraftVersion::V_1_13_2 {
        if stack.is_empty() {
            write.write_bool(false)
        } else {
            write.write_bool(true)?;
            write.put_var_int(&VarInt::from(item_id))?;
            write.write_i8(stack.item_count as i8)?;
            write.write_u8(0)?; // TAG_End (no NBT)
            Ok(())
        }
    } else if version >= JavaMinecraftVersion::V_1_13 {
        // 1.13 and 1.13.1: short id (-1 if empty), byte count, TAG_End
        if stack.is_empty() {
            write.write_i16_be(-1)
        } else {
            write.write_i16_be(item_id as i16)?;
            write.write_i8(stack.item_count as i8)?;
            write.write_u8(0)?; // TAG_End (no NBT)
            Ok(())
        }
    } else {
        // <= 1.12.2: short id (-1 if empty), byte count, short damage, TAG_End
        if stack.is_empty() {
            write.write_i16_be(-1)
        } else {
            write.write_i16_be(item_id as i16)?;
            write.write_i8(stack.item_count as i8)?;
            write.write_i16_be(0)?; // damage / metadata
            write.write_u8(0)?; // TAG_End (no NBT)
            Ok(())
        }
    }
}

fn serialize_length_prefixed_item_stack_with_id(
    stack: &ItemStack,
    item_id: u16,
    version: JavaMinecraftVersion,
    write: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    if version >= JavaMinecraftVersion::V_1_20_5 {
        if stack.is_empty() {
            write.put_var_int(&VarInt(0))
        } else {
            let (to_add, to_remove) = item_component_counts(stack)?;
            write.put_var_int(&VarInt::from(stack.item_count))?;
            write.put_var_int(&VarInt::from(item_id))?;
            write.put_var_int(&to_add)?;
            write.put_var_int(&to_remove)?;

            for (id, data) in &stack.patch {
                if let Some(data) = data {
                    write.put_var_int(&VarInt(i32::from(id.to_id())))?;
                    let mut comp_buf = Vec::new();
                    serialize(*id, data.as_ref(), &mut comp_buf)?;
                    write.put_var_int(&VarInt::from(comp_buf.len() as i32))?;
                    write.write_slice(&comp_buf)?;
                }
            }

            for (id, data) in &stack.patch {
                if data.is_none() {
                    write.put_var_int(&VarInt(i32::from(id.to_id())))?;
                }
            }

            Ok(())
        }
    } else {
        serialize_item_stack_with_id(stack, item_id, version, write)
    }
}

fn serialize_item_cost_with_id(
    stack: &ItemStack,
    item_id: u16,
    _version: JavaMinecraftVersion,
    write: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    let component_count = stack
        .patch
        .iter()
        .filter(|(_, data)| data.is_some())
        .count();
    let component_count = i32::try_from(component_count)
        .map_err(|_| WritingError::Message("Too many item cost components".into()))?;

    write.put_var_int(&VarInt::from(item_id))?;
    write.put_var_int(&VarInt::from(stack.item_count))?;
    write.put_var_int(&VarInt(component_count))?;
    for (id, data) in &stack.patch {
        if let Some(data) = data {
            write.put_var_int(&VarInt(i32::from(id.to_id())))?;
            serialize(*id, data.as_ref(), write)?;
        }
    }
    Ok(())
}

fn read_component_id(read: &mut impl NetworkReadExt) -> Result<DataComponent, ReadingError> {
    let id_val = read.get_var_int()?.0;
    let id_u8 = id_val
        .try_into()
        .map_err(|_| ReadingError::Message(format!("Invalid component ID: {id_val}")))?;
    DataComponent::try_from_id(id_u8)
        .ok_or_else(|| ReadingError::Message(format!("Unknown component ID: {id_val}")))
}

/// Collects a component patch as it is read, keeping the last entry for a
/// component id.
///
/// Vanilla holds a patch in a map keyed by component, so a repeated id
/// overwrites the earlier value and keeps the place it was first inserted at.
struct PatchBuilder(Vec<(DataComponent, Option<Box<dyn DataComponentImpl>>)>);

impl PatchBuilder {
    fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    fn push(&mut self, id: DataComponent, data: Option<Box<dyn DataComponentImpl>>) {
        if let Some((_, existing)) = self.0.iter_mut().find(|(seen, _)| *seen == id) {
            *existing = data;
        } else {
            self.0.push((id, data));
        }
    }

    fn finish(self) -> Vec<(DataComponent, Option<Box<dyn DataComponentImpl>>)> {
        self.0
    }
}

fn decode_custom_name(component_data: &[u8]) -> Result<Box<dyn DataComponentImpl>, ReadingError> {
    let mut cursor = Cursor::new(component_data);
    let mut nbt_reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(&mut cursor);
    let tag = NbtTag::deserialize(&mut nbt_reader)
        .map_err(|err| ReadingError::Message(format!("Failed to decode CustomName NBT: {err}")))?;
    let name = TextComponent::from_nbt(&tag);
    Ok(CustomNameImpl { name }.to_dyn())
}

fn decode_item_name(component_data: &[u8]) -> Result<Box<dyn DataComponentImpl>, ReadingError> {
    let mut cursor = Cursor::new(component_data);
    let mut nbt_reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(&mut cursor);
    let tag = NbtTag::deserialize(&mut nbt_reader)
        .map_err(|err| ReadingError::Message(format!("Failed to decode ItemName NBT: {err}")))?;
    let name = match tag {
        NbtTag::String(name) => name.to_string(),
        NbtTag::Compound(compound) => compound
            .get_string("translate")
            .or_else(|| compound.get_string("text"))
            .unwrap_or_default()
            .to_owned(),
        _ => String::new(),
    };
    Ok(ItemNameImpl {
        name: Cow::Owned(name),
    }
    .to_dyn())
}

fn decode_custom_data(component_data: &[u8]) -> Result<Box<dyn DataComponentImpl>, ReadingError> {
    let mut cursor = Cursor::new(component_data);
    let mut nbt_reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(&mut cursor);
    let tag = NbtTag::deserialize(&mut nbt_reader)
        .map_err(|err| ReadingError::Message(format!("Failed to decode CustomData NBT: {err}")))?;
    let data = match tag {
        NbtTag::Compound(compound) => compound,
        _ => pumpkin_nbt::compound::NbtCompound::new(),
    };
    Ok(CustomDataImpl::new(data).to_dyn())
}

fn decode_component(
    id: DataComponent,
    component_data: &[u8],
) -> Result<Box<dyn DataComponentImpl>, ReadingError> {
    match id {
        DataComponent::CustomName => decode_custom_name(component_data),
        DataComponent::ItemName => decode_item_name(component_data),
        DataComponent::CustomData => decode_custom_data(component_data),
        _ => {
            let mut cursor = Cursor::new(component_data);
            deserialize(id, &mut cursor)
        }
    }
}

fn read_length_prefixed_component(
    read: &mut impl NetworkReadExt,
) -> Result<(DataComponent, Box<dyn DataComponentImpl>), ReadingError> {
    let id = read_component_id(read)?;
    let byte_len = read.get_var_int()?.0;
    let byte_len: usize = byte_len
        .try_into()
        .map_err(|_| ReadingError::Message("Negative component data length".into()))?;
    if byte_len > crate::MAX_PACKET_DATA_SIZE {
        return Err(ReadingError::TooLarge("Component data too large".into()));
    }

    let component_impl = if byte_len <= 256 {
        let mut stack_buf = [0u8; 256];
        let slice = &mut stack_buf[..byte_len];
        read.read_bytes_to_buf(slice)?;
        decode_component(id, slice)?
    } else {
        let mut component_data = vec![0u8; byte_len];
        read.read_bytes_to_buf(&mut component_data)?;
        decode_component(id, &component_data)?
    };

    Ok((id, component_impl))
}

impl ItemStackSerializer<'_> {
    pub fn read(
        read: &mut impl NetworkReadExt,
    ) -> Result<ItemStackSerializer<'static>, ReadingError> {
        let item_count = read.get_var_int()?;
        if item_count.0 == 0 {
            return Ok(ItemStackSerializer(Cow::Borrowed(ItemStack::EMPTY)));
        }
        let item_count_u8: u8 = item_count
            .0
            .try_into()
            .map_err(|_| ReadingError::Message("Invalid item count!".into()))?;

        let item_id = read.get_var_int()?;
        let num_to_add = read.get_var_int()?.0;
        let num_to_remove = read.get_var_int()?.0;

        if num_to_add < 0 || num_to_remove < 0 {
            return Err(ReadingError::Message("Negative component count".into()));
        }

        let total_components = num_to_add
            .checked_add(num_to_remove)
            .ok_or_else(|| ReadingError::Message("Component count overflow".into()))?;

        if total_components > MAX_COMPONENTS {
            return Err(ReadingError::Message(
                "Too many components in ItemStack patch".into(),
            ));
        }

        let mut patch = PatchBuilder::with_capacity(total_components as usize);

        for _ in 0..num_to_add {
            let id = read_component_id(read)?;

            let component_impl = if id == DataComponent::CustomData {
                CustomDataImpl::deserialize(read)?.to_dyn()
            } else {
                deserialize(id, read)?
            };
            patch.push(id, Some(component_impl));
        }

        for _ in 0..num_to_remove {
            patch.push(read_component_id(read)?, None);
        }

        let item_id_u16: u16 = item_id
            .0
            .try_into()
            .map_err(|_| ReadingError::Message("Invalid item id!".into()))?;

        Ok(ItemStackSerializer(Cow::Owned(
            ItemStack::new_with_component(
                item_count_u8,
                Item::from_id(item_id_u16).unwrap_or(&Item::AIR),
                patch.finish(),
            ),
        )))
    }

    pub fn read_with_version(
        read: &mut impl NetworkReadExt,
        version: &JavaMinecraftVersion,
    ) -> Result<ItemStackSerializer<'static>, ReadingError> {
        if *version >= JavaMinecraftVersion::V_1_20_5 {
            let serializer = Self::read(read)?;
            if *version < JavaMinecraftVersion::V_26_3 {
                Ok(ItemStackSerializer(Cow::Owned(
                    serializer.to_stack_for_version(version),
                )))
            } else {
                Ok(serializer)
            }
        } else if *version >= JavaMinecraftVersion::V_1_13_2 {
            let present = read.get_bool()?;
            if !present {
                return Ok(ItemStackSerializer(Cow::Borrowed(ItemStack::EMPTY)));
            }
            let raw_item_id = read.get_var_int()?.0 as u16;
            let count = read.get_i8()? as u8;
            let nbt_type = read.get_u8()?;
            if nbt_type != 0 {
                // TAG_End is 0 when no NBT is present
            }
            let item = Item::from_id(raw_item_id).unwrap_or(&Item::AIR);
            Ok(ItemStackSerializer(Cow::Owned(ItemStack::new(count, item))))
        } else if *version >= JavaMinecraftVersion::V_1_13 {
            let raw_item_id = read.get_i16_be()?;
            if raw_item_id == -1 || raw_item_id < 0 {
                return Ok(ItemStackSerializer(Cow::Borrowed(ItemStack::EMPTY)));
            }
            let count = read.get_i8()? as u8;
            let nbt_type = read.get_u8()?;
            if nbt_type != 0 {
                // TAG_End is 0 when no NBT is present
            }
            let item = Item::from_id(raw_item_id as u16).unwrap_or(&Item::AIR);
            Ok(ItemStackSerializer(Cow::Owned(ItemStack::new(count, item))))
        } else {
            let raw_item_id = read.get_i16_be()?;
            if raw_item_id == -1 || raw_item_id < 0 {
                return Ok(ItemStackSerializer(Cow::Borrowed(ItemStack::EMPTY)));
            }
            let count = read.get_i8()? as u8;
            let _damage = read.get_i16_be()?;
            let nbt_type = read.get_u8()?;
            if nbt_type != 0 {
                // TAG_End is 0 when no NBT is present
            }
            let item = Item::from_id(raw_item_id as u16).unwrap_or(&Item::AIR);
            Ok(ItemStackSerializer(Cow::Owned(ItemStack::new(count, item))))
        }
    }

    pub fn read_untrusted_with_version(
        read: &mut impl NetworkReadExt,
        version: &JavaMinecraftVersion,
    ) -> Result<ItemStackSerializer<'static>, ReadingError> {
        if *version >= JavaMinecraftVersion::V_1_21_5 {
            Self::read_length_prefixed_optional(read)
        } else {
            Self::read_with_version(read, version)
        }
    }

    pub fn read_template_with_version(
        read: &mut impl NetworkReadExt,
        version: &JavaMinecraftVersion,
    ) -> Result<ItemStackSerializer<'static>, ReadingError> {
        if *version < JavaMinecraftVersion::V_26_1 {
            Self::read_with_version(read, version)
        } else {
            Self::read_template0(read, version)
        }
    }

    pub fn read_optional_template_with_version(
        read: &mut impl NetworkReadExt,
        version: &JavaMinecraftVersion,
    ) -> Result<ItemStackSerializer<'static>, ReadingError> {
        if *version < JavaMinecraftVersion::V_26_1 {
            Self::read_with_version(read, version)
        } else if read.get_bool()? {
            Self::read_template0(read, version)
        } else {
            Ok(ItemStackSerializer(Cow::Borrowed(ItemStack::EMPTY)))
        }
    }

    pub fn read_template0(
        read: &mut impl NetworkReadExt,
        _version: &JavaMinecraftVersion,
    ) -> Result<ItemStackSerializer<'static>, ReadingError> {
        let raw_item_id = read.get_var_int()?;
        let item_count = read.get_var_int()?;

        let item_id_u16: u16 = raw_item_id
            .0
            .try_into()
            .map_err(|_| ReadingError::Message("Invalid item id!".into()))?;
        let item = Item::from_id(item_id_u16).unwrap_or(&Item::AIR);

        let num_to_add = read.get_var_int()?.0;
        let num_to_remove = read.get_var_int()?.0;

        if num_to_add < 0 || num_to_remove < 0 {
            return Err(ReadingError::Message("Negative component count".into()));
        }

        let total_components = num_to_add
            .checked_add(num_to_remove)
            .ok_or_else(|| ReadingError::Message("Component count overflow".into()))?;

        if total_components > MAX_COMPONENTS {
            return Err(ReadingError::Message(
                "Too many components in ItemStack patch".into(),
            ));
        }

        let mut patch = PatchBuilder::with_capacity(total_components as usize);

        for _ in 0..num_to_add {
            let id = read_component_id(read)?;

            let component_impl = if id == DataComponent::CustomData {
                CustomDataImpl::deserialize(read)?.to_dyn()
            } else {
                deserialize(id, read)?
            };
            patch.push(id, Some(component_impl));
        }

        for _ in 0..num_to_remove {
            patch.push(read_component_id(read)?, None);
        }

        let item_count_u8: u8 = item_count
            .0
            .try_into()
            .map_err(|_| ReadingError::Message("Invalid item count!".into()))?;

        let stack = ItemStack::new_with_component(item_count_u8, item, patch.finish());
        if stack.is_empty() {
            return Err(ReadingError::Message(
                "Can't read empty item stack template".into(),
            ));
        }

        Ok(ItemStackSerializer(Cow::Owned(stack)))
    }

    pub fn write(&self, write: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        self.write_with_version(write, &JavaMinecraftVersion::V_26_3)
    }

    pub fn read_length_prefixed_optional(
        read: &mut impl NetworkReadExt,
    ) -> Result<ItemStackSerializer<'static>, ReadingError> {
        let item_count = read.get_var_int()?;
        if item_count.0 == 0 {
            return Ok(ItemStackSerializer(Cow::Borrowed(ItemStack::EMPTY)));
        }
        let item_count_u8 = item_count
            .0
            .try_into()
            .map_err(|_| ReadingError::Message("Invalid item count!".into()))?;

        let item_id = read.get_var_int()?;
        let num_to_add = read.get_var_int()?.0;
        let num_to_remove = read.get_var_int()?.0;

        if num_to_add < 0 || num_to_remove < 0 {
            return Err(ReadingError::Message("Negative component count".into()));
        }

        let total_components = num_to_add
            .checked_add(num_to_remove)
            .ok_or_else(|| ReadingError::Message("Component count overflow".into()))?;

        if total_components > MAX_COMPONENTS {
            return Err(ReadingError::Message(
                "Too many components in ItemStack patch".into(),
            ));
        }

        let mut patch = PatchBuilder::with_capacity(total_components as usize);

        for _ in 0..num_to_add {
            let (id, component_impl) = read_length_prefixed_component(read)?;
            patch.push(id, Some(component_impl));
        }

        for _ in 0..num_to_remove {
            patch.push(read_component_id(read)?, None);
        }

        let item_id_u16 = item_id
            .0
            .try_into()
            .map_err(|_| ReadingError::Message("Invalid item id!".into()))?;

        Ok(ItemStackSerializer(Cow::Owned(
            ItemStack::new_with_component(
                item_count_u8,
                Item::from_id(item_id_u16).unwrap_or(&Item::AIR),
                patch.finish(),
            ),
        )))
    }

    pub fn write_with_version(
        &self,
        write: &mut impl NetworkWriteExt,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        serialize_item_stack_with_id(self.0.as_ref(), self.0.item.id, *version, write)
    }

    pub fn write_length_prefixed_with_version(
        &self,
        write: &mut impl NetworkWriteExt,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        serialize_length_prefixed_item_stack_with_id(
            self.0.as_ref(),
            self.0.item.id,
            *version,
            write,
        )
    }

    pub fn write_item_cost_with_version(
        &self,
        write: &mut impl NetworkWriteExt,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        serialize_item_cost_with_id(self.0.as_ref(), self.0.item.id, *version, write)
    }

    pub fn write_untrusted_with_version(
        &self,
        write: &mut impl NetworkWriteExt,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version >= JavaMinecraftVersion::V_1_21_5 {
            self.write_length_prefixed_with_version(write, version)
        } else {
            self.write_with_version(write, version)
        }
    }

    pub fn write_template_with_version(
        &self,
        write: &mut impl NetworkWriteExt,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version < JavaMinecraftVersion::V_26_1 {
            self.write_with_version(write, version)
        } else {
            self.write_template0(write, version)
        }
    }

    pub fn write_optional_template_with_version(
        &self,
        write: &mut impl NetworkWriteExt,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version < JavaMinecraftVersion::V_26_1 {
            self.write_with_version(write, version)
        } else if !self.0.is_empty() {
            write.write_bool(true)?;
            self.write_template0(write, version)
        } else {
            write.write_bool(false)
        }
    }

    pub fn write_template0(
        &self,
        write: &mut impl NetworkWriteExt,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if self.0.is_empty() {
            return Err(WritingError::Message(
                "Can't write empty item stack template".into(),
            ));
        }
        let (to_add, to_remove) = item_component_counts(self.0.as_ref())?;
        write.put_var_int(&VarInt::from(self.0.item.id))?;
        write.put_var_int(&VarInt::from(self.0.item_count))?;
        write.put_var_int(&to_add)?;
        write.put_var_int(&to_remove)?;

        for (id, data) in &self.0.patch {
            if let Some(data) = data {
                write.put_var_int(&VarInt(i32::from(id.to_id())))?;
                serialize(*id, data.as_ref(), write)?;
            }
        }

        for (id, data) in &self.0.patch {
            if data.is_none() {
                write.put_var_int(&VarInt(i32::from(id.to_id())))?;
            }
        }

        Ok(())
    }

    #[must_use]
    pub fn to_stack(self) -> ItemStack {
        self.0.into_owned()
    }

    #[must_use]
    pub fn to_stack_for_version(self, _version: &JavaMinecraftVersion) -> ItemStack {
        self.0.into_owned()
    }
}

impl From<ItemStack> for ItemStackSerializer<'_> {
    fn from(item: ItemStack) -> Self {
        ItemStackSerializer(Cow::Owned(item))
    }
}

impl From<Option<ItemStack>> for ItemStackSerializer<'_> {
    fn from(item: Option<ItemStack>) -> Self {
        item.map_or_else(
            || ItemStackSerializer(Cow::Borrowed(ItemStack::EMPTY)),
            ItemStackSerializer::from,
        )
    }
}

#[derive(Debug, Clone)]
pub struct ItemComponentHash {
    pub added: Vec<(VarInt, i32)>,
    pub removed: Vec<VarInt>,
}

impl ItemComponentHash {
    pub fn read(read: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let added_length = read.get_var_int()?;
        if added_length.0 < 0 || added_length.0 > MAX_COMPONENTS {
            return Err(ReadingError::Message("added_length out of bounds".into()));
        }
        let mut added = Vec::with_capacity(added_length.0 as usize);
        for _ in 0..added_length.0 {
            let component_id = read.get_var_int()?;
            let component_value = read.get_i32()?;
            added.push((component_id, component_value));
        }

        let removed_length = read.get_var_int()?;
        if removed_length.0 < 0 || removed_length.0 > MAX_COMPONENTS {
            return Err(ReadingError::Message("removed_length out of bounds".into()));
        }
        let mut removed = Vec::with_capacity(removed_length.0 as usize);
        for _ in 0..removed_length.0 {
            let component_id = read.get_var_int()?;
            removed.push(component_id);
        }

        Ok(Self { added, removed })
    }

    pub fn write(&self, write: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        write.put_var_int(&VarInt::from(self.added.len() as i32))?;
        for (id, val) in &self.added {
            write.put_var_int(id)?;
            write.put_i32(*val)?;
        }
        write.put_var_int(&VarInt::from(self.removed.len() as i32))?;
        for id in &self.removed {
            write.put_var_int(id)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ItemStackHash {
    item_id: VarInt,
    count: VarInt,
    components: ItemComponentHash,
}

/// Claims the first unclaimed entry that satisfies `matches` and reports whether
/// there was one.
///
/// A patch can hold the same component id twice when server code builds it, so
/// entries are paired off one to one; without that a single hash entry stands in
/// for several patch entries and leaves its own counterpart unchecked.
fn claim_matching<T>(entries: &[T], claimed: &mut [bool], matches: impl Fn(&T) -> bool) -> bool {
    for (entry, claimed) in entries.iter().zip(claimed.iter_mut()) {
        if !*claimed && matches(entry) {
            *claimed = true;
            return true;
        }
    }
    false
}

#[derive(Debug, Clone)]
pub struct OptionalItemStackHash(pub Option<ItemStackHash>);

impl OptionalItemStackHash {
    pub fn read(read: &mut impl NetworkReadExt) -> Result<Self, ReadingError> {
        let is_some = read.get_bool()?;
        if is_some {
            let item_id = read.get_var_int()?;
            let count = read.get_var_int()?;
            let components = ItemComponentHash::read(read)?;

            Ok(Self(Some(ItemStackHash {
                item_id,
                count,
                components,
            })))
        } else {
            Ok(Self(None))
        }
    }

    pub fn write(&self, write: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        if let Some(hash) = &self.0 {
            write.put_bool(true)?;
            write.put_var_int(&hash.item_id)?;
            write.put_var_int(&hash.count)?;
            hash.components.write(write)?;
        } else {
            write.put_bool(false)?;
        }
        Ok(())
    }

    #[must_use]
    pub fn hash_equals(&self, other: &ItemStack) -> bool {
        if let Some(hash) = &self.0 {
            if hash.item_id != other.item.id.into() || hash.count != other.item_count.into() {
                return false;
            }
            let to_remove = other
                .patch
                .iter()
                .filter(|(_, data)| data.is_none())
                .count();
            let to_add = other.patch.len() - to_remove;
            if to_add != hash.components.added.len() || to_remove != hash.components.removed.len() {
                return false;
            }
            let mut added_claimed = vec![false; hash.components.added.len()];
            let mut removed_claimed = vec![false; hash.components.removed.len()];

            for (other_id, data) in &other.patch {
                let wire_id = VarInt::from(other_id.to_id());
                if let Some(data) = data {
                    let checksum = data.get_hash();
                    if !claim_matching(
                        &hash.components.added,
                        &mut added_claimed,
                        |(id, entry_hash)| *id == wire_id && *entry_hash == checksum,
                    ) {
                        return false;
                    }
                } else if !claim_matching(&hash.components.removed, &mut removed_claimed, |id| {
                    *id == wire_id
                }) {
                    return false;
                }
            }
            true
        } else {
            other.is_empty()
        }
    }
}

pub struct ItemStackTemplateSerializer<'a>(pub Cow<'a, ItemStack>);

impl ItemStackTemplateSerializer<'_> {
    pub fn write_with_version(
        &self,
        write: &mut impl NetworkWriteExt,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let serializer = ItemStackSerializer(Cow::Borrowed(self.0.as_ref()));
        serializer.write_template_with_version(write, version)
    }

    pub fn write(&self, write: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        self.write_with_version(write, &JavaMinecraftVersion::V_26_3)
    }
}

impl From<ItemStack> for ItemStackTemplateSerializer<'_> {
    fn from(item: ItemStack) -> Self {
        ItemStackTemplateSerializer(Cow::Owned(item))
    }
}

pub struct ItemStackOptionalTemplateSerializer<'a>(pub Cow<'a, ItemStack>);

impl ItemStackOptionalTemplateSerializer<'_> {
    pub fn write_with_version(
        &self,
        write: &mut impl NetworkWriteExt,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let serializer = ItemStackSerializer(Cow::Borrowed(self.0.as_ref()));
        serializer.write_optional_template_with_version(write, version)
    }

    pub fn write(&self, write: &mut impl NetworkWriteExt) -> Result<(), WritingError> {
        self.write_with_version(write, &JavaMinecraftVersion::V_26_3)
    }
}

impl From<ItemStack> for ItemStackOptionalTemplateSerializer<'_> {
    fn from(item: ItemStack) -> Self {
        ItemStackOptionalTemplateSerializer(Cow::Owned(item))
    }
}

impl From<Option<ItemStack>> for ItemStackOptionalTemplateSerializer<'_> {
    fn from(item: Option<ItemStack>) -> Self {
        item.map_or_else(
            || ItemStackOptionalTemplateSerializer(Cow::Borrowed(ItemStack::EMPTY)),
            ItemStackOptionalTemplateSerializer::from,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::data_component_impl::{
        DamageImpl, ItemModelImpl, MaxDamageImpl, MaxStackSizeImpl, RepairCostImpl, UnbreakableImpl,
    };

    /// The versions whose item stack encoding carries a component patch.
    const COMPONENT_VERSIONS: [JavaMinecraftVersion; 5] = [
        JavaMinecraftVersion::V_1_20_5,
        JavaMinecraftVersion::V_1_21_4,
        JavaMinecraftVersion::V_1_21_5,
        JavaMinecraftVersion::V_26_1,
        JavaMinecraftVersion::V_26_3,
    ];

    fn component_body(id: DataComponent, value: &dyn DataComponentImpl) -> Vec<u8> {
        let mut bytes = Vec::new();
        serialize(id, value, &mut bytes).unwrap();
        bytes
    }

    fn write_patch(bytes: &mut Vec<u8>, added: &[(i32, Vec<u8>)], removed: &[i32], prefixed: bool) {
        bytes.put_var_int(&VarInt(added.len() as i32)).unwrap();
        bytes.put_var_int(&VarInt(removed.len() as i32)).unwrap();
        for (id, body) in added {
            bytes.put_var_int(&VarInt(*id)).unwrap();
            if prefixed {
                bytes.put_var_int(&VarInt(body.len() as i32)).unwrap();
            }
            bytes.write_slice(body).unwrap();
        }
        for id in removed {
            bytes.put_var_int(&VarInt(*id)).unwrap();
        }
    }

    /// Builds the 1.20.5+ encoding from raw wire values, so a test can use counts
    /// and ids the writer would never produce.
    fn stack_bytes(count: i32, item_id: u16, added: &[(i32, Vec<u8>)], removed: &[i32]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.put_var_int(&VarInt(count)).unwrap();
        bytes.put_var_int(&VarInt::from(item_id)).unwrap();
        write_patch(&mut bytes, added, removed, false);
        bytes
    }

    /// The 1.21.5+ shape, where every added component carries its own length.
    fn length_prefixed_stack_bytes(
        count: i32,
        item_id: u16,
        added: &[(i32, Vec<u8>)],
        removed: &[i32],
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.put_var_int(&VarInt(count)).unwrap();
        bytes.put_var_int(&VarInt::from(item_id)).unwrap();
        write_patch(&mut bytes, added, removed, true);
        bytes
    }

    /// The 26.1+ template shape, which leads with the item id instead of the count.
    fn template_bytes(
        item_id: u16,
        count: i32,
        added: &[(i32, Vec<u8>)],
        removed: &[i32],
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.put_var_int(&VarInt::from(item_id)).unwrap();
        bytes.put_var_int(&VarInt(count)).unwrap();
        write_patch(&mut bytes, added, removed, false);
        bytes
    }

    fn sword_with_ten_components() -> ItemStack {
        ItemStack::new_with_component(
            7,
            &Item::DIAMOND_SWORD,
            vec![
                (
                    DataComponent::MaxStackSize,
                    Some(MaxStackSizeImpl { size: 1 }.to_dyn()),
                ),
                (
                    DataComponent::Damage,
                    Some(DamageImpl { damage: 42 }.to_dyn()),
                ),
                (
                    DataComponent::RepairCost,
                    Some(RepairCostImpl { cost: 3 }.to_dyn()),
                ),
                (DataComponent::Unbreakable, Some(UnbreakableImpl.to_dyn())),
                (
                    DataComponent::ItemModel,
                    Some(
                        ItemModelImpl {
                            id: Cow::Borrowed("minecraft:diamond_sword"),
                        }
                        .to_dyn(),
                    ),
                ),
                (DataComponent::Lore, None),
                (DataComponent::Rarity, None),
                (DataComponent::Enchantments, None),
                (DataComponent::CustomName, None),
                (DataComponent::MaxDamage, None),
            ],
        )
    }

    #[test]
    fn a_patch_round_trips_through_every_item_stack_encoding() {
        let serializer = ItemStackSerializer::from(sword_with_ten_components());

        for version in COMPONENT_VERSIONS {
            let mut written = Vec::new();
            serializer
                .write_with_version(&mut written, &version)
                .unwrap();
            let mut rest = written.as_slice();
            let back = ItemStackSerializer::read_with_version(&mut rest, &version).unwrap();
            assert!(rest.is_empty(), "{version}");
            assert_eq!(back.0.patch.len(), 10, "{version}");
            assert_eq!(back.0.item_count, 7, "{version}");
            let mut rewritten = Vec::new();
            back.write_with_version(&mut rewritten, &version).unwrap();
            assert_eq!(rewritten, written, "{version}");

            let mut written = Vec::new();
            serializer
                .write_untrusted_with_version(&mut written, &version)
                .unwrap();
            let mut rest = written.as_slice();
            let back =
                ItemStackSerializer::read_untrusted_with_version(&mut rest, &version).unwrap();
            assert!(rest.is_empty(), "{version}");
            assert_eq!(back.0.patch.len(), 10, "{version}");
            let mut rewritten = Vec::new();
            back.write_untrusted_with_version(&mut rewritten, &version)
                .unwrap();
            assert_eq!(rewritten, written, "{version}");

            let mut written = Vec::new();
            serializer
                .write_template_with_version(&mut written, &version)
                .unwrap();
            let mut rest = written.as_slice();
            let back =
                ItemStackSerializer::read_template_with_version(&mut rest, &version).unwrap();
            assert!(rest.is_empty(), "{version}");
            assert_eq!(back.0.patch.len(), 10, "{version}");
            let mut rewritten = Vec::new();
            back.write_template_with_version(&mut rewritten, &version)
                .unwrap();
            assert_eq!(rewritten, written, "{version}");
        }
    }

    /// The lowest byte value the component enum does not use. It moves as the
    /// generated enum grows, so no test may spell it out.
    fn unknown_component_id() -> i32 {
        let id = (0..=255u8)
            .find(|id| DataComponent::try_from_id(*id).is_none())
            .expect("the component enum cannot cover every byte");
        i32::from(id)
    }

    fn assert_component_id_is_rejected(wire_id: i32) {
        let body = component_body(
            DataComponent::MaxStackSize,
            MaxStackSizeImpl { size: 64 }.to_dyn().as_ref(),
        );

        let bytes = stack_bytes(1, Item::DIAMOND_SWORD.id, &[(wire_id, body.clone())], &[]);
        assert!(
            ItemStackSerializer::read(&mut bytes.as_slice()).is_err(),
            "component id {wire_id} must not decode"
        );

        let bytes = template_bytes(Item::DIAMOND_SWORD.id, 1, &[(wire_id, body)], &[]);
        assert!(
            ItemStackSerializer::read_template0(
                &mut bytes.as_slice(),
                &JavaMinecraftVersion::V_26_3
            )
            .is_err(),
            "component id {wire_id} must not decode as a template"
        );
    }

    #[test]
    fn a_component_id_outside_a_byte_is_rejected() {
        let body = component_body(
            DataComponent::MaxStackSize,
            MaxStackSizeImpl { size: 64 }.to_dyn().as_ref(),
        );

        // 1 is MaxStackSize, which is what 257 becomes once truncated to a byte
        let sane = stack_bytes(1, Item::DIAMOND_SWORD.id, &[(1, body)], &[]);
        let stack = ItemStackSerializer::read(&mut sane.as_slice())
            .unwrap()
            .to_stack();
        assert_eq!(stack.patch.len(), 1);
        assert!(stack.patch[0].0 == DataComponent::MaxStackSize);

        for wire_id in [-1, 256, 257, 100_000] {
            assert_component_id_is_rejected(wire_id);
        }
    }

    #[test]
    fn a_component_id_that_no_component_uses_is_rejected() {
        assert_component_id_is_rejected(unknown_component_id());
    }

    #[test]
    fn an_item_count_that_does_not_fit_a_byte_is_rejected() {
        for count in [-1, 256, 257, 100_000] {
            let bytes = stack_bytes(count, Item::DIAMOND_SWORD.id, &[], &[]);
            assert!(
                ItemStackSerializer::read(&mut bytes.as_slice()).is_err(),
                "item count {count} must not decode"
            );
        }

        let bytes = stack_bytes(255, Item::DIAMOND_SWORD.id, &[], &[]);
        let stack = ItemStackSerializer::read(&mut bytes.as_slice())
            .unwrap()
            .to_stack();
        assert_eq!(stack.item_count, 255);
    }

    #[test]
    fn a_patch_naming_a_component_twice_keeps_the_last_entry() {
        let damage = i32::from(DataComponent::Damage.to_id());
        let first = component_body(
            DataComponent::Damage,
            DamageImpl { damage: 42 }.to_dyn().as_ref(),
        );
        let last = component_body(
            DataComponent::Damage,
            DamageImpl { damage: 7 }.to_dyn().as_ref(),
        );
        let sword = Item::DIAMOND_SWORD.id;
        let added = [(damage, first), (damage, last.clone())];

        let assert_last_wins = |stack: &ItemStack| {
            assert_eq!(stack.patch.len(), 1);
            assert_eq!(
                stack
                    .get_data_component::<DamageImpl>()
                    .map(|damage| damage.damage),
                Some(7)
            );
        };

        let bytes = stack_bytes(1, sword, &added, &[]);
        assert_last_wins(
            &ItemStackSerializer::read(&mut bytes.as_slice())
                .unwrap()
                .to_stack(),
        );

        let bytes = template_bytes(sword, 1, &added, &[]);
        assert_last_wins(
            &ItemStackSerializer::read_template0(
                &mut bytes.as_slice(),
                &JavaMinecraftVersion::V_26_3,
            )
            .unwrap()
            .to_stack(),
        );

        // the creative slot packet decodes through the length prefixed reader
        let bytes = length_prefixed_stack_bytes(1, sword, &added, &[]);
        assert_last_wins(
            &ItemStackSerializer::read_length_prefixed_optional(&mut bytes.as_slice())
                .unwrap()
                .to_stack(),
        );

        // an id named in both lists ends up as the removal, which comes last
        let bytes = stack_bytes(1, sword, &[(damage, last)], &[damage]);
        let stack = ItemStackSerializer::read(&mut bytes.as_slice())
            .unwrap()
            .to_stack();
        assert_eq!(stack.patch.len(), 1);
        assert!(stack.patch[0].1.is_none());
    }

    #[test]
    fn a_patch_above_the_component_count_limit_is_refused_instead_of_wrapping() {
        let oversized: Vec<_> = (0..300)
            .map(|_| {
                (
                    DataComponent::Damage,
                    Some(DamageImpl { damage: 1 }.to_dyn()),
                )
            })
            .collect();
        let serializer = ItemStackSerializer::from(ItemStack::new_with_component(
            1,
            &Item::DIAMOND_SWORD,
            oversized,
        ));

        for version in COMPONENT_VERSIONS {
            let mut bytes = Vec::new();
            assert!(
                serializer.write_with_version(&mut bytes, &version).is_err(),
                "{version}"
            );

            let mut bytes = Vec::new();
            assert!(
                serializer
                    .write_untrusted_with_version(&mut bytes, &version)
                    .is_err(),
                "{version}"
            );

            let mut bytes = Vec::new();
            assert!(
                serializer
                    .write_template_with_version(&mut bytes, &version)
                    .is_err(),
                "{version}"
            );
        }
    }

    #[test]
    fn a_hash_entry_stands_in_for_at_most_one_patch_entry() {
        let damage = DamageImpl { damage: 1 };
        let damage_id = VarInt::from(DataComponent::Damage.to_id());
        let max_damage_id = VarInt::from(DataComponent::MaxDamage.to_id());

        let hash_of = |added: Vec<(VarInt, i32)>, removed: Vec<VarInt>| {
            OptionalItemStackHash(Some(ItemStackHash {
                item_id: VarInt::from(Item::DIAMOND_SWORD.id),
                count: VarInt(1),
                components: ItemComponentHash { added, removed },
            }))
        };
        let stack_of = |patch| ItemStack::new_with_component(1, &Item::DIAMOND_SWORD, patch);

        let added_twice = stack_of(vec![
            (DataComponent::Damage, Some(damage.clone().to_dyn())),
            (DataComponent::Damage, Some(damage.clone().to_dyn())),
        ]);
        assert!(
            !hash_of(
                vec![
                    (damage_id, damage.get_hash()),
                    (max_damage_id, MaxDamageImpl { max_damage: 9 }.get_hash()),
                ],
                Vec::new(),
            )
            .hash_equals(&added_twice)
        );
        assert!(
            hash_of(
                vec![
                    (damage_id, damage.get_hash()),
                    (damage_id, damage.get_hash())
                ],
                Vec::new(),
            )
            .hash_equals(&added_twice)
        );

        let removed_twice = stack_of(vec![
            (DataComponent::Damage, None),
            (DataComponent::Damage, None),
        ]);
        assert!(!hash_of(Vec::new(), vec![damage_id, max_damage_id]).hash_equals(&removed_twice));
        assert!(hash_of(Vec::new(), vec![damage_id, damage_id]).hash_equals(&removed_twice));
    }

    #[test]
    fn a_hash_naming_another_component_does_not_match() {
        let component = DamageImpl { damage: 1 };
        let stack = ItemStack::new_with_component(
            1,
            &Item::DIAMOND_SWORD,
            vec![(DataComponent::Damage, Some(component.clone().to_dyn()))],
        );

        let hash = OptionalItemStackHash(Some(ItemStackHash {
            item_id: VarInt::from(Item::DIAMOND_SWORD.id),
            count: VarInt(1),
            components: ItemComponentHash {
                added: vec![(
                    VarInt::from(DataComponent::MaxDamage.to_id()),
                    component.get_hash(),
                )],
                removed: Vec::new(),
            },
        }));

        assert!(!hash.hash_equals(&stack));
    }

    #[test]
    fn a_patch_longer_than_a_byte_does_not_match_a_shorter_hash() {
        let component = DamageImpl { damage: 1 };
        let patch: Vec<_> = (0..257)
            .map(|_| (DataComponent::Damage, Some(component.clone().to_dyn())))
            .collect();
        let stack = ItemStack::new_with_component(1, &Item::DIAMOND_SWORD, patch);

        let hash = OptionalItemStackHash(Some(ItemStackHash {
            item_id: VarInt::from(Item::DIAMOND_SWORD.id),
            count: VarInt(1),
            components: ItemComponentHash {
                added: vec![(
                    VarInt::from(DataComponent::Damage.to_id()),
                    component.get_hash(),
                )],
                removed: Vec::new(),
            },
        }));

        assert!(!hash.hash_equals(&stack));
    }

    // The writer bounds the number of entries, not their distinctness.
    #[test]
    fn a_patch_at_the_component_count_limit_still_writes() {
        let at_limit: Vec<_> = (0..MAX_COMPONENTS)
            .map(|_| {
                (
                    DataComponent::Damage,
                    Some(DamageImpl { damage: 1 }.to_dyn()),
                )
            })
            .collect();
        let serializer = ItemStackSerializer::from(ItemStack::new_with_component(
            1,
            &Item::DIAMOND_SWORD,
            at_limit,
        ));

        let mut bytes = Vec::new();
        serializer
            .write_with_version(&mut bytes, &JavaMinecraftVersion::V_26_3)
            .unwrap();

        let stack = ItemStackSerializer::read(&mut bytes.as_slice())
            .unwrap()
            .to_stack();
        assert_eq!(stack.patch.len(), 1);
    }
}
