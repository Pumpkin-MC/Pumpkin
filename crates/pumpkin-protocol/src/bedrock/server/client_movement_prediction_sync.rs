// Last verified for v2208

use pumpkin_macros::packet;

/// The client's movement prediction state (actor flags, bounding box, movement attributes,
/// flying). Only sent after server-authoritative movement corrections, so its body is not read.
#[packet(322)]
pub struct SClientMovementPredictionSync;
