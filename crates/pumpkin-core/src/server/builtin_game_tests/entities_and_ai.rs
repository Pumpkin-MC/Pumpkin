use pumpkin_data::Block;
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use super::build_stone_floor;

pub(super) async fn test_boats_pick_up_nearby_mobs(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 8).await?;
    for (boat, z) in [
        ("minecraft:oak_boat", 1.5),
        ("minecraft:oak_chest_boat", 5.5),
    ] {
        helper
            .spawn_entity_at(boat, Vector3::new(1.5, 1.0, z))
            .await?;
        for _ in 0..3 {
            helper
                .spawn_entity_at("minecraft:pig", Vector3::new(2.7, 1.0, z))
                .await?;
        }
    }

    helper.succeed_when(move |helper| async move {
        for (boat, z, expected) in [("oak_boat", 1, 2), ("oak_chest_boat", 5, 1)] {
            let passengers = helper
                .execute_command(
                    &BlockPos::new(1, 1, z),
                    &format!(
                        "execute as @e[type=minecraft:{boat},distance=..2] on passengers if entity @s[type=minecraft:pig]"
                    ),
                )
                .await?;
            helper.assert_true(
                passengers == expected,
                format!("Expected {expected} pigs riding {boat}, found {passengers}"),
            )?;
        }
        Ok(())
    })?;
    Ok(())
}

pub(super) async fn test_item_entities_merge(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 4).await?;

    for x in 0..4 {
        for z in 0..4 {
            if x == 0 || x == 3 || z == 0 || z == 3 {
                helper
                    .set_block_default(&BlockPos::new(x, 1, z), &Block::STONE)
                    .await?;
            }
        }
    }

    helper
        .spawn_item_at("minecraft:diamond", 10, Vector3::new(1.5, 1.1, 1.5))
        .await?;
    helper
        .spawn_item_at("minecraft:diamond", 15, Vector3::new(1.5, 1.1, 1.5))
        .await?;

    let center = BlockPos::new(1, 1, 1);
    helper.succeed_when(move |helper| async move {
        helper.assert_entities_present("minecraft:item", 1).await?;
        helper
            .assert_item_entity_count("minecraft:diamond", &center, 2.0, 25)
            .await
    })?;
    Ok(())
}

pub(super) async fn test_zombie_pathfinds_to_villager(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 8).await?;

    for x in 0..5 {
        for z in 0..8 {
            if x == 0 || x == 4 || z == 0 || z == 7 {
                helper
                    .set_block_default(&BlockPos::new(x, 1, z), &Block::STONE)
                    .await?;
                helper
                    .set_block_default(&BlockPos::new(x, 2, z), &Block::STONE)
                    .await?;
            }
        }
    }

    helper
        .spawn_entity_at("minecraft:villager", Vector3::new(2.5, 1.0, 6.0))
        .await?;
    helper
        .spawn_entity_at("minecraft:husk", Vector3::new(2.5, 1.0, 1.5))
        .await?;

    helper.succeed_when(move |helper| async move {
        helper
            .assert_entity_present_in_box(
                "minecraft:husk",
                BlockPos::new(1, 1, 4),
                BlockPos::new(4, 3, 7),
            )
            .await
    })?;
    Ok(())
}
