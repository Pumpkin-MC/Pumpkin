//! # DTO (Data Transfer Object) — Multi-Version Protocol Translation
//!
//! This module provides version-aware packet translation for supporting
//! older Minecraft clients connecting to Pumpkin's 1.21.11 server.
//!
//! ## Architecture
//!
//! Internal game state is always 1.21.11 (canonical model). The DTO layer
//! sits between internal state and the wire format, translating packets
//! for older protocol versions.
//!
//! ```text
//! Internal Game State (1.21.11)
//!          │
//!    VersionAdapter
//!     ├── V1_21: passthrough (identity)
//!     └── V1_18: translate fields + suppress new packets
//!          │
//!    Wire Format (per-version encoding)
//! ```
//!
//! ## Integration Point
//!
//! The adapter is selected per-client based on the protocol version
//! sent during handshake. It intercepts packet writing in
//! `JavaClient::write_packet` where the `PacketId::to_id(version)` and
//! `ClientPacket::write_packet_data(write, &version)` calls already exist.
//!
//! ## Tiered Rollback (ARCH-016/017)
//!
//! - Tier 0 (current): 1.21.7–1.21.11 — packet ID mapping only
//! - Tier 1: 1.18.2 — packet ID remapping + minor field changes
//! - Tier 2: 1.16.5 — + Config state bypass + item NBT + dimension codec
//! - Tier 3: 1.14.x — + chunk format v1
//! - Tier 4: 1.12.2 — + pre-Flattening block IDs (stretch)

use pumpkin_util::version::MinecraftVersion;

/// Determines whether a packet should be sent to a client at a given version,
/// and whether any translation is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketAction {
    /// Send the packet as-is (no translation needed).
    Send,
    /// Suppress the packet entirely (doesn't exist in this version).
    Suppress,
    /// The packet needs version-specific translation before sending.
    /// The caller should use the VersionAdapter to transform it.
    Translate,
}

/// Returns the appropriate `PacketAction` for a clientbound packet ID
/// targeting a specific client version.
///
/// This is the fast path — most packets can be sent as-is or suppressed
/// based solely on whether the packet ID exists in the target version
/// (i.e., `PacketId::to_id(version) != -1`).
#[must_use]
pub fn packet_action_for(packet_id_in_version: i32) -> PacketAction {
    if packet_id_in_version == -1 {
        PacketAction::Suppress
    } else {
        PacketAction::Send
    }
}

/// Returns true if the given version has the Configuration connection state.
/// Added in 1.20.2 (protocol 764). Versions before this go Login → Play directly.
#[must_use]
pub const fn has_config_state(version: MinecraftVersion) -> bool {
    version.protocol_version() >= MinecraftVersion::V_1_20_2.protocol_version()
}

/// Returns true if the given version uses item components (1.20.5+, protocol 766+).
/// Older versions use NBT-based item data.
#[must_use]
pub const fn has_item_components(version: MinecraftVersion) -> bool {
    version.protocol_version() >= MinecraftVersion::V_1_20_5.protocol_version()
}

/// Returns true if the given version uses the extended world height (-64 to 320).
/// Added in 1.18 (protocol 757). Older versions use 0 to 255.
#[must_use]
pub const fn has_extended_world_height(version: MinecraftVersion) -> bool {
    version.protocol_version() >= MinecraftVersion::V_1_18.protocol_version()
}

/// Returns true if the given version supports signed chat (1.19+, protocol 759+).
#[must_use]
pub const fn has_signed_chat(version: MinecraftVersion) -> bool {
    version.protocol_version() >= MinecraftVersion::V_1_19.protocol_version()
}

/// Returns true if the given version supports chunk batch packets (1.20.2+, protocol 764+).
#[must_use]
pub const fn has_chunk_batching(version: MinecraftVersion) -> bool {
    version.protocol_version() >= MinecraftVersion::V_1_20_2.protocol_version()
}

/// Returns true if the given version supports bundle delimiter packets (1.19.4+, protocol 762+).
#[must_use]
pub const fn has_bundle_delimiter(version: MinecraftVersion) -> bool {
    version.protocol_version() >= MinecraftVersion::V_1_19_4.protocol_version()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_action_send() {
        assert_eq!(packet_action_for(42), PacketAction::Send);
        assert_eq!(packet_action_for(0), PacketAction::Send);
    }

    #[test]
    fn test_packet_action_suppress() {
        assert_eq!(packet_action_for(-1), PacketAction::Suppress);
    }

    #[test]
    fn test_has_config_state() {
        // 1.20.2 (764) introduced Config state
        assert!(has_config_state(MinecraftVersion::V_1_20_2));
        assert!(has_config_state(MinecraftVersion::V_1_21_11));
        // 1.20.3 (765) also has it
        assert!(has_config_state(MinecraftVersion::V_1_20_3));
        // 1.20 (763) does NOT
        assert!(!has_config_state(MinecraftVersion::V_1_20));
        // 1.18.2 does NOT
        assert!(!has_config_state(MinecraftVersion::V_1_18_2));
        // 1.16.4 does NOT
        assert!(!has_config_state(MinecraftVersion::V_1_16_4));
    }

    #[test]
    fn test_has_item_components() {
        // 1.20.5 (766) introduced item components
        assert!(has_item_components(MinecraftVersion::V_1_20_5));
        assert!(has_item_components(MinecraftVersion::V_1_21_11));
        // 1.20.3 (765) does NOT
        assert!(!has_item_components(MinecraftVersion::V_1_20_3));
        assert!(!has_item_components(MinecraftVersion::V_1_18_2));
    }

    #[test]
    fn test_has_extended_world_height() {
        // 1.18 (757) introduced -64 to 320
        assert!(has_extended_world_height(MinecraftVersion::V_1_18));
        assert!(has_extended_world_height(MinecraftVersion::V_1_18_2));
        assert!(has_extended_world_height(MinecraftVersion::V_1_21_11));
        // 1.17.1 (756) does NOT
        assert!(!has_extended_world_height(MinecraftVersion::V_1_17_1));
        assert!(!has_extended_world_height(MinecraftVersion::V_1_16_4));
    }

    #[test]
    fn test_has_signed_chat() {
        assert!(has_signed_chat(MinecraftVersion::V_1_19));
        assert!(has_signed_chat(MinecraftVersion::V_1_21_11));
        assert!(!has_signed_chat(MinecraftVersion::V_1_18_2));
    }

    #[test]
    fn test_has_chunk_batching() {
        assert!(has_chunk_batching(MinecraftVersion::V_1_20_2));
        assert!(!has_chunk_batching(MinecraftVersion::V_1_20));
    }

    #[test]
    fn test_has_bundle_delimiter() {
        assert!(has_bundle_delimiter(MinecraftVersion::V_1_19_4));
        assert!(!has_bundle_delimiter(MinecraftVersion::V_1_19_3));
    }
}
