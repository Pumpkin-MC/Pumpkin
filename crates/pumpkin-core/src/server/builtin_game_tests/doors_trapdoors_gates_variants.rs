use pumpkin_data::{
    Block,
    block_properties::{
        DoubleBlockHalf, OakDoorLikeProperties, OakFenceGateLikeProperties, OakFenceLikeProperties,
        OakTrapdoorLikeProperties,
    },
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

async fn place_door(
    helper: &GameTestHelper,
    lower_pos: &BlockPos,
    block: &Block,
) -> GameTestResult<()> {
    let mut lower = OakDoorLikeProperties::default(block);
    lower.half = DoubleBlockHalf::Lower;
    lower.open = false;
    lower.powered = false;
    let mut upper = lower;
    upper.half = DoubleBlockHalf::Upper;
    helper
        .set_block(lower_pos, lower.to_state_id(block))
        .await?;
    helper
        .set_block(&lower_pos.up(), upper.to_state_id(block))
        .await
}

async fn assert_door_open(
    helper: &GameTestHelper,
    lower_pos: &BlockPos,
    expected_open: bool,
) -> GameTestResult<()> {
    let l = OakDoorLikeProperties::from_state_id(helper.block_state_id(lower_pos).await);
    let u = OakDoorLikeProperties::from_state_id(helper.block_state_id(&lower_pos.up()).await);
    helper.assert_true(
        l.open == expected_open && u.open == expected_open,
        format!(
            "Expected door at {lower_pos:?} open={expected_open}, got lower={} upper={}",
            l.open, u.open
        ),
    )
}

pub(super) async fn test_oak_and_spruce_doors_redstone(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 4).await?;
    let d1 = BlockPos::new(1, 1, 2);
    let d2 = BlockPos::new(3, 1, 2);
    place_door(&helper, &d1, &Block::OAK_DOOR).await?;
    place_door(&helper, &d2, &Block::SPRUCE_DOOR).await?;

    helper
        .set_block_default(&BlockPos::new(1, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;
    helper
        .set_block_default(&BlockPos::new(3, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;
    assert_door_open(&helper, &d1, true).await?;
    assert_door_open(&helper, &d2, true).await?;

    helper
        .set_block_default(&BlockPos::new(1, 1, 1), &Block::AIR)
        .await?;
    helper
        .set_block_default(&BlockPos::new(3, 1, 1), &Block::AIR)
        .await?;
    assert_door_open(&helper, &d1, false).await?;
    assert_door_open(&helper, &d2, false).await?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_birch_and_jungle_doors_redstone(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 4).await?;
    let d1 = BlockPos::new(1, 1, 2);
    let d2 = BlockPos::new(3, 1, 2);
    place_door(&helper, &d1, &Block::BIRCH_DOOR).await?;
    place_door(&helper, &d2, &Block::JUNGLE_DOOR).await?;

    helper
        .set_block_default(&BlockPos::new(1, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;
    helper
        .set_block_default(&BlockPos::new(3, 1, 1), &Block::REDSTONE_BLOCK)
        .await?;
    assert_door_open(&helper, &d1, true).await?;
    assert_door_open(&helper, &d2, true).await?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_acacia_dark_oak_mangrove_cherry_doors(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 8, 4).await?;
    let doors = [
        (1, &Block::ACACIA_DOOR),
        (3, &Block::DARK_OAK_DOOR),
        (5, &Block::MANGROVE_DOOR),
        (7, &Block::CHERRY_DOOR),
    ];
    for (x, block) in doors {
        let pos = BlockPos::new(x, 1, 2);
        place_door(&helper, &pos, block).await?;
        helper
            .set_block_default(&BlockPos::new(x, 1, 1), &Block::REDSTONE_BLOCK)
            .await?;
        assert_door_open(&helper, &pos, true).await?;
    }
    helper.succeed();
    Ok(())
}

pub(super) async fn test_bamboo_crimson_warped_doors(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 7, 4).await?;
    let doors = [
        (1, &Block::BAMBOO_DOOR),
        (3, &Block::CRIMSON_DOOR),
        (5, &Block::WARPED_DOOR),
    ];
    for (x, block) in doors {
        let pos = BlockPos::new(x, 1, 2);
        place_door(&helper, &pos, block).await?;
        helper
            .set_block_default(&BlockPos::new(x, 1, 1), &Block::REDSTONE_BLOCK)
            .await?;
        assert_door_open(&helper, &pos, true).await?;
    }
    helper.succeed();
    Ok(())
}

pub(super) async fn test_copper_doors_oxidation_variants(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 8, 4).await?;
    let doors = [
        (1, &Block::COPPER_DOOR),
        (3, &Block::EXPOSED_COPPER_DOOR),
        (5, &Block::WEATHERED_COPPER_DOOR),
        (7, &Block::OXIDIZED_COPPER_DOOR),
    ];
    for (x, block) in doors {
        let pos = BlockPos::new(x, 1, 2);
        place_door(&helper, &pos, block).await?;
        helper
            .set_block_default(&BlockPos::new(x, 1, 1), &Block::REDSTONE_BLOCK)
            .await?;
        assert_door_open(&helper, &pos, true).await?;
    }
    helper.succeed();
    Ok(())
}

pub(super) async fn test_door_breaks_when_lower_half_removed(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 4).await?;
    let lower = BlockPos::new(1, 1, 2);
    let upper = BlockPos::new(1, 2, 2);
    place_door(&helper, &lower, &Block::OAK_DOOR).await?;

    helper.set_block_default(&lower, &Block::AIR).await?;
    helper.assert_block_present(&upper, &Block::AIR).await?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_wooden_trapdoors_variants_redstone(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 8, 4).await?;
    let trapdoors = [
        (1, &Block::SPRUCE_TRAPDOOR),
        (3, &Block::BIRCH_TRAPDOOR),
        (5, &Block::BAMBOO_TRAPDOOR),
        (7, &Block::CRIMSON_TRAPDOOR),
    ];
    for (x, block) in trapdoors {
        let pos = BlockPos::new(x, 1, 2);
        helper.set_block_default(&pos, block).await?;
        helper
            .set_block_default(&BlockPos::new(x, 1, 1), &Block::REDSTONE_BLOCK)
            .await?;
        let open = OakTrapdoorLikeProperties::from_state_id(helper.block_state_id(&pos).await).open;
        helper.assert_true(open, "Expected wooden trapdoor variant to open on redstone")?;
    }
    helper.succeed();
    Ok(())
}

pub(super) async fn test_copper_trapdoors_oxidation_variants(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 8, 4).await?;
    let trapdoors = [
        (1, &Block::COPPER_TRAPDOOR),
        (3, &Block::EXPOSED_COPPER_TRAPDOOR),
        (5, &Block::WEATHERED_COPPER_TRAPDOOR),
        (7, &Block::OXIDIZED_COPPER_TRAPDOOR),
    ];
    for (x, block) in trapdoors {
        let pos = BlockPos::new(x, 1, 2);
        helper.set_block_default(&pos, block).await?;
        helper
            .set_block_default(&BlockPos::new(x, 1, 1), &Block::REDSTONE_BLOCK)
            .await?;
        let open = OakTrapdoorLikeProperties::from_state_id(helper.block_state_id(&pos).await).open;
        helper.assert_true(open, "Expected copper trapdoor variant to open on redstone")?;
    }
    helper.succeed();
    Ok(())
}

pub(super) async fn test_fence_gates_wood_variants_redstone(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 8, 4).await?;
    let gates = [
        (1, &Block::SPRUCE_FENCE_GATE),
        (3, &Block::BIRCH_FENCE_GATE),
        (5, &Block::BAMBOO_FENCE_GATE),
        (7, &Block::CRIMSON_FENCE_GATE),
    ];
    for (x, block) in gates {
        let pos = BlockPos::new(x, 1, 2);
        helper.set_block_default(&pos, block).await?;
        helper
            .set_block_default(&BlockPos::new(x, 1, 1), &Block::REDSTONE_BLOCK)
            .await?;
        let open =
            OakFenceGateLikeProperties::from_state_id(helper.block_state_id(&pos).await).open;
        helper.assert_true(open, "Expected fence gate variant to open on redstone")?;
    }
    helper.succeed();
    Ok(())
}

pub(super) async fn test_fence_gate_in_wall_neighbor_update(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 4).await?;
    let gate_pos = BlockPos::new(2, 1, 2);
    helper
        .set_block_default(&gate_pos, &Block::OAK_FENCE_GATE)
        .await?;
    // Default facing is North, so perpendicular walls are at West (1,1,2) and East (3,1,2)
    helper
        .set_block_default(&BlockPos::new(1, 1, 2), &Block::COBBLESTONE_WALL)
        .await?;

    let props = OakFenceGateLikeProperties::from_state_id(helper.block_state_id(&gate_pos).await);
    helper.assert_true(
        props.in_wall,
        "Expected fence gate adjacent to cobblestone_wall to set in_wall=true",
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_fence_horizontal_connections_update(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    let center = BlockPos::new(2, 1, 2);
    helper.set_block_default(&center, &Block::OAK_FENCE).await?;
    helper
        .set_block_default(&BlockPos::new(2, 1, 1), &Block::OAK_FENCE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(3, 1, 2), &Block::STONE)
        .await?;

    let props = OakFenceLikeProperties::from_state_id(helper.block_state_id(&center).await);
    helper.assert_true(
        props.north && props.east && !props.south && !props.west,
        "Expected oak_fence north & east connected",
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_iron_bars_and_glass_pane_connections(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    let bars = BlockPos::new(2, 1, 2);
    let pane = BlockPos::new(2, 1, 3);
    helper.set_block_default(&bars, &Block::IRON_BARS).await?;
    helper.set_block_default(&pane, &Block::GLASS_PANE).await?;
    helper
        .set_block_default(&BlockPos::new(1, 1, 2), &Block::STONE)
        .await?;

    let bars_props = OakFenceLikeProperties::from_state_id(helper.block_state_id(&bars).await);
    let pane_props = OakFenceLikeProperties::from_state_id(helper.block_state_id(&pane).await);
    helper.assert_true(
        bars_props.south && bars_props.west && pane_props.north,
        "Expected iron_bars and glass_pane to connect",
    )?;
    helper.succeed();
    Ok(())
}
