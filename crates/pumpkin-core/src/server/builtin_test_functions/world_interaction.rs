use pumpkin_data::{Block, block_properties::OakTrapdoorLikeProperties};
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult};
use pumpkin_util::{
    GameMode, PermissionLvl,
    math::{position::BlockPos, vector3::Vector3},
};
use pumpkin_world::world::BlockFlags;

use crate::server::server_test_manager::ServerSimulatedPlayer;

use super::{block_tuning_and_invert::use_block, build_stone_floor};

pub(super) async fn test_world_block_use_and_break(test: GameTestContext) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let trapdoor_pos = BlockPos::new(2, 1, 3);
    let break_pos = BlockPos::new(1, 2, 2);
    test.set_block_default(&trapdoor_pos, &Block::OAK_TRAPDOOR)
        .await?;
    test.set_block_default(&break_pos, &Block::GOLD_BLOCK)
        .await?;

    let simulated = test
        .spawn_simulated_player(
            "WorldTester",
            Vector3::new(2.5, 1.0, 2.5),
            GameMode::Creative,
        )
        .await?;
    let player = simulated
        .as_any()
        .downcast_ref::<ServerSimulatedPlayer>()
        .ok_or_else(|| GameTestError::World("Expected server simulated player".to_string()))?
        .player();
    use_block(player, &test.absolute_pos(&trapdoor_pos))?;
    test.assert_true(
        OakTrapdoorLikeProperties::from_state_id(test.block_state_id(&trapdoor_pos).await).open,
        "Right click did not open the oak trapdoor",
    )?;

    let world = player.world();
    let server = world
        .server
        .upgrade()
        .ok_or_else(|| GameTestError::World("Server stopped".to_string()))?;
    let dispatcher = server.command_dispatcher.load();
    player.set_permission_lvl(&server, PermissionLvl::Four, &dispatcher);
    let world_break_pos = test.absolute_pos(&break_pos);
    let (block, state) = world.get_block_and_state(&world_break_pos);
    let broken = world.break_block(
        &world_break_pos,
        Some(player),
        BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_DROPS,
    );
    if broken.is_some() {
        server
            .block_registry
            .broken(&world, block, player, &world_break_pos, &server, state);
    }
    test.assert_true(
        broken.is_some() && world.get_block(&world_break_pos).id == Block::AIR.id,
        "Creative player did not break the gold block",
    )?;
    test.assert_item_entity_count("minecraft:gold_block", &break_pos, 1.0, 0)
        .await?;
    test.succeed();
    Ok(())
}
