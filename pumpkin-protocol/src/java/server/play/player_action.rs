use pumpkin_data::packet::serverbound::PLAY_PLAYER_ACTION;
use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::VarInt;

#[derive(serde::Deserialize)]
#[packet(PLAY_PLAYER_ACTION)]
pub struct SPlayerAction {
    pub status: VarInt,
    pub position: BlockPos,
    pub face: u8,
    pub sequence: VarInt,
}

pub enum Status {
    /// Sent when the player starts digging a block. If the block was insta-mined or the player is in creative mode, the client will not send `Status` = `FinishedDigging``, and will assume the server completed the destruction. To detect this, it is necessary to calculate the block destruction speed server-side.
    StartedDigging = 0,
    /// Sent when the player lets go of the Mine Block key (default: left click). Face is always set to -Y.
    CancelledDigging,
    /// Sent when the client thinks it is finished.
    FinishedDigging,
    /// Triggered by using the Drop Item key (default: Q) with the modifier to drop the entire selected stack (default: Control or Command, depending on OS). Location is always set to 0/0/0, Face is always set to -Y. Sequence is always set to 0.
    DropItemStack,
    /// Triggered by using the Drop Item key (default: Q). Location is always set to 0/0/0, Face is always set to -Y. Sequence is always set to 0.
    DropItem,
    /// I didn't make that up
    /// Indicates that the currently held item should have its state updated such as eating food, pulling back bows, using buckets, etc. Location is always set to 0/0/0, Face is always set to -Y. Sequence is always set to 0.
    ShootArrowOrFinishEating,
    /// Used to swap or assign an item to the second hand. Location is always set to 0/0/0, Face is always set to -Y. Sequence is always set to 0.  
    SwapItem,
}

pub struct InvalidStatus;

impl TryFrom<i32> for Status {
    type Error = InvalidStatus;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::StartedDigging),
            1 => Ok(Self::CancelledDigging),
            2 => Ok(Self::FinishedDigging),
            3 => Ok(Self::DropItemStack),
            4 => Ok(Self::DropItem),
            5 => Ok(Self::ShootArrowOrFinishEating),
            6 => Ok(Self::SwapItem),
            _ => Err(InvalidStatus),
        }
    }
}
