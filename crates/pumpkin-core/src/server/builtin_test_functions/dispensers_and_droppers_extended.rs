use pumpkin_data::{
    Block, BlockStateId,
    block_properties::{
        BeeNestLikeProperties, CampfireLikeProperties, DispenserLikeProperties, Facing,
        HopperLikeProperties,
    },
};
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

fn east_dispenser_id(block: &Block) -> BlockStateId {
    let mut props = DispenserLikeProperties::default(block);
    props.facing = Facing::East;
    props.to_state_id(block)
}

pub(super) async fn test_dispenser_lava_bucket_place_and_pickup(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let disp = BlockPos::new(1, 1, 2);
    let target = BlockPos::new(2, 1, 2);
    let power = BlockPos::new(1, 2, 2);

    test.set_block(&disp, east_dispenser_id(&Block::DISPENSER))
        .await?;
    for wall in [
        BlockPos::new(3, 1, 2),
        BlockPos::new(2, 1, 1),
        BlockPos::new(2, 1, 3),
    ] {
        test.set_block_default(&wall, &Block::STONE).await?;
    }
    test.set_container_item(&disp, 0, "minecraft:lava_bucket", 1)
        .await?;
    test.set_block_default(&power, &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(5)
        .then_execute(move |test| async move {
            test.assert_block_present(&target, &Block::LAVA).await?;
            test.assert_container_contains_count(&disp, "minecraft:bucket", 1)
                .await?;
            test.set_block_default(&power, &Block::AIR).await
        })
        .then_idle(2)
        .then_execute(move |test| async move {
            test.set_block_default(&power, &Block::REDSTONE_BLOCK).await
        })
        .then_idle(5)
        .then_execute(move |test| async move {
            test.assert_block_present(&target, &Block::AIR).await?;
            test.assert_container_contains_count(&disp, "minecraft:lava_bucket", 1)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_dispenser_powder_snow_bucket_place_and_pickup(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let disp = BlockPos::new(1, 1, 2);
    let target = BlockPos::new(2, 1, 2);
    let power = BlockPos::new(1, 2, 2);

    test.set_block(&disp, east_dispenser_id(&Block::DISPENSER))
        .await?;
    test.set_container_item(&disp, 0, "minecraft:powder_snow_bucket", 1)
        .await?;
    test.set_block_default(&power, &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(5)
        .then_execute(move |test| async move {
            test.assert_block_present(&target, &Block::POWDER_SNOW)
                .await?;
            test.assert_container_contains_count(&disp, "minecraft:bucket", 1)
                .await?;
            test.set_block_default(&power, &Block::AIR).await
        })
        .then_idle(2)
        .then_execute(move |test| async move {
            test.set_block_default(&power, &Block::REDSTONE_BLOCK).await
        })
        .then_idle(5)
        .then_execute(move |test| async move {
            test.assert_block_present(&target, &Block::AIR).await?;
            test.assert_container_contains_count(&disp, "minecraft:powder_snow_bucket", 1)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_dispenser_shears_full_beehive_drops_honeycomb(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 4).await?;
    let disp = BlockPos::new(1, 1, 1);
    let hive = BlockPos::new(2, 1, 1);
    test.set_block(&disp, east_dispenser_id(&Block::DISPENSER))
        .await?;
    let mut hive_props = BeeNestLikeProperties::default(&Block::BEEHIVE);
    hive_props.honey_level = 5;
    test.set_block(&hive, hive_props.to_state_id(&Block::BEEHIVE))
        .await?;
    test.set_container_item(&disp, 0, "minecraft:shears", 1)
        .await?;
    test.set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(5)
        .then_execute(move |test| async move {
            let level =
                BeeNestLikeProperties::from_state_id(test.block_state_id(&hive).await).honey_level;
            test.assert_true(
                level == 0,
                format!("Expected sheared beehive level 0, got {level}"),
            )?;
            test.assert_entity_present("minecraft:item").await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_dispenser_glass_bottle_fills_water_from_source(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let disp = BlockPos::new(1, 1, 2);
    let water = BlockPos::new(2, 1, 2);
    test.set_block(&disp, east_dispenser_id(&Block::DISPENSER))
        .await?;
    for wall in [
        BlockPos::new(3, 1, 2),
        BlockPos::new(2, 1, 1),
        BlockPos::new(2, 1, 3),
    ] {
        test.set_block_default(&wall, &Block::STONE).await?;
    }
    test.set_block_default(&water, &Block::WATER).await?;
    test.set_container_item(&disp, 0, "minecraft:glass_bottle", 1)
        .await?;
    test.set_block_default(&BlockPos::new(1, 2, 2), &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(5)
        .then_execute(move |test| async move {
            test.assert_container_contains_count(&disp, "minecraft:potion", 1)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_dispenser_flint_and_steel_primes_tnt(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 4).await?;
    let disp = BlockPos::new(1, 1, 1);
    let tnt = BlockPos::new(2, 1, 1);
    test.set_block(&disp, east_dispenser_id(&Block::DISPENSER))
        .await?;
    test.set_block_default(&tnt, &Block::TNT).await?;
    test.set_container_item(&disp, 0, "minecraft:flint_and_steel", 1)
        .await?;
    test.set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(5)
        .then_execute(move |test| async move {
            test.assert_block_present(&tnt, &Block::AIR).await?;
            test.assert_entity_present("minecraft:tnt").await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_dispenser_flint_and_steel_lights_campfire(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 4).await?;
    let disp = BlockPos::new(1, 1, 1);
    let campfire = BlockPos::new(2, 1, 1);
    test.set_block(&disp, east_dispenser_id(&Block::DISPENSER))
        .await?;
    let mut cf_props = CampfireLikeProperties::default(&Block::CAMPFIRE);
    cf_props.lit = false;
    test.set_block(&campfire, cf_props.to_state_id(&Block::CAMPFIRE))
        .await?;
    test.set_container_item(&disp, 0, "minecraft:flint_and_steel", 1)
        .await?;
    test.set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(5)
        .then_execute(move |test| async move {
            let lit =
                CampfireLikeProperties::from_state_id(test.block_state_id(&campfire).await).lit;
            test.assert_true(
                lit,
                "Expected flint_and_steel in dispenser to light unlit campfire",
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_dispenser_armor_stand_spawns_entity(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 4).await?;
    let disp = BlockPos::new(1, 1, 1);
    test.set_block(&disp, east_dispenser_id(&Block::DISPENSER))
        .await?;
    test.set_container_item(&disp, 0, "minecraft:armor_stand", 1)
        .await?;
    test.set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(5)
        .then_execute(move |test| async move {
            test.assert_container_empty(&disp).await?;
            test.assert_entity_present("minecraft:armor_stand").await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_dispenser_tnt_spawns_primed_tnt(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 4).await?;
    let disp = BlockPos::new(1, 1, 1);
    test.set_block(&disp, east_dispenser_id(&Block::DISPENSER))
        .await?;
    test.set_container_item(&disp, 0, "minecraft:tnt", 1)
        .await?;
    test.set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(5)
        .then_execute(move |test| async move {
            test.assert_container_empty(&disp).await?;
            test.assert_entity_present("minecraft:tnt").await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_dispenser_minecart_places_on_rail(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 4).await?;
    let disp = BlockPos::new(1, 1, 1);
    let rail = BlockPos::new(2, 1, 1);
    test.set_block(&disp, east_dispenser_id(&Block::DISPENSER))
        .await?;
    test.set_block_default(&rail, &Block::RAIL).await?;
    test.set_container_item(&disp, 0, "minecraft:minecart", 1)
        .await?;
    test.set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(5)
        .then_execute(move |test| async move {
            test.assert_container_empty(&disp).await?;
            test.assert_entity_present("minecraft:minecart").await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_dispenser_boat_places_on_water(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 6, 5).await?;
    let disp = BlockPos::new(1, 2, 2);
    test.set_block(&disp, east_dispenser_id(&Block::DISPENSER))
        .await?;
    for x in 1..=3 {
        for z in 1..=3 {
            test.set_block_default(&BlockPos::new(x, 1, z), &Block::WATER)
                .await?;
        }
    }
    test.set_container_item(&disp, 0, "minecraft:oak_boat", 1)
        .await?;
    test.set_block_default(&BlockPos::new(1, 3, 2), &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(5)
        .then_execute(move |test| async move {
            test.assert_container_empty(&disp).await?;
            test.assert_entity_present("minecraft:oak_boat").await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_dropper_drops_item_entity_into_air(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 4).await?;
    let dropper = BlockPos::new(1, 1, 1);
    test.set_block(&dropper, east_dispenser_id(&Block::DROPPER))
        .await?;
    test.set_container_item(&dropper, 0, "minecraft:gold_ingot", 1)
        .await?;
    test.set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(5)
        .then_execute(move |test| async move {
            test.assert_container_empty(&dropper).await?;
            test.assert_entity_present("minecraft:item").await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_dropper_transfers_into_hopper_and_barrel(
    test: GameTestContext,
) -> GameTestResult<()> {
    build_stone_floor(&test, 5, 5).await?;
    let d1 = BlockPos::new(1, 1, 1);
    let hopper = BlockPos::new(2, 1, 1);
    let d2 = BlockPos::new(1, 1, 3);
    let barrel = BlockPos::new(2, 1, 3);

    test.set_block(&d1, east_dispenser_id(&Block::DROPPER))
        .await?;
    let mut h_props = HopperLikeProperties::default(&Block::HOPPER);
    h_props.enabled = false;
    test.set_block(&hopper, h_props.to_state_id(&Block::HOPPER))
        .await?;
    test.set_container_item(&d1, 0, "minecraft:iron_ingot", 2)
        .await?;

    test.set_block(&d2, east_dispenser_id(&Block::DROPPER))
        .await?;
    test.set_block_default(&barrel, &Block::BARREL).await?;
    test.set_container_item(&d2, 0, "minecraft:copper_ingot", 2)
        .await?;

    test.set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;
    test.set_block_default(&BlockPos::new(1, 2, 3), &Block::REDSTONE_BLOCK)
        .await?;

    test.start_sequence()
        .then_idle(5)
        .then_execute(move |test| async move {
            test.assert_container_contains_count(&hopper, "minecraft:iron_ingot", 1)
                .await?;
            test.assert_container_contains_count(&barrel, "minecraft:copper_ingot", 1)
                .await
        })
        .then_succeed();
    Ok(())
}
