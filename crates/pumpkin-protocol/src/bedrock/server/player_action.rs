// Last verified for v2169

use std::io::{Error, Read};

use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{
    codec::{var_int::VarInt, var_ulong::VarULong},
    serial::PacketRead,
};

#[derive(Debug, PacketRead)]
#[packet(36)]
pub struct SPlayerAction {
    pub player_runtime_id: VarULong,
    pub action: PlayerActionType,
    pub block_position: BlockPos,
    pub result_pos: BlockPos,
    pub face: VarInt,
}

#[derive(Debug)]
#[repr(i32)]
pub enum PlayerActionType {
    Unknown = -1,
    StartDestroyBlock,
    AbortDestroyBlock,
    StopDestroyBlock,
    GetUpdatedBlock,
    /// Seems to be not used, or atleast not send by client
    DropItem,
    StartSleeping,
    StopSleeping,
    Respawn,
    StartJump,
    StartSprinting,
    StopSprinting,
    StartSneaking,
    StopSneaking,
    CreativeDestroyBlock,
    ChangeDimensionAck,
    StartGliding,
    StopGliding,
    DenyDestroyBlock,
    CrackBlock,
    ChangeSkin,
    UpdatedEnchantingSeed,
    StartSwimming,
    StopSwimming,
    StartSpinAttack,
    StopSpinAttack,
    InteractWithBlock,
    PredictDestroyBlock,
    ContinueDestroyBlock,
    StartItemUseOn,
    StopItemUseOn,
    HandledTeleport,
    MissedSwing,
    StartCrawling,
    StopCrawling,
    StartFlying,
    StopFlying,
    ClientAckServerData,
    StartUsingItem,
    InternalUpdate,
    Count,
}

impl TryFrom<i32> for PlayerActionType {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            -1 => Ok(Self::Unknown),
            0 => Ok(Self::StartDestroyBlock),
            1 => Ok(Self::AbortDestroyBlock),
            2 => Ok(Self::StopDestroyBlock),
            3 => Ok(Self::GetUpdatedBlock),
            4 => Ok(Self::DropItem),
            5 => Ok(Self::StartSleeping),
            6 => Ok(Self::StopSleeping),
            7 => Ok(Self::Respawn),
            8 => Ok(Self::StartJump),
            9 => Ok(Self::StartSprinting),
            10 => Ok(Self::StopSprinting),
            11 => Ok(Self::StartSneaking),
            12 => Ok(Self::StopSneaking),
            13 => Ok(Self::CreativeDestroyBlock),
            14 => Ok(Self::ChangeDimensionAck),
            15 => Ok(Self::StartGliding),
            16 => Ok(Self::StopGliding),
            17 => Ok(Self::DenyDestroyBlock),
            18 => Ok(Self::CrackBlock),
            19 => Ok(Self::ChangeSkin),
            20 => Ok(Self::UpdatedEnchantingSeed),
            21 => Ok(Self::StartSwimming),
            22 => Ok(Self::StopSwimming),
            23 => Ok(Self::StartSpinAttack),
            24 => Ok(Self::StopSpinAttack),
            25 => Ok(Self::InteractWithBlock),
            26 => Ok(Self::PredictDestroyBlock),
            27 => Ok(Self::ContinueDestroyBlock),
            28 => Ok(Self::StartItemUseOn),
            29 => Ok(Self::StopItemUseOn),
            30 => Ok(Self::HandledTeleport),
            31 => Ok(Self::MissedSwing),
            32 => Ok(Self::StartCrawling),
            33 => Ok(Self::StopCrawling),
            34 => Ok(Self::StartFlying),
            35 => Ok(Self::StopFlying),
            36 => Ok(Self::ClientAckServerData),
            37 => Ok(Self::StartUsingItem),
            38 => Ok(Self::InternalUpdate),
            39 => Ok(Self::Count),
            _ => Err(format!("Invalid action ID: {value}")),
        }
    }
}

impl PacketRead for PlayerActionType {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let action = VarInt::read(reader)?;

        Self::try_from(action.0).map_err(Error::other)
    }
}
