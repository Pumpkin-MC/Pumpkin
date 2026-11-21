use pumpkin_data::{
    Block,
    block_properties::{
        CampfireLikeProperties, EastWall, Half, HorizontalFacing, NorthWall,
        ResinBrickWallLikeProperties, SouthWall, StairsShape, WestWall,
        WhiteWoolStairsLikeProperties,
    },
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_cobblestone_wall_connects_north_south_straight(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    let n = BlockPos::new(2, 1, 1);
    let c = BlockPos::new(2, 1, 2);
    let s = BlockPos::new(2, 1, 3);
    helper
        .set_block_default(&c, &Block::COBBLESTONE_WALL)
        .await?;
    helper
        .set_block_default(&n, &Block::COBBLESTONE_WALL)
        .await?;
    helper
        .set_block_default(&s, &Block::COBBLESTONE_WALL)
        .await?;

    let props = ResinBrickWallLikeProperties::from_state_id(helper.block_state_id(&c).await);
    helper.assert_true(
        props.north == NorthWall::Low
            && props.south == SouthWall::Low
            && props.east == EastWall::None
            && props.west == WestWall::None
            && !props.up,
        "Expected straight N-S wall with up=false",
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_cobblestone_wall_corner_raises_center_post(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    let c = BlockPos::new(2, 1, 2);
    helper
        .set_block_default(&c, &Block::COBBLESTONE_WALL)
        .await?;
    helper
        .set_block_default(&BlockPos::new(2, 1, 1), &Block::COBBLESTONE_WALL)
        .await?;
    helper
        .set_block_default(&BlockPos::new(3, 1, 2), &Block::COBBLESTONE_WALL)
        .await?;

    let props = ResinBrickWallLikeProperties::from_state_id(helper.block_state_id(&c).await);
    helper.assert_true(
        props.north == NorthWall::Low && props.east == EastWall::Low && props.up,
        "Expected L-corner wall to raise center post (up=true)",
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_cobblestone_wall_tall_side_under_full_block(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    let c = BlockPos::new(2, 1, 2);
    helper
        .set_block_default(&c, &Block::COBBLESTONE_WALL)
        .await?;
    helper
        .set_block_default(&BlockPos::new(2, 2, 2), &Block::STONE)
        .await?;
    helper
        .set_block_default(&BlockPos::new(2, 1, 1), &Block::COBBLESTONE_WALL)
        .await?;

    let props = ResinBrickWallLikeProperties::from_state_id(helper.block_state_id(&c).await);
    helper.assert_true(
        props.north == NorthWall::Tall,
        "Expected wall side under full stone block to become Tall",
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_wall_connects_to_iron_bars_and_glass_pane(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    let wall = BlockPos::new(2, 1, 2);
    helper
        .set_block_default(&wall, &Block::COBBLESTONE_WALL)
        .await?;
    helper
        .set_block_default(&BlockPos::new(2, 1, 1), &Block::IRON_BARS)
        .await?;
    helper
        .set_block_default(&BlockPos::new(3, 1, 2), &Block::GLASS_PANE)
        .await?;

    let props = ResinBrickWallLikeProperties::from_state_id(helper.block_state_id(&wall).await);
    helper.assert_true(
        props.north == NorthWall::Low && props.east == EastWall::Low,
        "Expected wall to connect to iron_bars and glass_pane",
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_wall_connects_to_fence_gate(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    let wall = BlockPos::new(2, 1, 2);
    let gate = BlockPos::new(3, 1, 2);
    helper
        .set_block_default(&wall, &Block::COBBLESTONE_WALL)
        .await?;
    // Default oak_fence_gate faces North, so its axis runs West-East and connects to wall at (2,1,2)
    helper
        .set_block_default(&gate, &Block::OAK_FENCE_GATE)
        .await?;

    let props = ResinBrickWallLikeProperties::from_state_id(helper.block_state_id(&wall).await);
    helper.assert_true(
        props.east == EastWall::Low,
        "Expected wall to connect to perpendicular fence_gate",
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_stairs_inner_left_and_right_corner_shapes(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 6, 5).await?;
    let main = BlockPos::new(2, 1, 2);
    let front = BlockPos::new(2, 1, 3);

    let mut s_north = WhiteWoolStairsLikeProperties::default(&Block::OAK_STAIRS);
    s_north.facing = HorizontalFacing::North;
    s_north.half = Half::Bottom;
    s_north.shape = StairsShape::Straight;
    helper
        .set_block(&main, s_north.to_state_id(&Block::OAK_STAIRS))
        .await?;

    let mut s_west = s_north;
    s_west.facing = HorizontalFacing::West;
    helper
        .set_block(&front, s_west.to_state_id(&Block::OAK_STAIRS))
        .await?;

    let shape_left =
        WhiteWoolStairsLikeProperties::from_state_id(helper.block_state_id(&main).await).shape;
    helper.assert_true(
        shape_left == StairsShape::InnerLeft,
        format!("Expected InnerLeft stair shape, got {shape_left:?}"),
    )?;

    let mut s_east = s_north;
    s_east.facing = HorizontalFacing::East;
    helper
        .set_block(&front, s_east.to_state_id(&Block::OAK_STAIRS))
        .await?;

    let shape_right =
        WhiteWoolStairsLikeProperties::from_state_id(helper.block_state_id(&main).await).shape;
    helper.assert_true(
        shape_right == StairsShape::InnerRight,
        format!("Expected InnerRight stair shape, got {shape_right:?}"),
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_stairs_outer_left_and_right_corner_shapes(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 6, 5).await?;
    let main = BlockPos::new(2, 1, 2);
    let back = BlockPos::new(2, 1, 1);

    let mut s_north = WhiteWoolStairsLikeProperties::default(&Block::OAK_STAIRS);
    s_north.facing = HorizontalFacing::North;
    s_north.half = Half::Bottom;
    s_north.shape = StairsShape::Straight;
    helper
        .set_block(&main, s_north.to_state_id(&Block::OAK_STAIRS))
        .await?;

    let mut s_west = s_north;
    s_west.facing = HorizontalFacing::West;
    helper
        .set_block(&back, s_west.to_state_id(&Block::OAK_STAIRS))
        .await?;

    let shape_left =
        WhiteWoolStairsLikeProperties::from_state_id(helper.block_state_id(&main).await).shape;
    helper.assert_true(
        shape_left == StairsShape::OuterLeft,
        format!("Expected OuterLeft stair shape, got {shape_left:?}"),
    )?;

    let mut s_east = s_north;
    s_east.facing = HorizontalFacing::East;
    helper
        .set_block(&back, s_east.to_state_id(&Block::OAK_STAIRS))
        .await?;

    let shape_right =
        WhiteWoolStairsLikeProperties::from_state_id(helper.block_state_id(&main).await).shape;
    helper.assert_true(
        shape_right == StairsShape::OuterRight,
        format!("Expected OuterRight stair shape, got {shape_right:?}"),
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_stairs_revert_to_straight_when_neighbor_removed(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    let main = BlockPos::new(2, 1, 2);
    let back = BlockPos::new(2, 1, 1);

    let mut s_north = WhiteWoolStairsLikeProperties::default(&Block::OAK_STAIRS);
    s_north.facing = HorizontalFacing::North;
    s_north.half = Half::Bottom;
    let mut s_west = s_north;
    s_west.facing = HorizontalFacing::West;

    helper
        .set_block(&main, s_north.to_state_id(&Block::OAK_STAIRS))
        .await?;
    helper
        .set_block(&back, s_west.to_state_id(&Block::OAK_STAIRS))
        .await?;
    helper.set_block_default(&back, &Block::AIR).await?;

    let shape =
        WhiteWoolStairsLikeProperties::from_state_id(helper.block_state_id(&main).await).shape;
    helper.assert_true(
        shape == StairsShape::Straight,
        format!("Expected stair to revert to Straight after neighbor removal, got {shape:?}"),
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_dirt_path_turns_to_dirt_under_solid_block(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 4).await?;
    let path = BlockPos::new(1, 1, 1);
    let above = BlockPos::new(1, 2, 1);
    helper.set_block_default(&path, &Block::DIRT_PATH).await?;
    helper.set_block_default(&above, &Block::STONE).await?;

    helper
        .start_sequence()
        .then_idle(2)
        .then_execute(move |helper| async move {
            helper.assert_block_present(&path, &Block::DIRT).await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_farmland_turns_to_dirt_under_solid_block(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 4).await?;
    let farm = BlockPos::new(1, 1, 1);
    let above = BlockPos::new(1, 2, 1);
    helper.set_block_default(&farm, &Block::FARMLAND).await?;
    helper.set_block_default(&above, &Block::STONE).await?;

    helper
        .start_sequence()
        .then_idle(2)
        .then_execute(move |helper| async move {
            helper.assert_block_present(&farm, &Block::DIRT).await
        })
        .then_succeed();
    Ok(())
}

pub(super) async fn test_campfire_signal_fire_over_hay_block(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 4).await?;
    let base = BlockPos::new(1, 1, 1);
    let fire = BlockPos::new(1, 2, 1);
    helper.set_block_default(&fire, &Block::CAMPFIRE).await?;
    helper.set_block_default(&base, &Block::HAY_BLOCK).await?;

    let props = CampfireLikeProperties::from_state_id(helper.block_state_id(&fire).await);
    helper.assert_true(
        props.signal_fire,
        "Expected campfire above hay_block to set signal_fire=true",
    )?;

    helper.set_block_default(&base, &Block::STONE).await?;
    let props_after = CampfireLikeProperties::from_state_id(helper.block_state_id(&fire).await);
    helper.assert_true(
        !props_after.signal_fire,
        "Expected campfire above stone to set signal_fire=false",
    )?;
    helper.succeed();
    Ok(())
}

pub(super) async fn test_campfire_waterlogged_extinguishes_lit(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 4).await?;
    let fire = BlockPos::new(1, 1, 1);
    let mut props = CampfireLikeProperties::default(&Block::CAMPFIRE);
    props.lit = true;
    props.waterlogged = true;
    helper
        .set_block(&fire, props.to_state_id(&Block::CAMPFIRE))
        .await?;
    // Trigger neighbor shape update so waterlogged campfire extinguishes
    helper
        .set_block_default(&BlockPos::new(2, 1, 1), &Block::STONE)
        .await?;

    let after = CampfireLikeProperties::from_state_id(helper.block_state_id(&fire).await);
    helper.assert_true(
        !after.lit,
        "Expected waterlogged campfire to extinguish lit=false on neighbor update",
    )?;
    helper.succeed();
    Ok(())
}
