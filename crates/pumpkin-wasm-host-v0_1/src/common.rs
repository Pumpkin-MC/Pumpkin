use crate::pumpkin;
pub use pumpkin::plugin::common::{
    NbtEntry as WitNbtEntry, NbtTag as WitNbtTag, NbtTree as WitNbtTree,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_wasm_host_common::state::PluginHostState;

impl pumpkin::plugin::common::Host for PluginHostState {}

pub fn push_wit_nbt_tag(tag: NbtTag, tags: &mut Vec<WitNbtTag>) -> u32 {
    let index = tags.len() as u32;
    tags.push(WitNbtTag::Byte(0));
    let tag = match tag {
        NbtTag::End => WitNbtTag::Compound(Vec::new()),
        NbtTag::Byte(value) => WitNbtTag::Byte(value),
        NbtTag::Short(value) => WitNbtTag::Short(value),
        NbtTag::Int(value) => WitNbtTag::Int(value),
        NbtTag::Long(value) => WitNbtTag::Long(value),
        NbtTag::Float(value) => WitNbtTag::Float(value),
        NbtTag::Double(value) => WitNbtTag::Double(value),
        NbtTag::ByteArray(value) => WitNbtTag::ByteArray(value.into_vec()),
        NbtTag::String(value) => WitNbtTag::StringTag(value.into()),
        NbtTag::List(value) => WitNbtTag::ListTag(
            value
                .into_iter()
                .map(|value| push_wit_nbt_tag(value, tags))
                .collect(),
        ),
        NbtTag::Compound(value) => WitNbtTag::Compound(
            value
                .child_tags
                .into_iter()
                .map(|(key, value)| WitNbtEntry {
                    key: key.into(),
                    value: push_wit_nbt_tag(value, tags),
                })
                .collect(),
        ),
        NbtTag::IntArray(value) => WitNbtTag::IntArray(value),
        NbtTag::LongArray(value) => WitNbtTag::LongArray(value),
    };
    tags[index as usize] = tag;
    index
}

#[must_use]
pub fn to_wit_nbt_tree(tag: NbtTag) -> WitNbtTree {
    let mut tags = Vec::new();
    let root = push_wit_nbt_tag(tag, &mut tags);
    WitNbtTree { root, tags }
}

pub fn from_wit_nbt_tree(tree: &WitNbtTree) -> Result<NbtTag, String> {
    fn read_tag(
        index: u32,
        tags: &[WitNbtTag],
        used: &mut [bool],
        depth: usize,
    ) -> Result<NbtTag, String> {
        let (Some(tag), Some(seen)) = (tags.get(index as usize), used.get_mut(index as usize))
        else {
            return Err(format!("NBT tag index {index} is out of bounds"));
        };
        // Each entry may be reached once; a repeat is a cycle or an amplifying shared node.
        if std::mem::replace(seen, true) {
            return Err(format!("NBT tag tree reaches index {index} more than once"));
        }
        if depth >= pumpkin_nbt::MAX_NBT_DEPTH {
            return Err(format!(
                "NBT tag tree is deeper than {}",
                pumpkin_nbt::MAX_NBT_DEPTH
            ));
        }
        let tag = match tag {
            WitNbtTag::Byte(value) => NbtTag::Byte(*value),
            WitNbtTag::Short(value) => NbtTag::Short(*value),
            WitNbtTag::Int(value) => NbtTag::Int(*value),
            WitNbtTag::Long(value) => NbtTag::Long(*value),
            WitNbtTag::Float(value) => NbtTag::Float(*value),
            WitNbtTag::Double(value) => NbtTag::Double(*value),
            WitNbtTag::ByteArray(value) => NbtTag::ByteArray(value.clone().into()),
            WitNbtTag::StringTag(value) => NbtTag::String(value.clone().into()),
            WitNbtTag::ListTag(value) => NbtTag::List(
                value
                    .iter()
                    .map(|value| read_tag(*value, tags, used, depth + 1))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            WitNbtTag::Compound(value) => NbtTag::Compound(NbtCompound {
                child_tags: value
                    .iter()
                    .map(|entry| {
                        read_tag(entry.value, tags, used, depth + 1)
                            .map(|value| (entry.key.clone().into(), value))
                    })
                    .collect::<Result<_, _>>()?,
            }),
            WitNbtTag::IntArray(value) => NbtTag::IntArray(value.clone()),
            WitNbtTag::LongArray(value) => NbtTag::LongArray(value.clone()),
        };
        Ok(tag)
    }

    read_tag(tree.root, &tree.tags, &mut vec![false; tree.tags.len()], 0)
}
