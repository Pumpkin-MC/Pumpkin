use std::io::{Error, ErrorKind, Read, Write};

use crate::{
    bedrock::network_item::FullContainerName,
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::{PacketRead, PacketWrite},
};
use pumpkin_macros::packet;

#[derive(Debug)]
pub struct ItemStackRequestSlotInfo {
    pub container_name: FullContainerName,
    pub slot_id: u8,
    pub stack_id: VarInt,
}

impl PacketRead for ItemStackRequestSlotInfo {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let container_name = FullContainerName::read(buf)?;
        let slot_id = u8::read(buf)?;
        let stack_id = VarInt::read(buf)?;
        Ok(Self {
            container_name,
            slot_id,
            stack_id,
        })
    }
}

impl PacketWrite for ItemStackRequestSlotInfo {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.container_name.write(writer)?;
        self.slot_id.write(writer)?;
        self.stack_id.write(writer)?;
        Ok(())
    }
}

/// Bounds for `ItemStackRequest` wire arrays (alloc bombs from huge `VarUInt`).
const MAX_ITEM_STACK_ACTIONS: usize = 256;
const MAX_ITEM_STACK_REQUESTS: usize = 64;
const MAX_RESULT_ITEMS: usize = 64;
const MAX_FILTER_STRINGS: usize = 64;

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
    PlaceInContainer {
        count: u8,
        source: ItemStackRequestSlotInfo,
        destination: ItemStackRequestSlotInfo,
    },
    TakeOutContainer {
        count: u8,
        source: ItemStackRequestSlotInfo,
        destination: ItemStackRequestSlotInfo,
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
        repetitions2: u8,
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
        result_items: Vec<crate::bedrock::network_item::NetworkItemStack>,
        times_crafted: u8,
    },
}

impl PacketRead for ItemStackRequestAction {
    #[allow(clippy::too_many_lines)]
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let action_type = u8::read(buf)?;
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
            7 => Ok(Self::PlaceInContainer {
                count: u8::read(buf)?,
                source: ItemStackRequestSlotInfo::read(buf)?,
                destination: ItemStackRequestSlotInfo::read(buf)?,
            }),
            8 => Ok(Self::TakeOutContainer {
                count: u8::read(buf)?,
                source: ItemStackRequestSlotInfo::read(buf)?,
                destination: ItemStackRequestSlotInfo::read(buf)?,
            }),
            9 => Ok(Self::LabTableCombine),
            10 => Ok(Self::BeaconPayment {
                primary_effect_id: VarInt::read(buf)?,
                secondary_effect_id: VarInt::read(buf)?,
            }),
            11 => Ok(Self::MineBlock {
                hotbar_slot: VarInt::read(buf)?,
                predicted_durability: VarInt::read(buf)?,
                stack_id: VarInt::read(buf)?,
            }),
            12 => Ok(Self::CraftRecipe {
                recipe_id: VarUInt::read(buf)?,
                repetitions: u8::read(buf)?,
            }),
            13 => {
                let recipe_id = VarUInt::read(buf)?;
                let repetitions = u8::read(buf)?;
                let repetitions2 = u8::read(buf)?;
                let count = u8::read(buf)?;
                // Read and discard ingredients if present (we don't need them server-side)
                if count > 0 {
                    for _ in 0..count {
                        // NetworkItemStack includes id, count, aux_value, block_runtime_id and extra_data
                        let _ = crate::bedrock::network_item::NetworkItemStack::read(buf)?;
                    }
                }
                Ok(Self::CraftRecipeAuto {
                    recipe_id,
                    repetitions,
                    repetitions2,
                })
            }
            14 => Ok(Self::CraftCreative {
                creative_item_id: VarUInt::read(buf)?,
                repetitions: u8::read(buf)?,
            }),
            15 => Ok(Self::Optional {
                recipe_id: VarUInt::read(buf)?,
                filter_string_index: i32::read(buf)?,
            }),
            16 => Ok(Self::Grindstone {
                recipe_id: VarUInt::read(buf)?,
                repair_cost: VarInt::read(buf)?,
                repetitions: u8::read(buf)?,
            }),
            17 => Ok(Self::Loom {
                pattern_id: String::read(buf)?,
                repetitions: u8::read(buf)?,
            }),
            18 => Ok(Self::CraftNonImplemented),
            19 => {
                let result_items_len = VarUInt::read(buf)?.0 as usize;
                if result_items_len > MAX_RESULT_ITEMS {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("result_items_len {result_items_len} exceeds {MAX_RESULT_ITEMS}"),
                    ));
                }
                let mut result_items = Vec::with_capacity(result_items_len);
                for _ in 0..result_items_len {
                    result_items.push(crate::bedrock::network_item::NetworkItemStack::read(buf)?);
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
        let actions_len = VarUInt::read(buf)?.0 as usize;
        if actions_len > MAX_ITEM_STACK_ACTIONS {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("actions_len {actions_len} exceeds {MAX_ITEM_STACK_ACTIONS}"),
            ));
        }
        let mut actions = Vec::with_capacity(actions_len);
        for _ in 0..actions_len {
            actions.push(ItemStackRequestAction::read(buf)?);
        }
        let filter_strings_len = VarUInt::read(buf)?.0 as usize;
        if filter_strings_len > MAX_FILTER_STRINGS {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("filter_strings_len {filter_strings_len} exceeds {MAX_FILTER_STRINGS}"),
            ));
        }
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
        let requests_len = VarUInt::read(buf)?.0 as usize;
        if requests_len > MAX_ITEM_STACK_REQUESTS {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("requests_len {requests_len} exceeds {MAX_ITEM_STACK_REQUESTS}"),
            ));
        }
        let mut requests = Vec::with_capacity(requests_len);
        for _ in 0..requests_len {
            requests.push(ItemStackRequest::read(buf)?);
        }
        Ok(Self { requests })
    }
}

#[cfg(test)]
mod alloc_cap_tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn rejects_oversize_actions_len() {
        // request_id VarInt(0) + actions_len VarUInt huge
        let mut buf = Vec::new();
        // request_id = 0
        buf.push(0);
        // VarUInt 10000 encoded - write a large value via many continuation bits is hard;
        // encode 300 as varuint: 300 = 0b1_00101100 with continuation
        // Simple: write VarUInt manually for MAX+1
        let over = (MAX_ITEM_STACK_ACTIONS + 1) as u32;
        let mut v = over;
        loop {
            let mut b = (v & 0x7F) as u8;
            v >>= 7;
            if v != 0 {
                b |= 0x80;
            }
            buf.push(b);
            if v == 0 {
                break;
            }
        }
        let err = ItemStackRequest::read(&mut Cursor::new(buf)).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
    }
}
