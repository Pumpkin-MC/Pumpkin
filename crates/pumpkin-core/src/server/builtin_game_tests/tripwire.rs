use pumpkin_data::{
    Block,
    block_properties::{HorizontalFacing, TripwireHookLikeProperties, TripwireLikeProperties},
};
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use super::build_stone_floor;

pub(super) async fn test_tripwire_entity_collision(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    let west_wall = BlockPos::new(0, 1, 2);
    let west_hook = BlockPos::new(1, 1, 2);
    let wire_pos = BlockPos::new(2, 1, 2);
    let east_hook = BlockPos::new(3, 1, 2);
    let east_wall = BlockPos::new(4, 1, 2);

    helper.set_block_default(&west_wall, &Block::STONE).await?;
    helper.set_block_default(&east_wall, &Block::STONE).await?;

    let mut west_hook_props = TripwireHookLikeProperties::default(&Block::TRIPWIRE_HOOK);
    west_hook_props.facing = HorizontalFacing::East;
    helper
        .set_block(
            &west_hook,
            west_hook_props.to_state_id(&Block::TRIPWIRE_HOOK),
        )
        .await?;

    let mut east_hook_props = TripwireHookLikeProperties::default(&Block::TRIPWIRE_HOOK);
    east_hook_props.facing = HorizontalFacing::West;
    helper
        .set_block(
            &east_hook,
            east_hook_props.to_state_id(&Block::TRIPWIRE_HOOK),
        )
        .await?;

    let mut wire_props = TripwireLikeProperties::default(&Block::TRIPWIRE);
    wire_props.east = true;
    wire_props.west = true;
    helper
        .set_block(&wire_pos, wire_props.to_state_id(&Block::TRIPWIRE))
        .await?;

    helper
        .spawn_entity_at("minecraft:armor_stand", Vector3::new(2.5, 1.0, 2.5))
        .await?;

    helper.succeed_when(move |helper| async move {
        let wire_state =
            TripwireLikeProperties::from_state_id(helper.block_state_id(&wire_pos).await);
        let west_state =
            TripwireHookLikeProperties::from_state_id(helper.block_state_id(&west_hook).await);
        let east_state =
            TripwireHookLikeProperties::from_state_id(helper.block_state_id(&east_hook).await);
        helper.assert_true(
            wire_state.powered && west_state.powered && east_state.powered,
            "Expected tripwire and both tripwire hooks to become powered on entity collision",
        )
    })?;
    Ok(())
}
