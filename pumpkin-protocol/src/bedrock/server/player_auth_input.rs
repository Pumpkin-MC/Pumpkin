use std::io::{Error, Read};

use pumpkin_macros::packet;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};

use crate::{
    codec::{
        bitset::Bitset, var_int::VarInt, var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong,
    },
    serial::PacketRead,
};

#[derive(Debug)]
#[packet(144)]
pub struct SPlayerAuthInput {
    pub pitch: f32,
    pub yaw: f32,
    pub position: Vector3<f32>,
    pub move_vec: Vector2<f32>,
    pub head_yaw: f32,
    pub input_data: Bitset<65>,
    pub input_mode: VarUInt,
    pub play_mode: VarUInt,
    pub interaction_model: VarUInt,
    pub interact_pitch: f32,
    pub interact_yaw: f32,
    pub tick: VarULong,
    pub delta: Vector3<f32>,
    pub block_actions: Option<Vec<PlayerBlockAction>>,
    pub item_interaction: Option<PlayerInventoryAction>,
    pub item_stack_request: Option<crate::bedrock::server::item_stack_request::ItemStackRequest>,
    pub vehicle_rotation: Option<Vector2<f32>>,
    pub vehicle_unique_id: Option<VarLong>,
    pub analog_move: Vector2<f32>,
    pub camera_orientation: Vector3<f32>,
    pub raw_move: Vector2<f32>,
}

impl PacketRead for SPlayerAuthInput {
    #[expect(clippy::useless_let_if_seq)]
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let pitch = f32::read(reader)?;
        let yaw = f32::read(reader)?;
        let position = Vector3::<f32>::read(reader)?;
        let move_vec = Vector2::<f32>::read(reader)?;
        let head_yaw = f32::read(reader)?;
        let input_data = Bitset::<65>::read(reader)?;
        let input_mode = VarUInt::read(reader)?;
        let play_mode = VarUInt::read(reader)?;
        let interaction_model = VarUInt::read(reader)?;
        let interact_pitch = f32::read(reader)?;
        let interact_yaw = f32::read(reader)?;
        let tick = VarULong::read(reader)?;
        let delta = Vector3::<f32>::read(reader)?;

        // 1. Perform Item Interaction
        let item_interaction = if input_data.get(InputData::PerformItemInteraction as usize) {
            Some(PlayerInventoryAction::read(reader)?)
        } else {
            None
        };

        // 2. Item Stack Request
        let item_stack_request = if input_data.get(InputData::PerformItemStackRequest as usize) {
            Some(crate::bedrock::server::item_stack_request::ItemStackRequest::read(reader)?)
        } else {
            None
        };

        // 3. Block Actions
        let block_actions = if input_data.get(InputData::PerformBlockActions as usize) {
            const MAX_BLOCK_ACTIONS: usize = 64;
            let count_i = VarInt::read(reader)?.0;
            if count_i < 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "negative block action count",
                ));
            }
            let count = count_i as usize;
            if count > MAX_BLOCK_ACTIONS {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("block action count {count_i} exceeds limit {MAX_BLOCK_ACTIONS}"),
                ));
            }
            let mut actions = Vec::with_capacity(count);
            for _ in 0..count {
                actions.push(PlayerBlockAction::read(reader)?);
            }
            Some(actions)
        } else {
            None
        };

        // 4. Vehicle Info (Matches Go logic)
        let mut vehicle_rotation = None;
        let mut vehicle_unique_id = None;
        if input_data.get(InputData::ClientPredictedVehicle as usize) {
            vehicle_rotation = Some(Vector2::<f32>::read(reader)?);
            vehicle_unique_id = Some(VarLong::read(reader)?);
        }

        // 5. Trailing Data
        let analog_move = Vector2::<f32>::read(reader)?;
        let camera_orientation = Vector3::<f32>::read(reader)?;
        let raw_move = Vector2::<f32>::read(reader)?;

        Ok(Self {
            pitch,
            yaw,
            position,
            move_vec,
            head_yaw,
            input_data,
            input_mode,
            play_mode,
            interaction_model,
            interact_pitch,
            interact_yaw,
            tick,
            delta,
            block_actions,
            item_interaction,
            item_stack_request,
            vehicle_rotation,
            vehicle_unique_id,
            analog_move,
            camera_orientation,
            raw_move,
        })
    }
}

