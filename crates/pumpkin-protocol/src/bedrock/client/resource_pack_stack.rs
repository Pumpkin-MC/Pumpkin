// Last verified for v2169

use crate::{bedrock::client::start_game::Experiments, serial::PacketWrite};
use pumpkin_macros::packet;

/// Identifies a specific pack instance and sub-pack layer in the active resource pack stack.
#[derive(PacketWrite)]
pub struct PackInstanceId {
    /// UUID string of the resource pack.
    pub pack_id: String,
    /// Semantic version string.
    pub version: String,
    /// Sub-pack configuration name.
    pub sub_pack_name: String,
}

/// Sent by the server to define the order in which resource and behavior packs are applied by the client.
#[derive(PacketWrite)]
#[packet(7)]
pub struct CResourcePackStackPacket {
    /// If true, textures from the resource packs are strictly required to join.
    pub texture_pack_required: bool,
    /// Ordered list of packs to apply in bottom-to-top priority.
    pub texture_pack_list: Vec<PackInstanceId>,
    /// Base vanilla version of the game.
    pub base_game_version: String,
    /// Experimental gameplay feature toggles enabled for this world.
    pub experiments: Experiments,
    /// Whether to load editor mode packs.
    pub include_editor_packs: bool,
}
