#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::java::server::play::SSetStructureBlock;

impl JavaClient {
    pub fn handle_set_structure_block(&self, player: &Player, packet: &SSetStructureBlock<'_>) {
        if player.permission_lvl.load() < PermissionLvl::Two {
            return;
        }

        debug!(
            "Player {} set structure block at {:?}, name: {}, mode: {}",
            player.gameprofile.name, packet.location, packet.name, packet.mode.0
        );
    }
}
