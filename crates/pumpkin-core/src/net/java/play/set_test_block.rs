#[allow(clippy::wildcard_imports)]
use super::*;
use crate::block::entities::test_block::{TestBlockBlockEntity, TestBlockMode};
use pumpkin_protocol::java::server::play::TestBlockMode as PacketTestBlockMode;

const fn test_block_mode_from_packet(mode: PacketTestBlockMode) -> TestBlockMode {
    match mode {
        PacketTestBlockMode::Start => TestBlockMode::Start,
        PacketTestBlockMode::Log => TestBlockMode::Log,
        PacketTestBlockMode::Fail => TestBlockMode::Fail,
        PacketTestBlockMode::Accept => TestBlockMode::Accept,
    }
}

impl JavaClient {
    pub fn handle_set_test_block(&self, player: &Arc<Player>, packet: &SSetTestBlock<'_>) {
        if !player.is_creative() || player.permission_lvl.load() < PermissionLvl::Two {
            return;
        }

        let pos = packet.position;
        let world = player.world();
        let Some(block_entity) = world.get_block_entity(&pos) else {
            return;
        };
        let Some(test_block) = block_entity.as_any().downcast_ref::<TestBlockBlockEntity>() else {
            return;
        };

        test_block.set_mode(&world, test_block_mode_from_packet(packet.mode));
        test_block.set_message(packet.message.to_string());
        test_block.mark_dirty();

        world.update_block_entity(&block_entity);
        let new_state_id = world.get_block_state_id(&pos);
        world.register_block_change(pos, new_state_id);
    }
}
