use std::io::{Error, ErrorKind, Read};

use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt, var_ulong::VarULong},
    serial::PacketRead,
};

pub const WINDOW_ID_INVENTORY: i32 = 0;
pub const WINDOW_ID_OFF_HAND: i32 = 119;
pub const WINDOW_ID_ARMOUR: i32 = 120;
pub const WINDOW_ID_UI: i32 = 124;

#[derive(Debug, PartialEq, Eq)]
pub enum InventoryActionSource {
    Container,
    World,
    Creative,
    Todo,
    Unknown(u32),
}

impl From<u32> for InventoryActionSource {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Container,
            2 => Self::World,
            3 => Self::Creative,
            99999 => Self::Todo,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Debug)]
pub enum TransactionData {
    Normal(NormalTransactionData),
    Mismatch(MismatchTransactionData),
    UseItem(UseItemTransactionData),
    UseItemOnEntity(UseItemOnEntityTransactionData),
    ReleaseItem(ReleaseItemTransactionData),
}

#[derive(Debug, PacketRead)]
pub struct LegacySetItemSlot {
    pub container_id: u8,
    pub slots: Vec<u8>,
}

#[derive(Debug)]
pub struct InventoryAction {
    pub source_type: u32,
    pub window_id: Option<i32>,
    pub source_flags: Option<u32>,
    pub inventory_slot: u32,
    // TODO
    pub old_item: (),
    pub new_item: (),
}

impl PacketRead for InventoryAction {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let source_type = VarULong::read(buf)?.0 as u32;

        let mut window_id = None;
        let mut source_flags = None;

        match InventoryActionSource::from(source_type) {
            InventoryActionSource::Container | InventoryActionSource::Todo => {
                window_id = Some(VarInt::read(buf)?.0);
            }
            InventoryActionSource::World => {
                source_flags = Some(VarULong::read(buf)?.0 as u32);
            }
            _ => {}
        }

        let inventory_slot = VarULong::read(buf)?.0 as u32;

        // let old_item = ItemStack::read(buf)?;
        // let new_item = ItemStack::read(buf)?;

        Ok(Self {
            source_type,
            window_id,
            source_flags,
            inventory_slot,
            old_item: (),
            new_item: (),
        })
    }
}

#[derive(Debug, PacketRead)]
pub struct NormalTransactionData;

#[derive(Debug, PacketRead)]
pub struct MismatchTransactionData;

#[derive(Debug, PacketRead)]
pub struct UseItemTransactionData {
    pub action_type: VarULong,
    pub trigger_type: VarULong,
    pub block_position: BlockPos,
    pub block_face: VarInt,
    pub hot_bar_slot: VarInt,
    // TODO
}

#[derive(Debug, PacketRead)]
pub struct UseItemOnEntityTransactionData {
    pub target_entity_runtime_id: VarULong,
    pub action_type: VarULong,
    pub hot_bar_slot: VarInt,
    // TODO
}

#[derive(Debug, PacketRead)]
pub struct ReleaseItemTransactionData {
    pub action_type: VarULong,
    pub hot_bar_slot: VarInt,
    // TODO
}

#[derive(Debug)]
#[packet(30)]
pub struct SInventoryTransaction {
    pub legacy_request_id: VarInt,
    pub legacy_set_item_slots: Vec<LegacySetItemSlot>,
    pub actions: Vec<InventoryAction>,
    pub transaction_type: VarUInt,
    pub transaction_data: TransactionData,
}

impl PacketRead for SInventoryTransaction {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let legacy_request_id = VarInt::read(buf)?;

        let mut legacy_set_item_slots = Vec::new();
        if legacy_request_id.0 != 0 {
            let len = VarUInt::read(buf)?.0;
            for _ in 0..len {
                legacy_set_item_slots.push(LegacySetItemSlot::read(buf)?);
            }
        }

        let transaction_type = VarUInt::read(buf)?;

        let actions_len = VarUInt::read(buf)?.0;
        let mut actions = Vec::new();
        for _ in 0..actions_len {
            actions.push(InventoryAction::read(buf)?);
        }

        let transaction_data = match transaction_type.0 {
            0 => TransactionData::Normal(NormalTransactionData::read(buf)?),
            1 => TransactionData::Mismatch(MismatchTransactionData::read(buf)?),
            2 => TransactionData::UseItem(UseItemTransactionData::read(buf)?),
            3 => TransactionData::UseItemOnEntity(UseItemOnEntityTransactionData::read(buf)?),
            4 => TransactionData::ReleaseItem(ReleaseItemTransactionData::read(buf)?),
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Unknown inventory transaction type: {}", transaction_type.0),
                ));
            }
        };

        Ok(Self {
            legacy_request_id,
            legacy_set_item_slots,
            actions,
            transaction_type,
            transaction_data,
        })
    }
}
