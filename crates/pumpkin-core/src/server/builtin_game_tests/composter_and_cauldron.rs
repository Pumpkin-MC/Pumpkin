use pumpkin_data::{
    Block,
    block_properties::{
        ComparatorLikeProperties, ComposterLikeProperties, EastRedstone, HorizontalFacing,
        ModeComparator, NorthRedstone, RedstoneWireLikeProperties, SouthRedstone, WestRedstone,
    },
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

pub(super) async fn test_composter_and_cauldron_comparators(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let composter_pos = BlockPos::new(1, 1, 1);
    let comp_1 = BlockPos::new(1, 1, 2);
    let wire_1 = BlockPos::new(1, 1, 3);

    let cauldron_pos = BlockPos::new(3, 1, 1);
    let comp_2 = BlockPos::new(3, 1, 2);
    let wire_2 = BlockPos::new(3, 1, 3);

    let mut wire_props = RedstoneWireLikeProperties::default(&Block::REDSTONE_WIRE);
    wire_props.north = NorthRedstone::Side;
    wire_props.south = SouthRedstone::Side;
    wire_props.east = EastRedstone::None;
    wire_props.west = WestRedstone::None;
    let wire_state = wire_props.to_state_id(&Block::REDSTONE_WIRE);
    helper.set_block(&wire_1, wire_state).await?;
    helper.set_block(&wire_2, wire_state).await?;

    let mut comp_props = ComparatorLikeProperties::default(&Block::COMPARATOR);
    comp_props.facing = HorizontalFacing::North;
    comp_props.mode = ModeComparator::Compare;
    let comp_state = comp_props.to_state_id(&Block::COMPARATOR);
    helper.set_block(&comp_1, comp_state).await?;
    helper.set_block(&comp_2, comp_state).await?;

    let mut composter_props = ComposterLikeProperties::default(&Block::COMPOSTER);
    composter_props.level = 6;
    helper
        .set_block(
            &composter_pos,
            composter_props.to_state_id(&Block::COMPOSTER),
        )
        .await?;

    helper
        .set_block_default(&cauldron_pos, &Block::LAVA_CAULDRON)
        .await?;

    helper
        .start_sequence()
        .then_idle(3)
        .then_execute(move |helper| async move {
            let p1 =
                RedstoneWireLikeProperties::from_state_id(helper.block_state_id(&wire_1).await)
                    .power;
            let p2 =
                RedstoneWireLikeProperties::from_state_id(helper.block_state_id(&wire_2).await)
                    .power;
            helper.assert_true(
                p1 == 6 && p2 == 3,
                format!("Expected composter comparator power 6 and lava cauldron power 3, got {p1} and {p2}"),
            )
        })
        .then_succeed();
    Ok(())
}
