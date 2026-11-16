use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

/// Matchmaking state reported to the client.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PacketWrite)]
#[repr(u8)]
pub enum MatchmakingState {
    Idle = 0,
    Matchmaking = 1,
    MatchFound = 2,
    Canceled = 3,
    PlayerLeftParty = 4,
    PlayerLeftServer = 5,
    ServerShutdown = 6,
    TimedOut = 7,
    RequeueAsParty = 8,
}

/// Optional details regarding the matchmaking transition.
#[derive(Clone, Debug, PartialEq, Eq, PacketWrite)]
pub struct MatchmakingStateOptions {
    pub triggering_player_net_id: String,
    pub result_code: VarInt,
    pub error_reason: String,
    pub estimated_wait_seconds: VarInt,
    pub match_id: String,
    pub session_id: String,
}

/// Sent by the server to inform the client of matchmaking status.
#[derive(Clone, Debug, PartialEq, Eq, PacketWrite)]
#[packet(328)]
pub struct CMatchmakingState {
    pub state: u8,
    pub destination_name: String,
    pub options: Option<MatchmakingStateOptions>,
}
