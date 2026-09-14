use crate::data_component_impl::DataComponentImpl;
use crc_fast::CrcAlgorithm::Crc32Iscsi;
use crc_fast::Digest;
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EntityDataImpl;
impl EntityDataImpl {
    pub const fn read_data(_data: &NbtTag) -> Option<Self> {
        Some(Self)
    }
}
impl DataComponentImpl for EntityDataImpl {
    default_impl!(EntityData);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BucketEntityDataImpl;
impl BucketEntityDataImpl {
    pub const fn read_data(_data: &NbtTag) -> Option<Self> {
        Some(Self)
    }
}
impl DataComponentImpl for BucketEntityDataImpl {
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
        let mut digest = Digest::new(Crc32Iscsi);
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BeesImpl;
impl BeesImpl {
    pub const fn read_data(_data: &NbtTag) -> Option<Self> {
        Some(Self)
    }
}
impl DataComponentImpl for BeesImpl {
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SulfurCubeContentImpl;
impl SulfurCubeContentImpl {
    pub const fn read_data(_data: &NbtTag) -> Option<Self> {
        Some(Self)
    }
}
impl DataComponentImpl for SulfurCubeContentImpl {
    default_impl!(SulfurCubeContent);
}
