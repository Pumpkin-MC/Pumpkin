use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{
        AttachFace, DaylightDetectorLikeProperties, LeverLikeProperties, NoteBlockLikeProperties,
    },
    data_component_impl::EquipmentSlot,
    item_stack::ItemStack,
};
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult};
use pumpkin_util::{
    GameMode,
    math::{position::BlockPos, vector3::Vector3},
};

use crate::{
    block::{BlockHitResult, registry::BlockActionResult},
    entity::player::Player,
    server::server_test_manager::ServerSimulatedPlayer,
};

use super::build_stone_floor;

pub(super) fn use_block(player: &Arc<Player>, position: &BlockPos) -> GameTestResult<()> {
    let world = player.world();
    let server = world
        .server
        .upgrade()
        .ok_or_else(|| GameTestError::World("Server stopped".to_string()))?;
    let block = world.get_block(position);
    let hit = BlockHitResult {
        face: &BlockDirection::Up,
        cursor_pos: &Vector3::new(0.5, 0.5, 0.5),
    };
    let mut held = ItemStack::EMPTY.clone();
    let result = server.block_registry.use_with_item(
        block,
        player,
        position,
        &hit,
        &mut held,
        &EquipmentSlot::MAIN_HAND,
        &server,
        &world,
    );
    let result = if matches!(result, BlockActionResult::PassToDefaultBlockAction) {
        server
            .block_registry
            .on_use(block, player, position, &hit, &server, &world)
    } else {
        result
    };
    if result.consumes_action() {
        Ok(())
    } else {
        Err(GameTestError::World(format!(
            "Block use was not handled at {position:?}"
        )))
    }
}

pub(super) async fn test_block_tuning_and_invert(test: GameTestContext) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;

    let note_pos = BlockPos::new(1, 1, 2);
    let detector_pos = BlockPos::new(2, 1, 2);
    let lever_pos = BlockPos::new(3, 1, 2);
    test.set_block_default(&note_pos, &Block::NOTE_BLOCK)
        .await?;
    test.set_block_default(&detector_pos, &Block::DAYLIGHT_DETECTOR)
        .await?;
    let mut lever = LeverLikeProperties::default(&Block::LEVER);
    lever.face = AttachFace::Floor;
    lever.powered = false;
    test.set_block(&lever_pos, lever.to_state_id(&Block::LEVER))
        .await?;

    let simulated = test
        .spawn_simulated_player(
            "BlockUseTester",
            Vector3::new(2.5, 1.0, 1.5),
            GameMode::Survival,
        )
        .await?;
    let player = simulated
        .as_any()
        .downcast_ref::<ServerSimulatedPlayer>()
        .ok_or_else(|| GameTestError::World("Expected server simulated player".to_string()))?
        .player();

    use_block(player, &test.absolute_pos(&note_pos))?;
    use_block(player, &test.absolute_pos(&note_pos))?;
    test.assert_true(
        NoteBlockLikeProperties::from_state_id(test.block_state_id(&note_pos).await).note == 2,
        "Right clicks did not tune the note block twice",
    )?;

    use_block(player, &test.absolute_pos(&detector_pos))?;
    test.assert_true(
        DaylightDetectorLikeProperties::from_state_id(test.block_state_id(&detector_pos).await)
            .inverted,
        "Right click did not invert the daylight detector",
    )?;

    use_block(player, &test.absolute_pos(&lever_pos))?;
    test.assert_true(
        LeverLikeProperties::from_state_id(test.block_state_id(&lever_pos).await).powered,
        "Right click did not power the lever",
    )?;
    test.succeed();
    Ok(())
}
