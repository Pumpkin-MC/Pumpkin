use crate::plugin::persistence::{NamespacedKey, PersistentDataContainer, PersistentDataType};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;

pub trait NbtCompoundExt {
    fn to_pdc(&self) -> PersistentDataContainer;
}

impl NbtCompoundExt for NbtCompound {
    fn to_pdc(&self) -> PersistentDataContainer {
        let container = PersistentDataContainer::new();

        for (namespace, ns_tag) in &self.child_tags {
            if let NbtTag::Compound(ns_compound) = ns_tag {
                for (key, tag) in &ns_compound.child_tags {
                    if let Ok(ns_key) = NamespacedKey::new(namespace, key) {
                        let value = match tag {
                            NbtTag::Byte(b) => PersistentDataType::Bool(*b != 0),
                            NbtTag::Short(s) => PersistentDataType::I32(i32::from(*s)),
                            NbtTag::Int(i) => PersistentDataType::I32(*i),
                            NbtTag::Long(l) => PersistentDataType::I64(*l),
                            NbtTag::Float(f) => PersistentDataType::F32(*f),
                            NbtTag::Double(d) => PersistentDataType::F64(*d),
                            NbtTag::String(s) => PersistentDataType::String(s.clone()),
                            NbtTag::ByteArray(bytes) => PersistentDataType::Bytes(bytes.clone()),
                            NbtTag::List(list) => PersistentDataType::List(
                                list.iter()
                                    .filter_map(|t| match t {
                                        NbtTag::Int(i) => Some(PersistentDataType::I32(*i)),
                                        NbtTag::String(s) => {
                                            Some(PersistentDataType::String(s.clone()))
                                        }
                                        _ => None,
                                    })
                                    .collect(),
                            ),
                            _ => continue, // Unsupported tag
                        };
                        container.insert(ns_key, value);
                    }
                }
            }
        }
        container
    }
}

// Orphan rules suck hairy ass
#[must_use]
pub fn from_pdc(holder: &PersistentDataContainer) -> NbtCompound {
    let mut compound = NbtCompound::new();

    // Build Map from namespace to sub-compound
    let mut namespace_map: std::collections::HashMap<String, NbtCompound> =
        std::collections::HashMap::new();

    for entry in holder {
        let key = entry.key();
        let value = entry.value();

        let ns_compound = namespace_map.entry(key.namespace.clone()).or_default();

        let tag = match value {
            PersistentDataType::Bool(b) => NbtTag::Byte(i8::from(*b)),
            PersistentDataType::I32(i) => NbtTag::Int(*i),
            PersistentDataType::I64(l) => NbtTag::Long(*l),
            PersistentDataType::F32(f) => NbtTag::Float(*f),
            PersistentDataType::F64(d) => NbtTag::Double(*d),
            PersistentDataType::String(s) => NbtTag::String(s.clone()),
            PersistentDataType::Bytes(bytes) => NbtTag::ByteArray(bytes.clone()),
            PersistentDataType::List(list) => {
                let nbt_list = list
                    .iter()
                    .map(|elem| match elem {
                        PersistentDataType::I32(i) => NbtTag::Int(*i),
                        PersistentDataType::String(s) => NbtTag::String(s.clone()),
                        _ => unimplemented!(), // TODO: Add more
                    })
                    .collect();
                NbtTag::List(nbt_list)
            }
            _ => unimplemented!(), // TODO: Add more
        };

        ns_compound.put(&key.key, tag);
    }

    // Place all namespaced sub-compounds inside root compound
    for (namespace, ns_compound) in namespace_map {
        compound.put(&namespace, NbtTag::Compound(ns_compound));
    }

    compound
}
