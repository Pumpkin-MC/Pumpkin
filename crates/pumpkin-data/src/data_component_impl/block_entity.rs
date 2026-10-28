use super::Digest;
use crate::data_component_impl::DataComponentImpl;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;

#[derive(Clone, Debug, PartialEq)]
pub struct BlockEntityDataImpl {
    pub nbt: NbtCompound,
}
impl BlockEntityDataImpl {
    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        if let NbtTag::Compound(c) = tag {
            Some(Self { nbt: c.clone() })
        } else {
            None
        }
    }
}
impl DataComponentImpl for BlockEntityDataImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::Compound(self.nbt.clone())
    }
    default_impl!(BlockEntityData);
}

/// Entity NBT carried by an item, e.g. a spawn egg. `None` for the item's default component.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityDataImpl {
    pub nbt: Option<NbtCompound>,
}
impl EntityDataImpl {
    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        if let NbtTag::Compound(c) = tag {
            Some(Self {
                nbt: Some(c.clone()),
            })
        } else {
            None
        }
    }
}
impl DataComponentImpl for EntityDataImpl {
    fn write_data(&self) -> NbtTag {
        self.nbt.clone().map_or(NbtTag::End, NbtTag::Compound)
    }
    default_impl!(EntityData);
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct BucketEntityDataImpl {
    pub nbt: Option<NbtCompound>,
}
#[allow(non_upper_case_globals)]
pub const BucketEntityDataImpl: BucketEntityDataImpl = BucketEntityDataImpl { nbt: None };
impl BucketEntityDataImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        if let NbtTag::Compound(c) = data {
            Some(Self {
                nbt: Some(c.clone()),
            })
        } else {
            Some(Self { nbt: None })
        }
    }
}
impl DataComponentImpl for BucketEntityDataImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::Compound(self.nbt.clone().unwrap_or_default())
    }
    default_impl!(BucketEntityData);
}

#[derive(Clone)]
pub struct ContainerImpl {
    pub items: Vec<(u8, crate::item_stack::ItemStack)>,
}
impl PartialEq for ContainerImpl {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
impl Eq for ContainerImpl {}
impl std::fmt::Debug for ContainerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ContainerImpl")
    }
}
impl ContainerImpl {
    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        let mut items = Vec::new();
        if let NbtTag::List(l) = tag {
            for item_tag in l {
                if let NbtTag::Compound(c) = item_tag
                    && let Some(slot) = c.get_int("slot")
                    && let Some(item_compound) = c.get_compound("item")
                    && let Some(stack) =
                        crate::item_stack::ItemStack::read_item_stack(item_compound)
                {
                    items.push((slot as u8, stack));
                }
            }
        }
        Some(Self { items })
    }
}
impl DataComponentImpl for ContainerImpl {
    fn write_data(&self) -> NbtTag {
        let mut list = Vec::new();
        for (slot, stack) in &self.items {
            let mut entry = NbtCompound::new();
            entry.put_int("slot", *slot as i32);
            let mut item_compound = NbtCompound::new();
            stack.write_item_stack(&mut item_compound);
            entry.put_compound("item", item_compound);
            list.push(NbtTag::Compound(entry));
        }
        NbtTag::List(list)
    }
    default_impl!(Container);
}

use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct BlockStateImpl {
    pub properties: Cow<'static, [(Cow<'static, str>, Cow<'static, str>)]>,
}
impl PartialEq for BlockStateImpl {
    fn eq(&self, other: &Self) -> bool {
        let mut self_props = self.properties.to_vec();
        self_props.sort_by(|a, b| a.0.cmp(&b.0));
        let mut other_props = other.properties.to_vec();
        other_props.sort_by(|a, b| a.0.cmp(&b.0));
        self_props == other_props
    }
}
impl Eq for BlockStateImpl {}
impl std::hash::Hash for BlockStateImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut props = self.properties.to_vec();
        props.sort_by(|a, b| a.0.cmp(&b.0));
        for (k, v) in props {
            k.hash(state);
            v.hash(state);
        }
    }
}
impl BlockStateImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let mut properties = Vec::new();
        for (key, val) in compound.child_tags.iter() {
            if let Some(s) = val.extract_string() {
                properties.push((Cow::Owned(key.to_string()), Cow::Owned(s.to_string())));
            }
        }
        Some(Self {
            properties: Cow::Owned(properties),
        })
    }
}
impl DataComponentImpl for BlockStateImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        for (k, v) in self.properties.iter() {
            compound.put_string(k.as_ref(), v.to_string());
        }
        NbtTag::Compound(compound)
    }
    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new();
        let mut props = self.properties.to_vec();
        props.sort_by(|a, b| a.0.cmp(&b.0));
        for (k, v) in props {
            digest.update(k.as_ref().as_bytes());
            digest.update(v.as_ref().as_bytes());
        }
        digest.finalize() as i32
    }
    default_impl!(BlockState);
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct BeesImpl {
    pub bees: Vec<NbtTag>,
}
#[allow(non_upper_case_globals)]
pub const BeesImpl: BeesImpl = BeesImpl { bees: Vec::new() };
impl BeesImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let bees = if let NbtTag::List(list) = data {
            list.clone()
        } else {
            Vec::new()
        };
        Some(Self { bees })
    }
}
impl DataComponentImpl for BeesImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::List(self.bees.clone())
    }
    default_impl!(Bees);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ContainerLootImpl {
    pub loot_table: String,
    pub seed: i64,
}
impl ContainerLootImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let loot_table = compound.get_string("loot_table")?.to_string();
        let seed = compound.get_long("seed").unwrap_or(0);
        Some(Self { loot_table, seed })
    }
}
impl DataComponentImpl for ContainerLootImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_string("loot_table", self.loot_table.clone());
        compound.put_long("seed", self.seed);
        NbtTag::Compound(compound)
    }
    default_impl!(ContainerLoot);
}

