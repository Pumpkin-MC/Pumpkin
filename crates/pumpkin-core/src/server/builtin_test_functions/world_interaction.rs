use pumpkin_data::{
    Block, block_properties::OakTrapdoorLikeProperties, item::Item, item_stack::ItemStack,
};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::{
    GameMode, Hand, PermissionLvl,
    math::{position::BlockPos, vector3::Vector3},
};

use crate::server::server_test_manager::server_world;

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

    player.set_gamemode(GameMode::Survival)?;
    let log = BlockPos::new(3, 1, 2);
    test.set_block_default(&log, &Block::OAK_LOG).await?;
    player.set_held_item(Hand::Right, ItemStack::new(1, &Item::IRON_AXE))?;
    player.use_block(&test.absolute_pos(&log), Hand::Right)?;
    test.assert_block_present(&log, &Block::STRIPPED_OAK_LOG)
        .await?;
    test.assert_true(
        player.held_item(Hand::Right).get_damage() == 1,
        "Stripping a log did not wear down the held axe",
    )?;

    let support = BlockPos::new(2, 3, 2);
    test.set_block_default(&support, &Block::STONE).await?;
    player.set_held_item(Hand::Left, ItemStack::new(2, &Item::GOLD_BLOCK))?;
    let support = test.absolute_pos(&support);
    player.use_block(&support, Hand::Left)?;
    // The hit face is world north regardless of the test's rotation.
    test.assert_true(
        server_world(&test)?.get_block(&support.north()) == &Block::GOLD_BLOCK,
        "Block use did not place the held block on the north face",
    )?;
    test.assert_true(
        player.held_item(Hand::Left).item_count == 1,
        "Placing a block did not consume one item from the offhand",
    )?;
    test.succeed();
    Ok(())
}

pub(super) async fn test_shulker_box_open_blocked(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;
    let shulker = BlockPos::new(3, 1, 2);
    let above = shulker.up();
    test.set_block_default(&shulker, &Block::SHULKER_BOX)
        .await?;
    test.set_block_default(&above, &Block::STONE).await?;

    let player = test
        .spawn_simulated_player(
            "ShulkerTester",
            Vector3::new(1.5, 1.0, 2.5),
            GameMode::Survival,
        )
        .await?;
    let world = server_world(&test)?;
    let real_player = world.get_player_by_name(player.name()).ok_or_else(|| {
        pumpkin_gametest::GameTestError::World("Simulated player was not spawned".to_string())
    })?;
    let absolute_shulker = test.absolute_pos(&shulker);

    player.use_block(&absolute_shulker, Hand::Right)?;
    test.assert_true(
        real_player.open_container_pos.load() != Some(absolute_shulker),
        "Player opened a shulker box with a solid block above it",
    )?;

    test.set_block_default(&above, &Block::AIR).await?;
    player.use_block(&absolute_shulker, Hand::Right)?;
    test.assert_true(
        real_player.open_container_pos.load() == Some(absolute_shulker),
        "Player could not open a shulker box with clear space above it",
    )?;
    real_player.close_handled_screen();
    test.succeed();
    Ok(())
}
