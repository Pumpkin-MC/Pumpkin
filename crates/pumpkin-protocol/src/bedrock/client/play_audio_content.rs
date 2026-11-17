use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::serial::PacketWrite;

/// Sound playback options for signed audio content.
#[derive(Clone, Debug, PartialEq, PacketWrite)]
pub struct AudioContentSound {
    pub sound_name: String,
    pub position: BlockPos,
    pub volume: f32,
    pub pitch: f32,
}

/// Sent by the server to instruct the client to play signed audio content.
#[derive(Clone, Debug, PartialEq, PacketWrite)]
#[packet(331)]
pub struct CPlayAudioContent {
    pub shared_metadata: String,
    pub playback_content: String,
    pub playback_type: String,
    pub sound: AudioContentSound,
}
