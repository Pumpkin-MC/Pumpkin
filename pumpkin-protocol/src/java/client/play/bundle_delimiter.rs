use pumpkin_data::packet::clientbound::PLAY_BUNDLE_DELIMITER;
use pumpkin_macros::java_packet;
use serde::Serialize;

/// Delimiter for bundled packets that should be applied atomically.
///
/// The client accumulates all packets between two bundle delimiters
/// and applies them in a single tick, preventing visual glitches
/// from partial entity updates.
#[derive(Serialize)]
#[java_packet(PLAY_BUNDLE_DELIMITER)]
pub struct CBundleDelimiter;
