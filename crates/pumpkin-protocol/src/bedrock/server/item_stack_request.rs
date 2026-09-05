use std::io::{Error, ErrorKind, Read};

use crate::{
    bedrock::network_item::FullContainerName,
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::{PacketRead, PacketWrite},
};
use pumpkin_macros::packet;

const MAX_COLLECTION_LENGTH: u32 = 1024;

fn collection_length<R: Read>(reader: &mut R, name: &str) -> Result<usize, Error> {
    let len = VarUInt::read(reader)?.0;
    if len > MAX_COLLECTION_LENGTH {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("{name} length {len} exceeds {MAX_COLLECTION_LENGTH}"),
        ));
    }
    Ok(len as usize)
}

#[derive(Debug, PacketRead, PacketWrite)]
pub struct ItemStackRequestSlotInfo {
    pub container_name: FullContainerName,
    pub slot_id: u8,
    pub stack_id: i32,
}

#[derive(Debug)]
pub struct StackRequestItem {
    pub identifier: Option<String>,
    pub metadata_value: VarInt,
    pub count: u16,
    pub block_runtime_id: VarUInt,
    pub extra_data: Vec<u8>,
}

impl PacketRead for StackRequestItem {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let descriptor_type = VarUInt::read(reader)?.0;
        let _legacy_type = u8::read(reader)?;
        let (identifier, metadata_value) = match descriptor_type {
            0 => (None, VarInt(0)),
            1 => (Some(String::read(reader)?), VarInt::read(reader)?),
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("unknown stack request item descriptor type {descriptor_type}"),
                ));
            }
        };
        let count = i16::read(reader)? as u16;
        let block_runtime_id = VarUInt::read(reader)?;
        let data_len = VarUInt::read(reader)?.0 as usize;
        if data_len > 1_048_576 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "extra_data length exceeds limit",
            ));
        }
        let mut extra_data = vec![0; data_len];
        reader.read_exact(&mut extra_data)?;
        Ok(Self {
            identifier,
            metadata_value,
            count,
            block_runtime_id,
            extra_data,
        })
    }
}

#[derive(Debug)]
pub enum ItemStackRequestAction {
    Take {
        count: u8,
        source: ItemStackRequestSlotInfo,
        destination: ItemStackRequestSlotInfo,
    },
    Place {
        count: u8,
        source: ItemStackRequestSlotInfo,
        destination: ItemStackRequestSlotInfo,
    },
    Swap {
        slot1: ItemStackRequestSlotInfo,
        slot2: ItemStackRequestSlotInfo,
    },
    Drop {
        count: u8,
        source: ItemStackRequestSlotInfo,
        randomly: bool,
    },
    Destroy {
        count: u8,
        source: ItemStackRequestSlotInfo,
    },
    Consume {
        count: u8,
        source: ItemStackRequestSlotInfo,
    },
    Create {
        result_index: u8,
    },
    LabTableCombine,
    BeaconPayment {
        primary_effect_id: VarInt,
        secondary_effect_id: VarInt,
    },
    MineBlock {
        hotbar_slot: VarInt,
        predicted_durability: VarInt,
        stack_id: VarInt,
    },
    CraftRecipe {
        recipe_id: VarUInt,
        repetitions: u8,
    },
    CraftRecipeAuto {
        recipe_id: VarUInt,
        repetitions: u8,
    },
    CraftCreative {
        creative_item_id: VarUInt,
        repetitions: u8,
    },
    Optional {
        recipe_id: VarUInt,
        filter_string_index: i32,
    },
    Grindstone {
        recipe_id: VarUInt,
        repair_cost: VarInt,
        repetitions: u8,
    },
    Loom {
        pattern_id: String,
        repetitions: u8,
    },
    CraftNonImplemented,
    CraftResultsDeprecated {
        result_items: Vec<StackRequestItem>,
        times_crafted: u8,
    },
}

