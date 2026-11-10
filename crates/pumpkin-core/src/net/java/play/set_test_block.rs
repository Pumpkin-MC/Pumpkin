#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_set_test_block(&self, player: &Arc<Player>, packet: &SSetTestBlock<'_>) {
        if !player.has_client_loaded()
            || !player.is_creative()
            || player.permission_lvl.load() < PermissionLvl::Two
        {
            return;
        }
        player.update_last_action_time();
        let world = player.world();
        let Some(entity) = world.get_block_entity(&packet.position) else {
            return;
        };
        let Some(test_block) = entity
            .as_any()
            .downcast_ref::<crate::block::entities::test_block::TestBlockBlockEntity>(
        ) else {
            return;
        };
        let state = world.get_block_state_id(&packet.position);
        if pumpkin_data::Block::from_state_id(state) != &pumpkin_data::Block::TEST_BLOCK {
            return;
        }
        let (mode, name) = match packet.mode {
            pumpkin_protocol::java::server::play::TestBlockMode::Start => (
                pumpkin_data::block_properties::TestBlockMode::Start,
                "start",
            ),
            pumpkin_protocol::java::server::play::TestBlockMode::Log => {
                (pumpkin_data::block_properties::TestBlockMode::Log, "log")
            }
            pumpkin_protocol::java::server::play::TestBlockMode::Fail => {
                (pumpkin_data::block_properties::TestBlockMode::Fail, "fail")
            }
            pumpkin_protocol::java::server::play::TestBlockMode::Accept => (
                pumpkin_data::block_properties::TestBlockMode::Accept,
                "accept",
            ),
        };
        *test_block
            .mode
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = name.to_string();
        *test_block
            .message
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = packet.message.to_string();
        let mut properties =
            pumpkin_data::block_properties::TestBlockLikeProperties::from_state_id(state);
        properties.mode = mode;
        world.set_block_state(
            &packet.position,
            properties.to_state_id(&pumpkin_data::Block::TEST_BLOCK),
            pumpkin_world::world::BlockFlags::NOTIFY_LISTENERS,
        );
        world.update_block_entity(&entity);
    }
}
