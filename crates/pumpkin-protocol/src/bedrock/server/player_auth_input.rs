use std::io::{Error, ErrorKind, Read};

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
    pub input_data: Bitset<66>,
    pub input_mode: VarUInt,
    pub play_mode: VarUInt,
    pub interaction_model: VarInt,
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
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let pitch = f32::read(reader)?;
        let yaw = f32::read(reader)?;
        let position = Vector3::<f32>::read(reader)?;
        let move_vec = Vector2::<f32>::read(reader)?;
        let head_yaw = f32::read(reader)?;
        let mut input_data = Bitset::<66>::default();
        if bool::read(reader)? {
            let count = VarUInt::read(reader)?.0;
            if count > 66 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("too many player input flags: {count}"),
                ));
            }
            for _ in 0..count {
                let flag = VarInt::read(reader)?.0;
                if !(0..66).contains(&flag) {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("invalid player input flag {flag}"),
                    ));
                }
                if input_data.get(flag as usize) {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("duplicate player input flag {flag}"),
                    ));
                }
                input_data.set(flag as usize, true);
            }
        }
        let input_mode = VarUInt::read(reader)?;
        let play_mode = VarUInt::read(reader)?;
        let interaction_model = VarInt::read(reader)?;
        let interact_pitch = f32::read(reader)?;
        let interact_yaw = f32::read(reader)?;
        let tick = VarULong::read(reader)?;
        let delta = Vector3::<f32>::read(reader)?;

        // 1. Perform Item Interaction
        let item_interaction = if bool::read(reader)? && bool::read(reader)? {
            Some(PlayerInventoryAction::read(reader)?)
        } else {
            None
        };

        // 2. Item Stack Request
        let item_stack_request = if bool::read(reader)? && bool::read(reader)? {
            Some(crate::bedrock::server::item_stack_request::ItemStackRequest::read(reader)?)
        } else {
            None
        };

        // 3. Block Actions
        let block_actions = if bool::read(reader)? && bool::read(reader)? {
            let count = VarUInt::read(reader)?.0 as usize;
            if count > 1024 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "block_actions count exceeds limit",
                ));
            }
            let mut actions = Vec::with_capacity(count.min(64));
            for _ in 0..count {
                actions.push(PlayerBlockAction::read(reader)?);
            }
            Some(actions)
        } else {
            None
        };

        // 4. Vehicle Info (Matches Go logic)
        let vehicle_rotation = (bool::read(reader)? && bool::read(reader)?)
            .then(|| Vector2::<f32>::read(reader))
            .transpose()?;
        let vehicle_unique_id = (bool::read(reader)? && bool::read(reader)?)
            .then(|| VarLong::read(reader))
            .transpose()?;

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
    pub transaction: PlayerUseItemTransactionData,
}

impl PacketRead for PlayerInventoryAction {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let legacy_request_id = VarInt::read(buf)?;
        let mut legacy_slots = Vec::new();
        if bool::read(buf)? && legacy_request_id.0 < -1 && (legacy_request_id.0 & 1) == 0 {
            let slots_len = VarUInt::read(buf)?.0 as usize;
            if slots_len > 1024 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "slots_len exceeds limit",
                ));
            }
            legacy_slots.reserve(slots_len.min(64));
            for _ in 0..slots_len {
                legacy_slots.push(
                    crate::bedrock::server::inventory_transaction::LegacySetItemSlot::read(buf)?,
                );
            }
        }
        let mut actions = Vec::new();
        if bool::read(buf)? && bool::read(buf)? {
            let actions_len = VarUInt::read(buf)?.0 as usize;
            if actions_len > 1024 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "actions_len exceeds limit",
                ));
            }
            actions.reserve(actions_len.min(64));
            for _ in 0..actions_len {
                actions.push(
                    crate::bedrock::server::inventory_transaction::InventoryAction::read(buf)?,
                );
            }
        }
        let transaction = PlayerUseItemTransactionData::read(buf)?;
        Ok(Self {
            legacy_request_id,
            legacy_slots,
            actions,
            transaction,
        })
    }
}

#[derive(Debug)]
pub struct PlayerUseItemTransactionData {
    pub action_type: VarInt,
    pub trigger_type: u8,
    pub block_position: BlockPos,
    pub block_face: u8,
    pub hot_bar_slot: VarInt,
    pub item_in_hand: crate::bedrock::network_item::NetworkItemDescriptor,
    pub player_position: Vector3<f32>,
    pub click_position: Vector3<f32>,
    pub block_runtime_id: VarUInt,
    pub client_prediction: u8,
    pub client_cooldown_state: u8,
}

impl PacketRead for PlayerUseItemTransactionData {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            action_type: VarInt::read(reader)?,
            trigger_type: u8::read(reader)?,
            block_position: BlockPos::read(reader)?,
            block_face: u8::read(reader)?,
            hot_bar_slot: VarInt::read(reader)?,
            item_in_hand: crate::bedrock::network_item::NetworkItemDescriptor::read(reader)?,
            player_position: Vector3::read(reader)?,
            click_position: Vector3::read(reader)?,
            block_runtime_id: VarUInt::read(reader)?,
            client_prediction: u8::read(reader)?,
            client_cooldown_state: u8::read(reader)?,
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
    InternalUpdate = 65,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serial::PacketWrite;

    fn auth_input_prefix() -> Vec<u8> {
        let mut bytes = Vec::new();
        for _ in 0..8 {
            0.0f32.write(&mut bytes).unwrap();
        }
        bytes
    }

    #[test]
    fn auth_input_reads_v2168_flag_list_and_signed_interaction_model() {
        let mut bytes = auth_input_prefix();
        true.write(&mut bytes).unwrap();
        VarUInt(2).write(&mut bytes).unwrap();
        VarInt(0).write(&mut bytes).unwrap();
        VarInt(65).write(&mut bytes).unwrap();
        VarUInt(0).write(&mut bytes).unwrap();
        VarUInt(0).write(&mut bytes).unwrap();
        VarInt(-1).write(&mut bytes).unwrap();
        0.0f32.write(&mut bytes).unwrap();
        0.0f32.write(&mut bytes).unwrap();
        VarULong(0).write(&mut bytes).unwrap();
        for _ in 0..3 {
            0.0f32.write(&mut bytes).unwrap();
        }
        for _ in 0..5 {
            true.write(&mut bytes).unwrap();
            false.write(&mut bytes).unwrap();
        }
        for _ in 0..7 {
            0.0f32.write(&mut bytes).unwrap();
        }

        let packet = SPlayerAuthInput::read(&mut bytes.as_slice()).unwrap();
        assert!(packet.input_data.get(0usize));
        assert!(packet.input_data.get(65usize));
        assert_eq!(packet.interaction_model, VarInt(-1));
    }

    #[test]
    fn auth_input_rejects_duplicate_flags() {
        let mut bytes = auth_input_prefix();
        true.write(&mut bytes).unwrap();
        VarUInt(2).write(&mut bytes).unwrap();
        VarInt(1).write(&mut bytes).unwrap();
        VarInt(1).write(&mut bytes).unwrap();

        let error = SPlayerAuthInput::read(&mut bytes.as_slice()).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
    }
}
