use std::io::{Error, Read};

use pumpkin_macros::packet;

use crate::serial::PacketRead;

/// Sent by the client to request cancellation of the matchmaking ticket it currently holds.
#[derive(Debug)]
#[packet(329)]
pub struct SMatchmakingCancel;

impl PacketRead for SMatchmakingCancel {
    fn read<R: Read>(_reader: &mut R) -> Result<Self, Error> {
        Ok(Self)
    }
}
