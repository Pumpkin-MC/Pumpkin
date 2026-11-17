use pumpkin_data::{Block, block_properties::OakTrapdoorLikeProperties};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::{
    GameMode, Hand, PermissionLvl,
    math::{position::BlockPos, vector3::Vector3},
};

pub(super) async fn test_world_block_use_and_break(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;
    let trapdoor_pos = BlockPos::new(2, 1, 3);
    let break_pos = BlockPos::new(1, 2, 2);
    test.set_block_default(&trapdoor_pos, &Block::OAK_TRAPDOOR)
        .await?;
    test.set_block_default(&break_pos, &Block::GOLD_BLOCK)
        .await?;

    let player = test
        .spawn_simulated_player(
            "WorldTester",
            Vector3::new(2.5, 1.0, 2.5),
            GameMode::Creative,
        )
        .await?;
    player.use_block(&test.absolute_pos(&trapdoor_pos), Hand::Right)?;
    test.assert_true(
        OakTrapdoorLikeProperties::from_state_id(test.block_state_id(&trapdoor_pos).await).open,
        "Right click did not open the oak trapdoor",
    )?;

    player.set_permission_level(PermissionLvl::Four)?;
    let world_break_pos = test.absolute_pos(&break_pos);
    let broken = player.break_block(&world_break_pos)?;
    test.assert_true(
        broken && test.block_state_id(&break_pos).await == Block::AIR.default_state.id,
        "Creative player did not break the gold block",
    )?;
    test.assert_item_entity_count("minecraft:gold_block", &break_pos, 1.0, 0)
        .await?;
    test.succeed();
    Ok(())
}
