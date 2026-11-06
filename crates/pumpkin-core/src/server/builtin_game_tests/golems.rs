use pumpkin_data::Block;
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use super::build_stone_floor;

pub(super) async fn test_copper_golem_collects_items_at_path_end(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    for x in 0..5 {
        for z in 0..5 {
            if x == 0 || x == 4 || z == 0 || z == 4 {
                for y in 1..3 {
                    helper
                        .set_block_default(&BlockPos::new(x, y, z), &Block::STONE)
                        .await?;
                }
            }
        }
    }
    let source = BlockPos::new(1, 1, 2);
    helper
        .set_block_default(&source, &Block::WAXED_COPPER_CHEST)
        .await?;
    helper
        .set_container_item(&source, 0, "minecraft:iron_ingot", 16)
        .await?;
    helper
        .spawn_entity_at("minecraft:copper_golem", Vector3::new(3.5, 1.0, 2.5))
        .await?;

    helper
        .succeed_when(move |helper| async move { helper.assert_container_empty(&source).await })?;
    Ok(())
}

pub(super) async fn test_snow_golem_pattern_spawns(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let base_pos = BlockPos::new(2, 1, 2);
    let mid_pos = BlockPos::new(2, 2, 2);
    let head_pos = BlockPos::new(2, 3, 2);

    helper
        .set_block_default(&base_pos, &Block::SNOW_BLOCK)
        .await?;
    helper
        .set_block_default(&mid_pos, &Block::SNOW_BLOCK)
        .await?;
    helper
        .set_block_default(&head_pos, &Block::CARVED_PUMPKIN)
        .await?;

    helper.assert_block_present(&base_pos, &Block::AIR).await?;
    helper.assert_block_present(&mid_pos, &Block::AIR).await?;
    helper.assert_block_present(&head_pos, &Block::AIR).await?;
    helper.assert_entity_present("minecraft:snow_golem").await?;

    helper.succeed();
    Ok(())
}

pub(super) async fn test_iron_golem_pattern_spawns(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let base_pos = BlockPos::new(2, 1, 2);
    let mid_pos = BlockPos::new(2, 2, 2);
    let left_arm = BlockPos::new(1, 2, 2);
    let right_arm = BlockPos::new(3, 2, 2);
    let head_pos = BlockPos::new(2, 3, 2);

    helper
        .set_block_default(&base_pos, &Block::IRON_BLOCK)
        .await?;
    helper
        .set_block_default(&mid_pos, &Block::IRON_BLOCK)
        .await?;
    helper
        .set_block_default(&left_arm, &Block::IRON_BLOCK)
        .await?;
    helper
        .set_block_default(&right_arm, &Block::IRON_BLOCK)
        .await?;
    helper
        .set_block_default(&head_pos, &Block::CARVED_PUMPKIN)
        .await?;

    for pos in [base_pos, mid_pos, left_arm, right_arm, head_pos] {
        helper.assert_block_present(&pos, &Block::AIR).await?;
    }
    helper.assert_entity_present("minecraft:iron_golem").await?;

    helper.succeed();
    Ok(())
}

pub(super) async fn test_wither_pattern_spawns(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let base_pos = BlockPos::new(2, 1, 2);
    let mid_pos = BlockPos::new(2, 2, 2);
    let left_arm = BlockPos::new(1, 2, 2);
    let right_arm = BlockPos::new(3, 2, 2);
    let left_skull = BlockPos::new(1, 3, 2);
    let right_skull = BlockPos::new(3, 3, 2);
    let center_skull = BlockPos::new(2, 3, 2);

    for pos in [base_pos, mid_pos, left_arm, right_arm] {
        helper.set_block_default(&pos, &Block::SOUL_SAND).await?;
    }
    for pos in [left_skull, right_skull, center_skull] {
        helper
            .set_block_default(&pos, &Block::WITHER_SKELETON_SKULL)
            .await?;
    }

    for pos in [
        base_pos,
        mid_pos,
        left_arm,
        right_arm,
        left_skull,
        right_skull,
        center_skull,
    ] {
        helper.assert_block_present(&pos, &Block::AIR).await?;
    }
    helper.assert_entity_present("minecraft:wither").await?;

    helper.succeed();
    Ok(())
}