impl PacketRead for ItemStackRequestAction {
    #[allow(clippy::too_many_lines)]
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let action_type = VarUInt::read(buf)?.0;
        let _legacy_action_type = u8::read(buf)?;
        match action_type {
            0 => Ok(Self::Take {
                count: u8::read(buf)?,
                source: ItemStackRequestSlotInfo::read(buf)?,
                destination: ItemStackRequestSlotInfo::read(buf)?,
            }),
            1 => Ok(Self::Place {
                count: u8::read(buf)?,
                source: ItemStackRequestSlotInfo::read(buf)?,
                destination: ItemStackRequestSlotInfo::read(buf)?,
            }),
            2 => Ok(Self::Swap {
                slot1: ItemStackRequestSlotInfo::read(buf)?,
                slot2: ItemStackRequestSlotInfo::read(buf)?,
            }),
            3 => Ok(Self::Drop {
                count: u8::read(buf)?,
                source: ItemStackRequestSlotInfo::read(buf)?,
                randomly: bool::read(buf)?,
            }),
            4 => Ok(Self::Destroy {
                count: u8::read(buf)?,
                source: ItemStackRequestSlotInfo::read(buf)?,
            }),
            5 => Ok(Self::Consume {
                count: u8::read(buf)?,
                source: ItemStackRequestSlotInfo::read(buf)?,
            }),
            6 => Ok(Self::Create {
                result_index: u8::read(buf)?,
            }),
            7 => Ok(Self::LabTableCombine),
            8 => Ok(Self::BeaconPayment {
                primary_effect_id: VarInt::read(buf)?,
                secondary_effect_id: VarInt::read(buf)?,
            }),
            9 => Ok(Self::MineBlock {
                hotbar_slot: VarInt::read(buf)?,
                predicted_durability: VarInt::read(buf)?,
                stack_id: VarInt(i32::read(buf)?),
            }),
            10 => Ok(Self::CraftRecipe {
                recipe_id: VarUInt::read(buf)?,
                repetitions: u8::read(buf)?,
            }),
            11 => {
                let recipe_id = VarUInt::read(buf)?;
                let repetitions = u8::read(buf)?;
                let count = collection_length(buf, "auto-craft ingredients")?;
                for _ in 0..count {
                    skip_autocraft_ingredient(buf)?;
                }
                Ok(Self::CraftRecipeAuto {
                    recipe_id,
                    repetitions,
                })
            }
            12 => Ok(Self::CraftCreative {
                creative_item_id: VarUInt::read(buf)?,
                repetitions: u8::read(buf)?,
            }),
            13 => Ok(Self::Optional {
                recipe_id: VarUInt::read(buf)?,
                filter_string_index: i32::read(buf)?,
            }),
            14 => Ok(Self::Grindstone {
                recipe_id: VarUInt(i32::read(buf)? as u32),
                repetitions: u8::read(buf)?,
                repair_cost: VarInt::read(buf)?,
            }),
            15 => Ok(Self::Loom {
                pattern_id: String::read(buf)?,
                repetitions: u8::read(buf)?,
            }),
            16 => Ok(Self::CraftNonImplemented),
            17 => {
                let result_items_len = collection_length(buf, "craft result items")?;
                let mut result_items = Vec::with_capacity(result_items_len);
                for _ in 0..result_items_len {
                    result_items.push(StackRequestItem::read(buf)?);
                }
                let times_crafted = u8::read(buf)?;
                Ok(Self::CraftResultsDeprecated {
                    result_items,
                    times_crafted,
                })
            }
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unknown ItemStackRequestAction ID: {action_type}"),
            )),
        }
    }
}

