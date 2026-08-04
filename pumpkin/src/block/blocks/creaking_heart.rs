use pumpkin_data::block_properties::{
    Axis, BlockProperties, CreakingHeartLikeProperties, CreakingHeartState,
    PaleOakWoodLikeProperties,
};
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{BlockDirection, BlockId, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

use crate::block::entities::creaking_heart::CreakingHeartBlockEntity;
use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, GetComparatorOutputArgs,
    GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs,
};
use crate::world::World;

/// `net.minecraft.world.level.block.CreakingHeartBlock`.
pub struct CreakingHeartBlock;

impl BlockMetadata for CreakingHeartBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::CREAKING_HEART].into()
    }
}

/// `EnvironmentAttributes.CREAKING_ACTIVE`.
///
/// In 26.2 this is a data-driven environment attribute rather than a hardcoded night
/// check: `data/minecraft/timeline/day.json` drives the
/// `minecraft:gameplay/creaking_active` track with keyframes true at tick 12600 and
/// false at tick 23401 (the same track that gates `eyeblossom_open`).
pub async fn creaking_active(world: &World) -> bool {
    let time_of_day = world.level_time.lock().await.time_of_day.rem_euclid(24000);
    (12600..23401).contains(&time_of_day)
}

/// `CreakingHeartBlock.hasRequiredLogs`: both neighbours along the heart's axis must be
/// pale oak logs sharing that same axis.
pub fn has_required_logs(world: &World, pos: &BlockPos, axis: Axis) -> bool {
    for direction in axis_directions(axis) {
        let neighbour_pos = pos.offset(direction.to_offset());
        let (block, state) = world.get_block_and_state(&neighbour_pos);
        if !block.has_tag(&tag::Block::MINECRAFT_PALE_OAK_LOGS) {
            return false;
        }
        // Read the axis through the log's own property type, not the heart's.
        let props = PaleOakWoodLikeProperties::from_state_id(state.id, block);
        if props.axis != axis {
            return false;
        }
    }
    true
}

const fn axis_directions(axis: Axis) -> [BlockDirection; 2] {
    match axis {
        Axis::X => [BlockDirection::West, BlockDirection::East],
        Axis::Y => [BlockDirection::Down, BlockDirection::Up],
        Axis::Z => [BlockDirection::North, BlockDirection::South],
    }
}

/// `CreakingHeartBlock.updateState`: an UPROOTED heart that has gained its logs wakes to
/// AWAKE or DORMANT depending on `creaking_active`. Any other state is left alone -- only
/// the block entity's ticker moves a live heart between AWAKE and DORMANT.
async fn update_state(
    world: &World,
    pos: &BlockPos,
    props: &mut CreakingHeartLikeProperties,
) -> bool {
    if props.creaking_heart_state != CreakingHeartState::Uprooted
        || !has_required_logs(world, pos, props.axis)
    {
        return false;
    }
    props.creaking_heart_state = if creaking_active(world).await {
        CreakingHeartState::Awake
    } else {
        CreakingHeartState::Dormant
    };
    true
}

impl BlockBehaviour for CreakingHeartBlock {
    /// `getStateForPlacement`: AXIS from the clicked face, then `updateState`.
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = CreakingHeartLikeProperties::default(args.block);
            props.axis = match args.direction {
                BlockDirection::East | BlockDirection::West => Axis::X,
                BlockDirection::Up | BlockDirection::Down => Axis::Y,
                BlockDirection::North | BlockDirection::South => Axis::Z,
            };
            // NATURAL stays false: vanilla only sets it from the pale oak tree decorator.
            update_state(args.world, args.position, &mut props).await;
            props.to_state_id(args.block)
        })
    }

    /// `updateShape`: any neighbour change schedules a tick one tick out.
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
            args.state_id
        })
    }

    /// `CreakingHeartBlock.tick`: re-runs `updateState` and writes it back if it changed.
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            let mut props = CreakingHeartLikeProperties::from_state_id(state.id, args.block);
            if update_state(args.world, args.position, &mut props).await {
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        })
    }

    /// `getAnalogOutputSignal`: 0 while UPROOTED, else the block entity's cached signal.
    fn get_comparator_output<'a>(
        &'a self,
        args: GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        Box::pin(async move {
            let props = CreakingHeartLikeProperties::from_state_id(args.state.id, args.block);
            if props.creaking_heart_state == CreakingHeartState::Uprooted {
                return Some(0);
            }
            Some(
                args.world
                    .get_block_entity(args.position)
                    .and_then(|entity| {
                        entity
                            .as_any()
                            .downcast_ref::<CreakingHeartBlockEntity>()
                            .map(CreakingHeartBlockEntity::get_analog_output_signal)
                    })
                    .unwrap_or(0),
            )
        })
    }
}