#[derive(Debug)]
pub struct PlayerInventoryAction {
    pub legacy_request_id: VarInt,
    pub legacy_slots: Vec<crate::bedrock::server::inventory_transaction::LegacySetItemSlot>,
    pub actions: Vec<crate::bedrock::server::inventory_transaction::InventoryAction>,
    pub transaction: crate::bedrock::server::inventory_transaction::UseItemTransactionData,
}

/// Bound for the action list in an inventory transaction carried by auth input.
/// Vanilla sends a handful; an unbounded `VarUInt` length pre-allocates attacker-sized memory.
const MAX_INVENTORY_ACTIONS: usize = 64;

/// Bound for the legacy set-item slot entry count carried by auth input.
/// Vanilla sends a handful; an unbounded `VarUInt` count drives an attacker-sized parse loop.
const MAX_LEGACY_SLOTS: u32 = 256;

impl PacketRead for PlayerInventoryAction {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let legacy_request_id = VarInt::read(buf)?;
        let mut legacy_slots = Vec::new();
        if legacy_request_id.0 < -1 && (legacy_request_id.0 & 1) == 0 {
            let slots_len = VarUInt::read(buf)?.0;
            if slots_len > MAX_LEGACY_SLOTS {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("legacy slot count {slots_len} exceeds limit {MAX_LEGACY_SLOTS}"),
                ));
            }
            for _ in 0..slots_len {
                legacy_slots.push(
                    crate::bedrock::server::inventory_transaction::LegacySetItemSlot::read(buf)?,
                );
            }
        }
        let actions_len = VarUInt::read(buf)?.0 as usize;
        if actions_len > MAX_INVENTORY_ACTIONS {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "inventory action count {actions_len} exceeds limit {MAX_INVENTORY_ACTIONS}"
                ),
            ));
        }
        let mut actions = Vec::with_capacity(actions_len);
        for _ in 0..actions_len {
            actions
                .push(crate::bedrock::server::inventory_transaction::InventoryAction::read(buf)?);
        }
        let transaction =
            crate::bedrock::server::inventory_transaction::UseItemTransactionData::read(buf)?;
        Ok(Self {
            legacy_request_id,
            legacy_slots,
            actions,
            transaction,
        })
    }
}

