// Last verified for v2169

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

#[derive(Debug, PacketRead)]
#[repr(i32)]
#[serial(varint)]
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