#[derive(Clone)]
pub struct SulfurCubeContentImpl {
    pub item: crate::item_stack::ItemStack,
}
impl PartialEq for SulfurCubeContentImpl {
    fn eq(&self, other: &Self) -> bool {
        self.item.are_equal(&other.item)
    }
}
impl Eq for SulfurCubeContentImpl {}
impl std::fmt::Debug for SulfurCubeContentImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SulfurCubeContentImpl")
    }
}
impl SulfurCubeContentImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let item = crate::item_stack::ItemStack::read_item_stack(compound)?;
        Some(Self { item })
    }
}
impl DataComponentImpl for SulfurCubeContentImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        self.item.write_item_stack(&mut compound);
        NbtTag::Compound(compound)
    }
    default_impl!(SulfurCubeContent);
}

fn read_sign_messages(list: &[NbtTag]) -> [Cow<'static, str>; 4] {
    std::array::from_fn(|i| {
        list.get(i)
            .and_then(|tag| match tag {
                NbtTag::String(s) => Some(Cow::Owned(s.to_string())),
                NbtTag::Compound(c) => c
                    .get_string("text")
                    .or_else(|| c.get_string("translate"))
                    .map(|s| Cow::Owned(s.to_string())),
                _ => None,
            })
            .unwrap_or(Cow::Borrowed(""))
    })
}

fn write_sign_text(
    messages: &[Cow<'static, str>; 4],
    filtered_messages: Option<&[Cow<'static, str>; 4]>,
    color: crate::dye_color::DyeColor,
    has_glowing_text: bool,
) -> NbtTag {
    let mut compound = NbtCompound::new();
    compound.put_list(
        "messages",
        messages
            .iter()
            .map(|m| NbtTag::String(m.to_string().into()))
            .collect(),
    );
    if let Some(filtered) = filtered_messages {
        compound.put_list(
            "filtered_messages",
            filtered
                .iter()
                .map(|m| NbtTag::String(m.to_string().into()))
                .collect(),
        );
    }
    compound.put_string("color", color.name().to_string());
    compound.put_bool("has_glowing_text", has_glowing_text);
    NbtTag::Compound(compound)
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SignTextFrontImpl {
    pub messages: [Cow<'static, str>; 4],
    pub filtered_messages: Option<[Cow<'static, str>; 4]>,
    pub color: crate::dye_color::DyeColor,
    pub has_glowing_text: bool,
}
impl Default for SignTextFrontImpl {
    fn default() -> Self {
        Self::DEFAULT
    }
}
impl SignTextFrontImpl {
    pub const DEFAULT: Self = Self {
        messages: [
            Cow::Borrowed(""),
            Cow::Borrowed(""),
            Cow::Borrowed(""),
            Cow::Borrowed(""),
        ],
        filtered_messages: None,
        color: crate::dye_color::DyeColor::Black,
        has_glowing_text: false,
    };

    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let Some(compound) = data.extract_compound() else {
            return Some(Self::DEFAULT);
        };
        let messages = compound
            .get_list("messages")
            .map(|l| read_sign_messages(l))
            .unwrap_or(Self::DEFAULT.messages);
        let filtered_messages = compound
            .get_list("filtered_messages")
            .map(|l| read_sign_messages(l));
        let color = compound
            .get_string("color")
            .and_then(crate::dye_color::DyeColor::by_name)
            .unwrap_or(crate::dye_color::DyeColor::Black);
        let has_glowing_text = compound.get_bool("has_glowing_text").unwrap_or(false);
        Some(Self {
            messages,
            filtered_messages,
            color,
            has_glowing_text,
        })
    }
}
impl DataComponentImpl for SignTextFrontImpl {
    fn write_data(&self) -> NbtTag {
        write_sign_text(
            &self.messages,
            self.filtered_messages.as_ref(),
            self.color,
            self.has_glowing_text,
        )
    }
    default_impl!(SignTextFront);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SignTextBackImpl {
    pub messages: [Cow<'static, str>; 4],
    pub filtered_messages: Option<[Cow<'static, str>; 4]>,
    pub color: crate::dye_color::DyeColor,
    pub has_glowing_text: bool,
}
impl Default for SignTextBackImpl {
    fn default() -> Self {
        Self::DEFAULT
    }
}
impl SignTextBackImpl {
    pub const DEFAULT: Self = Self {
        messages: [
            Cow::Borrowed(""),
            Cow::Borrowed(""),
            Cow::Borrowed(""),
            Cow::Borrowed(""),
        ],
        filtered_messages: None,
        color: crate::dye_color::DyeColor::Black,
        has_glowing_text: false,
    };

    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let front = SignTextFrontImpl::read_data(data)?;
        Some(Self {
            messages: front.messages,
            filtered_messages: front.filtered_messages,
            color: front.color,
            has_glowing_text: front.has_glowing_text,
        })
    }
}
impl DataComponentImpl for SignTextBackImpl {
    fn write_data(&self) -> NbtTag {
        write_sign_text(
            &self.messages,
            self.filtered_messages.as_ref(),
            self.color,
            self.has_glowing_text,
        )
    }
    default_impl!(SignTextBack);
}