fn skip_autocraft_ingredient<R: Read>(reader: &mut R) -> Result<(), Error> {
    let descriptor_type = VarUInt::read(reader)?.0;
    let _legacy_type = u8::read(reader)?;
    match descriptor_type {
        0 => {}
        1 => {
            let _identifier = String::read(reader)?;
            let _aux = VarInt::read(reader)?;
        }
        2 => {
            let _expression = String::read(reader)?;
            let _version = i16::read(reader)?;
        }
        3 => {
            let _tag = String::read(reader)?;
        }
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("unknown item descriptor type {descriptor_type}"),
            ));
        }
    }
    let _count = u16::read(reader)?;
    Ok(())
}

#[derive(Debug)]
pub struct ItemStackRequest {
    pub request_id: VarInt,
    pub actions: Vec<ItemStackRequestAction>,
    pub filter_strings: Vec<String>,
    pub filter_cause: i32,
}

impl PacketRead for ItemStackRequest {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let request_id = VarInt::read(buf)?;
        let actions_len = collection_length(buf, "item stack request actions")?;
        let mut actions = Vec::with_capacity(actions_len);
        for _ in 0..actions_len {
            actions.push(ItemStackRequestAction::read(buf)?);
        }
        let filter_strings_len = collection_length(buf, "item stack request filters")?;
        let mut filter_strings = Vec::with_capacity(filter_strings_len);
        for _ in 0..filter_strings_len {
            filter_strings.push(String::read(buf)?);
        }
        let filter_cause = i32::read(buf)?;
        Ok(Self {
            request_id,
            actions,
            filter_strings,
            filter_cause,
        })
    }
}

#[derive(Debug)]
#[packet(147)]
pub struct SItemStackRequest {
    pub requests: Vec<ItemStackRequest>,
}

impl PacketRead for SItemStackRequest {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let requests_len = collection_length(buf, "item stack requests")?;
        let mut requests = Vec::with_capacity(requests_len);
        for _ in 0..requests_len {
            requests.push(ItemStackRequest::read(buf)?);
        }
        Ok(Self { requests })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bedrock::network_item::ContainerName;

    #[test]
    fn stack_request_slot_uses_raw_i32_network_id() {
        let slot = ItemStackRequestSlotInfo {
            container_name: FullContainerName {
                container_name: ContainerName::Inventory,
                dynamic_id: None,
            },
            slot_id: 4,
            stack_id: -2,
        };
        let mut encoded = Vec::new();
        slot.write(&mut encoded).unwrap();

        assert_eq!(&encoded[encoded.len() - 4..], &(-2i32).to_le_bytes());
        let decoded = ItemStackRequestSlotInfo::read(&mut encoded.as_slice()).unwrap();
        assert_eq!(decoded.stack_id, -2);
    }

    #[test]
    fn craft_result_item_uses_descriptor_backed_cereal_format() {
        let mut encoded = Vec::new();
        VarUInt(1).write(&mut encoded).unwrap();
        1u8.write(&mut encoded).unwrap();
        "minecraft:stone".to_string().write(&mut encoded).unwrap();
        VarInt(-1).write(&mut encoded).unwrap();
        2i16.write(&mut encoded).unwrap();
        VarUInt(3).write(&mut encoded).unwrap();
        VarUInt(2).write(&mut encoded).unwrap();
        encoded.extend_from_slice(&[0xaa, 0xbb]);

        let item = StackRequestItem::read(&mut encoded.as_slice()).unwrap();
        assert_eq!(item.identifier.as_deref(), Some("minecraft:stone"));
        assert_eq!(item.metadata_value, VarInt(-1));
        assert_eq!(item.count, 2);
        assert_eq!(item.block_runtime_id, VarUInt(3));
        assert_eq!(item.extra_data, [0xaa, 0xbb]);
    }

    #[test]
    fn craft_result_item_rejects_non_default_descriptors() {
        let mut encoded = Vec::new();
        VarUInt(2).write(&mut encoded).unwrap();
        2u8.write(&mut encoded).unwrap();
        assert!(StackRequestItem::read(&mut encoded.as_slice()).is_err());
    }
}
