use pumpkin_data::Block;
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::{
    GameMode,
    math::{position::BlockPos, vector3::Vector3},
};

pub(super) async fn test_boats_pick_up_nearby_mobs(helper: GameTestContext) -> GameTestResult<()> {
    helper.set_dimensions(5, 4, 8).await?;
    helper.build_floor(5, 8, &Block::STONE).await?;
    for (boat, z) in [
        ("minecraft:oak_boat", 1.5),
        ("minecraft:oak_chest_boat", 5.5),
    ] {
        helper.spawn_entity(boat, Vector3::new(1.5, 1.0, z)).await?;
        for _ in 0..3 {
            helper
                .spawn_entity("minecraft:pig", Vector3::new(2.7, 1.0, z))
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

pub(super) async fn test_boats_reject_submerged_mob_pickup(
    helper: GameTestContext,
) -> GameTestResult<()> {
    helper.set_dimensions(5, 4, 8).await?;
    helper.build_floor(5, 8, &Block::STONE).await?;
    let origin = BlockPos::new(0, 1, 0);
    for command in [
        "fill ~ ~ ~ ~4 ~ ~7 minecraft:stone",
        "fill ~1 ~ ~1 ~3 ~ ~6 minecraft:water",
        "fill ~1 ~1 ~1 ~3 ~1 ~3 minecraft:stone",
    ] {
        helper.execute_command(&origin, command).await?;
    }
    for z in [2, 5] {
        let seat = BlockPos::new(2, 1, z);
        // The boat's eyes exactly touch the ceiling's block boundary.
        helper
            .execute_command(
                &seat,
                "summon minecraft:oak_boat ~ ~0.4375 ~ {NoGravity:1b}",
            )
            .await?;
        helper
            .execute_command(
                &seat,
                "summon minecraft:pig ~1.05 ~0.4375 ~ {Age:-24000,NoAI:1b,NoGravity:1b}",
            )
            .await?;
    }

    helper
        .start_sequence()
        .then_idle(5)
        .then_execute(move |helper| async move {
            let ceiling = BlockPos::new(2, 1, 2);
            let valid = helper
                .execute_command(
                    &ceiling,
                    "execute if block ~ ~ ~ minecraft:water[level=0] if block ~ ~1 ~ minecraft:stone",
                )
                .await?;
            helper.assert_true(valid == 1, "Expected source water below a solid ceiling")?;
            for (z, expected) in [(2, 0), (5, 1)] {
                let passengers = helper
                    .execute_command(
                        &BlockPos::new(2, 1, z),
                        "execute as @e[type=minecraft:oak_boat,distance=..1] on passengers if entity @s[type=minecraft:pig]",
                    )
                    .await?;
                helper.assert_true(
                    passengers == expected,
                    format!("Expected {expected} pigs riding boat at z={z}, found {passengers}"),
                )?;
            }
            helper
                .execute_command(
                    &BlockPos::new(0, 1, 0),
                    "fill ~1 ~1 ~1 ~3 ~1 ~3 minecraft:air",
                )
                .await?;
            Ok(())
        })
        .then_idle(1)
        .then_execute(move |helper| async move {
            let passengers = helper
                .execute_command(
                    &BlockPos::new(2, 1, 2),
                    "execute as @e[type=minecraft:oak_boat,distance=..1] on passengers if entity @s[type=minecraft:pig]",
                )
                .await?;
            helper.assert_true(passengers == 1, "Pig must board after removing the ceiling")
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_riding_shulker_keeps_attach_face(
    helper: GameTestContext,
) -> GameTestResult<()> {
    helper.set_dimensions(5, 5, 5).await?;
    let seat = BlockPos::new(1, 2, 2);
    helper
        .set_block_default(&BlockPos::new(2, 2, 2), &Block::STONE)
        .await?;
    helper
        .execute_command(
            &seat,
            "summon minecraft:oak_boat ~ ~ ~ {Passengers:[{id:\"minecraft:shulker\",AttachFace:5b}]}",
        )
        .await?;
    let loaded = helper
        .execute_command(
            &seat,
            "execute as @e[type=minecraft:oak_boat,distance=..3] on passengers if entity @s[type=minecraft:shulker,nbt={AttachFace:0b}]",
        )
        .await?;
    helper.assert_true(loaded == 1, "Loaded shulker passenger must attach down")?;
    helper
        .execute_command(
            &seat,
            "ride @e[type=minecraft:shulker,distance=..3,limit=1] dismount",
        )
        .await?;
    helper
        .execute_command(&seat, "tp @e[type=minecraft:shulker,distance=..3] ~ ~ ~")
        .await?;
    helper
        .execute_command(
            &seat,
            "data merge entity @e[type=minecraft:shulker,distance=..3,limit=1] {AttachFace:5b}",
        )
        .await?;
    let attached = helper
        .execute_command(
            &seat,
            "execute if entity @e[type=minecraft:shulker,distance=..3,nbt={AttachFace:5b}]",
        )
        .await?;
    helper.assert_true(
        attached == 1,
        "Shulker must initially attach to the east wall",
    )?;
    helper
        .execute_command(
            &seat,
            "ride @e[type=minecraft:shulker,distance=..3,limit=1] mount @e[type=minecraft:oak_boat,distance=..3,limit=1]",
        )
        .await?;

    let mounted = helper
        .execute_command(
            &seat,
            "execute as @e[type=minecraft:oak_boat,distance=..3] on passengers if entity @s[type=minecraft:shulker,nbt={AttachFace:0b}]",
        )
        .await?;
    helper.assert_true(mounted == 1, "Shulker must attach down during /ride mount")?;

    helper
        .start_sequence()
        .then_idle(5)
        .then_execute(move |helper| async move {
            let passengers = helper
                .execute_command(
                    &seat,
                    "execute as @e[type=minecraft:oak_boat,distance=..3] on passengers if entity @s[type=minecraft:shulker,nbt={AttachFace:0b}]",
                )
                .await?;
            helper.assert_true(passengers == 1, "Riding shulker must remain attached down")?;
            helper
                .execute_command(
                    &seat,
                    "ride @e[type=minecraft:shulker,distance=..3,limit=1] dismount",
                )
                .await?;
            helper
                .execute_command(&seat, "kill @e[type=minecraft:oak_boat,distance=..3]")
                .await?;
            helper
                .execute_command(
                    &seat,
                    "tp @e[type=minecraft:shulker,distance=..3] ~ ~ ~",
                )
                .await?;
            Ok(())
        })
        .then_idle(1)
        .then_execute(move |helper| async move {
            let attached = helper
                .execute_command(
                    &seat,
                    "execute if entity @e[type=minecraft:shulker,distance=..3,nbt={AttachFace:5b}]",
                )
                .await?;
            helper.assert_true(attached == 1, "Dismounted shulker must attach to the east wall")
        })
        .then_succeed();
    Ok(())
}

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
