// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{
    codec::{var_int::VarInt, var_ulong::VarULong},
    serial::PacketRead,
};

/// Sent by the client when performing player actions such as mining, sprinting, sneaking, or dimensions changes.
#[derive(Debug, PacketRead)]
#[packet(36)]
pub struct SPlayerAction {
    /// Runtime entity ID of the executing player.
    pub player_runtime_id: VarULong,
    /// Specific action type triggered.
    pub action: PlayerActionType,
    /// Coordinates of the target block involved in the action, or zero if none.
    pub block_position: BlockPos,
    /// Result block position (e.g. adjacent position where a block would be placed).
    pub result_pos: BlockPos,
    /// Face of the block targeted by the action.
    pub face: VarInt,
}

/// Category of action performed by a player.
#[derive(Clone, Copy, Debug, PacketRead)]
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
