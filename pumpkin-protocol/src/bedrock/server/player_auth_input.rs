use std::io::{Error, Read};

use pumpkin_macros::packet;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};

use crate::{
    codec::{
        bitset::Bitset,
        var_int::VarInt,
        var_long::{VarLong, VarLongType},
        var_uint::VarUInt,
        var_ulong::VarULong,
    },
    packet::{self, Packet},
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
        let input_data = Bitset::<65>::read(reader)?;
        let input_mode = VarUInt::read(reader)?;
        let play_mode = VarUInt::read(reader)?;
        let interaction_model = VarUInt::read(reader)?;
        let interact_pitch = f32::read(reader)?;
        let interact_yaw = f32::read(reader)?;
        let tick = VarULong::read(reader)?;
        let delta = Vector3::<f32>::read(reader)?;

        // 1. Perform Item Interaction
        if input_data.get(InputData::PerformItemInteraction as usize) {
            // protocol.UseItemTransactionData (Simplified skip)
            let _action_type = VarUInt::read(reader)?;
            let action_count = VarUInt::read(reader)?.0;
            for _ in 0..action_count {
                let _type = VarUInt::read(reader)?;
                let _pos = BlockPos::read(reader)?;
                let _face = VarInt::read(reader)?;
            }
        }

        // 2. Item Stack Request
        if input_data.get(InputData::PerformItemStackRequest as usize) {
            // protocol.Single ItemStackRequest
            return Err(Error::other("ItemStackRequest decoding not implemented"));
        }

        // 3. Block Actions
        let block_actions = if input_data.get(InputData::PerformBlockActions as usize) {
            let count = VarInt::read(reader)?.0 as usize;
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
            vehicle_rotation,
            vehicle_unique_id,
            analog_move,
            camera_orientation,
            raw_move,
        })
    }
}

#[derive(Debug, PacketRead)]
pub struct PlayerBlockAction {
    pub action: VarInt,
    pub block_pos: BlockPos,
    pub face: VarInt,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum InputMode {
    Mouse = 1,
    Touch = 2,
    GamePad = 3,
    MotionController = 4,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum PlayMode {
    Normal = 0,
    Teaser = 1,
    Screen = 2,
    ExitLevel = 7,
    NumModes = 9,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum InteractionModel {
    Touch = 0,
    Crosshair = 1,
    Classic = 2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
    ClientPredictedVehicle = 45, // Renamed to match Go
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
