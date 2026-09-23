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
    /// Compares occupied slots and their item contents, independent of sparse entry order.
    fn eq(&self, other: &Self) -> bool {
        let mut left = [None; 256];
        let mut right = [None; 256];
        for (slot, stack) in &self.items {
            left[usize::from(*slot)] = (!stack.is_empty()).then_some(stack);
        }
        for (slot, stack) in &other.items {
            right[usize::from(*slot)] = (!stack.is_empty()).then_some(stack);
        }
        left.iter()
            .zip(right)
            .all(|(left, right)| match (left, right) {
                (Some(left), Some(right)) => left.are_equal(right),
                (None, None) => true,
                _ => false,
            })
    }
}
impl Eq for ContainerImpl {}
impl std::fmt::Debug for ContainerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ContainerImpl")
    }
}
impl ContainerImpl {
    /// Reads sparse slots, rejecting malformed entries and indices outside the 256-slot range.
    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        let NbtTag::List(list) = tag else {
            return None;
        };
        if list.len() > 256 {
            return None;
        }
        let mut items = Vec::new();
        for item_tag in list {
            let compound = item_tag.extract_compound()?;
            let slot = u8::try_from(compound.get_int("slot")?).ok()?;
            let stack =
                crate::item_stack::ItemStack::read_item_stack(compound.get_compound("item")?)?;
            // Repeated slots replace earlier values, matching the dense container representation.
            if let Some((_, previous)) = items.iter_mut().find(|(index, _)| *index == slot) {
                *previous = stack;
            } else {
                items.push((slot, stack));
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

/// An occupant's typed entity NBT and its elapsed and minimum hive residence times.
#[derive(Clone, Debug, PartialEq)]
pub struct BeeData {
    pub entity_data: NbtCompound,
    pub ticks_in_hive: i32,
    pub min_ticks_in_hive: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BeesImpl {
    pub bees: Vec<BeeData>,
}
impl BeesImpl {
    pub const EMPTY: Self = Self { bees: Vec::new() };

    /// Reads occupant data and timers, rejecting missing fields and unknown entity types.
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let NbtTag::List(list) = data else {
            return None;
        };
        let mut bees = Vec::new();
        for tag in list {
            let compound = tag.extract_compound()?;
            let entity_data = compound.get_compound("entity_data")?.clone();
            let entity_name = entity_data.get_string("id")?;
            crate::entity::EntityType::from_name(
                entity_name
                    .strip_prefix("minecraft:")
                    .unwrap_or(entity_name),
            )?;
            bees.push(BeeData {
                entity_data,
                ticks_in_hive: compound.get_int("ticks_in_hive")?,
                min_ticks_in_hive: compound.get_int("min_ticks_in_hive")?,
            });
        }
        Some(Self { bees })
    }
}
impl DataComponentImpl for BeesImpl {
    /// Persists each occupant's full entity data and both residence timers.
    fn write_data(&self) -> NbtTag {
        NbtTag::List(
            self.bees
                .iter()
                .map(|bee| {
                    let mut compound = NbtCompound::new();
                    compound.put_compound("entity_data", bee.entity_data.clone());
                    compound.put_int("ticks_in_hive", bee.ticks_in_hive);
                    compound.put_int("min_ticks_in_hive", bee.min_ticks_in_hive);
                    NbtTag::Compound(compound)
                })
                .collect(),
        )
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

#[cfg(test)]
mod copy_component_tests {
    use super::*;

    /// Hive occupants retain their entity payload and both timers through item persistence.
    #[test]
    fn bees_preserve_entity_and_timers() {
        let mut entity = NbtCompound::new();
        entity.put_string("id", "minecraft:bee".into());
        entity.put_bool("HasNectar", true);
        let mut bee = NbtCompound::new();
        bee.put_compound("entity_data", entity);
        bee.put_int("ticks_in_hive", 37);
        bee.put_int("min_ticks_in_hive", 2400);
        let expected = NbtTag::List(vec![NbtTag::Compound(bee)]);
        assert_eq!(
            BeesImpl::read_data(&expected).map(|value| value.write_data()),
            Some(expected)
        );
    }

    /// Persistent container indices outside 0 through 255 cannot wrap into a valid slot.
    #[test]
    fn container_rejects_out_of_range_persistent_slots() {
        for slot in [-1, 256] {
            let mut stack = NbtCompound::new();
            crate::item_stack::ItemStack::new(1, &crate::item::Item::DIAMOND)
                .write_item_stack(&mut stack);
            let mut entry = NbtCompound::new();
            entry.put_int("slot", slot);
            entry.put_compound("item", stack);
            assert!(
                ContainerImpl::read_data(&NbtTag::List(vec![NbtTag::Compound(entry)])).is_none()
            );
        }
    }
}