#[derive(Debug, PacketRead)]
pub struct PlayerBlockAction {
    pub action: VarInt,
    pub block_pos: BlockPos,
    pub face: VarInt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum InputMode {
    Mouse = 1,
    Touch = 2,
    GamePad = 3,
    MotionController = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PlayMode {
    Normal = 0,
    Teaser = 1,
    Screen = 2,
    ExitLevel = 7,
    NumModes = 9,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum InteractionModel {
    Touch = 0,
    Crosshair = 1,
    Classic = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputData {
    Ascend = 0,
    Descend = 1,
    NorthJump = 2,
    JumpDown = 3,
    SprintDown = 4,
    ChangeHeight = 5,
    Jumping = 6,
    AutoJumpingInWater = 7,
    Sneaking = 8,
    SneakDown = 9,
    Up = 10,
    Down = 11,
    Left = 12,
    Right = 13,
    UpLeft = 14,
    UpRight = 15,
    WantUp = 16,
    WantDown = 17,
    WantDownSlow = 18,
    WantUpSlow = 19,
    Sprinting = 20,
    AscendBlock = 21,
    DescendBlock = 22,
    SneakToggleDown = 23,
    PersistSneak = 24,
    StartSprinting = 25,
    StopSprinting = 26,
    StartSneaking = 27,
    StopSneaking = 28,
    StartSwimming = 29,
    StopSwimming = 30,
    StartJumping = 31,
    StartGliding = 32,
    StopGliding = 33,
    PerformItemInteraction = 34,
    PerformBlockActions = 35,
    PerformItemStackRequest = 36,
    HandledTeleport = 37,
    Emoting = 38,
    MissedSwing = 39,
    StartCrawling = 40,
    StopCrawling = 41,
    StartFlying = 42,
    StopFlying = 43,
    ClientAckServerData = 44,
    ClientPredictedVehicle = 45,
    PaddlingLeft = 46,
    PaddlingRight = 47,
    BlockBreakingDelayEnabled = 48,
    HorizontalCollision = 49,
    VerticalCollision = 50,
    DownLeft = 51,
    DownRight = 52,
    StartUsingItem = 53,
    CameraRelativeMovementEnabled = 54,
    RotControlledByMoveDirection = 55,
    StartSpinAttack = 56,
    StopSpinAttack = 57,
    IsHotbarTouchOnly = 58,
    JumpReleasedRaw = 59,
    JumpPressedRaw = 60,
    JumpCurrentRaw = 61,
    SneakReleasedRaw = 62,
    SneakPressedRaw = 63,
    SneakCurrentRaw = 64,
}

#[cfg(test)]
mod alloc_cap_tests {
    use super::*;
    use crate::bedrock::network_item::NetworkItemDescriptor;
    use crate::serial::PacketWrite;
    use std::io::Cursor;

    fn encode_inventory_action(buf: &mut Vec<u8>) {
        // source_type Container, window_id 0, slot 0, empty old/new items
        VarULong(0).write(buf).unwrap();
        VarInt(0).write(buf).unwrap();
        VarULong(0).write(buf).unwrap();
        NetworkItemDescriptor::default().write(buf).unwrap();
        NetworkItemDescriptor::default().write(buf).unwrap();
    }

    fn encode_use_item_transaction(buf: &mut Vec<u8>) {
        VarUInt(0).write(buf).unwrap(); // action_type
        0u8.write(buf).unwrap(); // trigger_type
        VarInt(0).write(buf).unwrap(); // block_pos x
        VarInt(0).write(buf).unwrap(); // block_pos y
        VarInt(0).write(buf).unwrap(); // block_pos z
        0u8.write(buf).unwrap(); // block_face
        VarInt(0).write(buf).unwrap(); // hot_bar_slot
        NetworkItemDescriptor::default().write(buf).unwrap(); // item_in_hand
        for _ in 0..6 {
            0.0f32.write(buf).unwrap(); // player_position + click_position
        }
        VarUInt(0).write(buf).unwrap(); // block_runtime_id
        0u8.write(buf).unwrap(); // client_prediction
        0u8.write(buf).unwrap(); // client_cooldown_state
    }

    #[test]
    fn accepts_actions_len_at_cap() {
        let mut buf = Vec::new();
        VarInt(0).write(&mut buf).unwrap(); // legacy_request_id: no legacy slots
        VarUInt(MAX_INVENTORY_ACTIONS as u32)
            .write(&mut buf)
            .unwrap();
        for _ in 0..MAX_INVENTORY_ACTIONS {
            encode_inventory_action(&mut buf);
        }
        encode_use_item_transaction(&mut buf);

        let parsed = PlayerInventoryAction::read(&mut Cursor::new(buf)).unwrap();
        assert_eq!(parsed.actions.len(), MAX_INVENTORY_ACTIONS);
    }

    #[test]
    fn rejects_actions_len_over_cap() {
        let mut buf = Vec::new();
        VarInt(0).write(&mut buf).unwrap(); // legacy_request_id: no legacy slots
        VarUInt((MAX_INVENTORY_ACTIONS + 1) as u32)
            .write(&mut buf)
            .unwrap();

        let err = PlayerInventoryAction::read(&mut Cursor::new(buf)).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_legacy_slots_len_over_cap() {
        let mut buf = Vec::new();
        VarInt(-2).write(&mut buf).unwrap(); // legacy_request_id: legacy slots follow
        VarUInt(MAX_LEGACY_SLOTS + 1).write(&mut buf).unwrap();

        let err = PlayerInventoryAction::read(&mut Cursor::new(buf)).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }
}
