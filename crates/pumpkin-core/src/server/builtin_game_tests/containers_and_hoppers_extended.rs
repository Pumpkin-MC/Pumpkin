use pumpkin_data::{
    Block, BlockStateId,
    block_properties::{
        CrafterLikeProperties, FacingHopper, FurnaceLikeProperties, HopperLikeProperties,
        Orientation,
    },
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use super::build_stone_floor;

fn hopper_state(facing: FacingHopper, enabled: bool) -> BlockStateId {
    let mut props = HopperLikeProperties::default(&Block::HOPPER);
    props.facing = facing;
    props.enabled = enabled;
    props.to_state_id(&Block::HOPPER)
}

pub(super) async fn test_hopper_pulls_item_entity_from_above(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 4).await?;
    let hopper = BlockPos::new(1, 1, 1);
    helper
        .set_block(&hopper, hopper_state(FacingHopper::Down, true))
        .await?;
    helper
        .spawn_item_at("minecraft:emerald", 1, Vector3::new(1.5, 2.2, 1.5))
        .await?;

    helper
        .start_sequence()
        .then_idle(6)
        .then_execute(move |helper| async move {
            helper
                .assert_container_contains_count(&hopper, "minecraft:emerald", 1)
                .await?;
            helper.assert_entity_not_present("minecraft:item").await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_hopper_pulls_from_chest_above(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 4).await?;
    let hopper = BlockPos::new(1, 1, 1);
    let top_chest = BlockPos::new(1, 2, 1);
    helper
        .set_block(&hopper, hopper_state(FacingHopper::North, true))
        .await?;
    helper.set_block_default(&top_chest, &Block::CHEST).await?;
    helper
        .set_container_item(&top_chest, 0, "minecraft:gold_ingot", 2)
        .await?;

    helper
        .start_sequence()
        .then_idle(12)
        .then_execute(move |helper| async move {
            helper.assert_container_empty(&top_chest).await?;
            helper
                .assert_container_contains_count(&hopper, "minecraft:gold_ingot", 2)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_hopper_pushes_sideways_into_barrel(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 4).await?;
    let hopper = BlockPos::new(1, 1, 1);
    let barrel = BlockPos::new(2, 1, 1);
    helper.set_block_default(&barrel, &Block::BARREL).await?;
    helper
        .set_block(&hopper, hopper_state(FacingHopper::East, true))
        .await?;
    helper
        .set_container_item(&hopper, 0, "minecraft:lapis_lazuli", 2)
        .await?;

    helper
        .start_sequence()
        .then_idle(12)
        .then_execute(move |helper| async move {
            helper.assert_container_empty(&hopper).await?;
            helper
                .assert_container_contains_count(&barrel, "minecraft:lapis_lazuli", 2)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_hopper_chain_three_hoppers_to_chest(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 6, 4).await?;
    let h1 = BlockPos::new(1, 1, 1);
    let h2 = BlockPos::new(2, 1, 1);
    let h3 = BlockPos::new(3, 1, 1);
    let chest = BlockPos::new(4, 1, 1);

    helper.set_block_default(&chest, &Block::CHEST).await?;
    helper
        .set_block(&h3, hopper_state(FacingHopper::East, true))
        .await?;
    helper
        .set_block(&h2, hopper_state(FacingHopper::East, true))
        .await?;
    helper
        .set_block(&h1, hopper_state(FacingHopper::East, true))
        .await?;
    helper
        .set_container_item(&h1, 0, "minecraft:quartz", 1)
        .await?;

    helper
        .start_sequence()
        .then_idle(20)
        .then_execute(move |helper| async move {
            helper
                .assert_container_contains_count(&chest, "minecraft:quartz", 1)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_hopper_locked_by_redstone_stops_transfer(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 4).await?;
    let hopper = BlockPos::new(1, 1, 1);
    let chest = BlockPos::new(2, 1, 1);
    helper.set_block_default(&chest, &Block::CHEST).await?;
    helper
        .set_block(&hopper, hopper_state(FacingHopper::East, true))
        .await?;
    // Power hopper from above before inserting item
    helper
        .set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;
    helper
        .set_container_item(&hopper, 0, "minecraft:amethyst_shard", 2)
        .await?;

    helper
        .start_sequence()
        .then_idle(10)
        .then_execute(move |helper| async move {
            helper.assert_container_empty(&chest).await?;
            helper
                .assert_container_contains_count(&hopper, "minecraft:amethyst_shard", 2)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_hopper_unlocked_resumes_transfer(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 4).await?;
    let hopper = BlockPos::new(1, 1, 1);
    let chest = BlockPos::new(2, 1, 1);
    let power = BlockPos::new(1, 2, 1);
    helper.set_block_default(&chest, &Block::CHEST).await?;
    helper
        .set_block(&hopper, hopper_state(FacingHopper::East, true))
        .await?;
    helper
        .set_block_default(&power, &Block::REDSTONE_BLOCK)
        .await?;
    helper
        .set_container_item(&hopper, 0, "minecraft:redstone", 1)
        .await?;

    helper
        .start_sequence()
        .then_idle(5)
        .then_execute(move |helper| async move {
            helper.assert_container_empty(&chest).await?;
            helper.set_block_default(&power, &Block::AIR).await
        })
        .then_idle(10)
        .then_execute(move |helper| async move {
            helper
                .assert_container_contains_count(&chest, "minecraft:redstone", 1)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_hopper_feeds_furnace_top_and_side(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 4).await?;
    let furnace = BlockPos::new(2, 1, 1);
    let top_hopper = BlockPos::new(2, 2, 1);
    let side_hopper = BlockPos::new(1, 1, 1);

    helper.set_block_default(&furnace, &Block::FURNACE).await?;
    helper
        .set_block(&top_hopper, hopper_state(FacingHopper::Down, true))
        .await?;
    helper
        .set_block(&side_hopper, hopper_state(FacingHopper::East, true))
        .await?;
    helper
        .set_container_item(&top_hopper, 0, "minecraft:raw_iron", 1)
        .await?;

    helper
        .start_sequence()
        .then_idle(4)
        .then_execute(move |helper| async move {
            helper
                .set_container_item(&side_hopper, 0, "minecraft:coal", 1)
                .await
        })
        .then_idle(6)
        .then_execute(move |helper| async move {
            let lit =
                FurnaceLikeProperties::from_state_id(helper.block_state_id(&furnace).await).lit;
            helper.assert_true(
                lit,
                "Expected furnace fed by top (input) and side (fuel) hoppers to ignite",
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_blast_furnace_smelts_raw_iron(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 3, 3).await?;
    let bf = BlockPos::new(1, 1, 1);
    helper.set_block_default(&bf, &Block::BLAST_FURNACE).await?;
    helper
        .set_container_item(&bf, 0, "minecraft:raw_iron", 1)
        .await?;
    helper
        .set_container_item(&bf, 1, "minecraft:coal", 1)
        .await?;

    helper
        .start_sequence()
        .then_idle(2)
        .then_execute(move |helper| async move {
            let lit = FurnaceLikeProperties::from_state_id(helper.block_state_id(&bf).await).lit;
            helper.assert_true(
                lit,
                "Expected blast_furnace to ignite with raw_iron and coal",
            )
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_smoker_cooks_raw_beef(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 3, 3).await?;
    let smoker = BlockPos::new(1, 1, 1);
    helper.set_block_default(&smoker, &Block::SMOKER).await?;
    helper
        .set_container_item(&smoker, 0, "minecraft:beef", 1)
        .await?;
    helper
        .set_container_item(&smoker, 1, "minecraft:coal", 1)
        .await?;

    helper
        .start_sequence()
        .then_idle(2)
        .then_execute(move |helper| async move {
            let lit =
                FurnaceLikeProperties::from_state_id(helper.block_state_id(&smoker).await).lit;
            helper.assert_true(lit, "Expected smoker to ignite with raw beef and coal")
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_crafter_crafts_iron_ingot_from_nuggets(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 4).await?;
    let crafter = BlockPos::new(1, 1, 1);
    let chest = BlockPos::new(2, 1, 1);
    let mut props = CrafterLikeProperties::default(&Block::CRAFTER);
    props.orientation = Orientation::EastUp;
    helper
        .set_block(&crafter, props.to_state_id(&Block::CRAFTER))
        .await?;
    helper.set_block_default(&chest, &Block::CHEST).await?;

    for slot in 0..9 {
        helper
            .set_container_item(&crafter, slot, "minecraft:iron_nugget", 1)
            .await?;
    }
    helper
        .set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(6)
        .then_execute(move |helper| async move {
            helper
                .assert_container_contains_count(&chest, "minecraft:iron_ingot", 1)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_crafter_disabled_slots_skip_items(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 4).await?;
    let crafter = BlockPos::new(1, 1, 1);
    let chest = BlockPos::new(2, 1, 1);
    let mut props = CrafterLikeProperties::default(&Block::CRAFTER);
    props.orientation = Orientation::EastUp;
    helper
        .set_block(&crafter, props.to_state_id(&Block::CRAFTER))
        .await?;
    helper.set_block_default(&chest, &Block::CHEST).await?;

    // Two planks vertically craft 4 sticks
    helper
        .set_container_item(&crafter, 0, "minecraft:oak_planks", 1)
        .await?;
    helper
        .set_container_item(&crafter, 3, "minecraft:oak_planks", 1)
        .await?;
    helper
        .set_block_default(&BlockPos::new(1, 2, 1), &Block::REDSTONE_BLOCK)
        .await?;

    helper
        .start_sequence()
        .then_idle(6)
        .then_execute(move |helper| async move {
            helper
                .assert_container_contains_count(&chest, "minecraft:stick", 4)
                .await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_barrel_and_trapped_chest_inventories(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 4).await?;
    let barrel = BlockPos::new(1, 1, 1);
    let trapped = BlockPos::new(3, 1, 1);
    helper.set_block_default(&barrel, &Block::BARREL).await?;
    helper
        .set_block_default(&trapped, &Block::TRAPPED_CHEST)
        .await?;

    helper
        .set_container_item(&barrel, 0, "minecraft:diamond", 4)
        .await?;
    helper
        .set_container_item(&trapped, 0, "minecraft:emerald", 6)
        .await?;

    helper
        .assert_container_contains_count(&barrel, "minecraft:diamond", 4)
        .await?;
    helper
        .assert_container_contains_count(&trapped, "minecraft:emerald", 6)
        .await?;
    helper.succeed();
    Ok(())
}
