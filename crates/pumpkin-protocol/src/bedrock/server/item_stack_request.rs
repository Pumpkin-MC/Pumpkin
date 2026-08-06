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
        let stack_id = VarInt(i32::read(buf)?);
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
                let count = VarUInt::read(buf)?.0;
                for _ in 0..count {
                    skip_autocraft_ingredient(buf)?;
                }
                Ok(Self::CraftRecipeAuto {
                    recipe_id,
                    repetitions,
                    repetitions2: repetitions,
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
                let result_items_len = VarUInt::read(buf)?.0;
                for _ in 0..result_items_len {
                    skip_request_result_item(buf)?;
                }
                let times_crafted = u8::read(buf)?;
                Ok(Self::CraftResultsDeprecated {
                    result_items: Vec::new(),
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

fn skip_request_result_item<R: Read>(reader: &mut R) -> Result<(), Error> {
    let descriptor_type = VarUInt::read(reader)?.0;
    let _legacy_type = u8::read(reader)?;
    if descriptor_type != 0 {
        let _identifier = String::read(reader)?;
        let _aux = VarInt::read(reader)?;
    }
    let _count = i16::read(reader)?;
    let _block_runtime_id = VarUInt::read(reader)?;
    let data_len = VarUInt::read(reader)?.0 as usize;
    let mut data = vec![0; data_len];
    reader.read_exact(&mut data)
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
        let actions_len = VarUInt::read(buf)?.0;
        let mut actions = Vec::with_capacity(actions_len as usize);
        for _ in 0..actions_len {
            actions.push(ItemStackRequestAction::read(buf)?);
        }
        let filter_strings_len = VarUInt::read(buf)?.0;
        let mut filter_strings = Vec::with_capacity(filter_strings_len as usize);
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
        let requests_len = VarUInt::read(buf)?.0;
        let mut requests = Vec::with_capacity(requests_len as usize);
        for _ in 0..requests_len {
            requests.push(ItemStackRequest::read(buf)?);
        }
        Ok(Self { requests })
    }
}
