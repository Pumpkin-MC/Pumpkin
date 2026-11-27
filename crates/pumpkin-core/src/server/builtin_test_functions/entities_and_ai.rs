use pumpkin_data::Block;
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::{
    GameMode,
    math::{position::BlockPos, vector3::Vector3},
};

pub(super) async fn test_item_entities_merge(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(4, 4, 4).await?;
    test.build_floor(4, 4, &Block::STONE).await?;

    for x in 0..4 {
        for z in 0..4 {
            if x == 0 || x == 3 || z == 0 || z == 3 {
                test.set_block_default(&BlockPos::new(x, 1, z), &Block::STONE)
                    .await?;
            }
        }
    }

    test.spawn_item_at("minecraft:diamond", 10, Vector3::new(1.5, 1.1, 1.5))
        .await?;
    test.spawn_item_at("minecraft:diamond", 15, Vector3::new(1.5, 1.1, 1.5))
        .await?;

    let center = BlockPos::new(1, 1, 1);
    test.succeed_when(move |test| async move {
        test.assert_entities_present("minecraft:item", 1).await?;
        test.assert_item_entity_count("minecraft:diamond", &center, 2.0, 25)
            .await
    })?;
    Ok(())
}

pub(super) async fn test_zombie_pathfinds_to_villager(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 4, 8).await?;
    test.build_floor(5, 8, &Block::STONE).await?;

    for x in 0..5 {
        for z in 0..8 {
            if x == 0 || x == 4 || z == 0 || z == 7 {
                test.set_block_default(&BlockPos::new(x, 1, z), &Block::STONE)
                    .await?;
                test.set_block_default(&BlockPos::new(x, 2, z), &Block::STONE)
                    .await?;
            }
        }
    }

    test.spawn_entity("minecraft:villager", Vector3::new(2.5, 1.0, 6.0))
        .await?;
    test.spawn_entity("minecraft:husk", Vector3::new(2.5, 1.0, 1.5))
        .await?;

    test.succeed_when(move |test| async move {
        test.assert_entity_present_in_box(
            "minecraft:husk",
            BlockPos::new(1, 1, 4),
            BlockPos::new(4, 3, 7),
        )
        .await
    })?;
    Ok(())
}

pub(super) async fn test_villager_stays_when_player_is_far(
    test: GameTestContext,
) -> GameTestResult<()> {
    assert_stays_when_player_is_far(test, "minecraft:villager").await
}

pub(super) async fn test_wandering_trader_stays_when_player_is_far(
    test: GameTestContext,
) -> GameTestResult<()> {
    assert_stays_when_player_is_far(test, "minecraft:wandering_trader").await
}

async fn assert_stays_when_player_is_far(
    test: GameTestContext,
    entity: &'static str,
) -> GameTestResult<()> {
    test.set_dimensions(5, 4, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;

    test.execute_command(
        &BlockPos::new(2, 1, 2),
        &format!("summon {entity} ~ ~ ~ {{NoAI:1b}}"),
    )
    .await?;
    test.spawn_simulated_player(
        "FarPlayer",
        Vector3::new(142.5, 1.0, 2.5),
        GameMode::Survival,
    )
    .await?;

    test.start_sequence()
        .then_idle(10)
        .then_execute(move |test| async move { test.assert_entity_present(entity).await })
        .then_succeed();
    Ok(())
}
