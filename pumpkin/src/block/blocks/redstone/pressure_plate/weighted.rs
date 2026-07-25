use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockState, BlockStateId, block_properties::BlockProperties,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, EmitsRedstonePowerArgs,
        GetRedstonePowerArgs, OnEntityCollisionArgs, OnNeighborUpdateArgs, OnScheduledTickArgs,
        OnStateReplacedArgs,
    },
    entity::EntityBase,
    world::World,
};

use super::{PressurePlate, detection_box_at};

/// This is for Gold and Iron Pressure Plate
pub struct WeightedPressurePlateBlock;

type PressurePlateProps = pumpkin_data::block_properties::LightWeightedPressurePlateLikeProperties;

/// Vanilla `WeightedPressurePlateBlock.getSignalStrength`.
#[must_use]
fn weighted_signal_strength(entity_count: usize, max_weight: usize) -> u8 {
    debug_assert!(max_weight > 0);
    let fraction = entity_count.min(max_weight) as f32 / max_weight as f32;
    (fraction * 15.0).ceil() as u8
}

impl BlockMetadata for WeightedPressurePlateBlock {
    fn ids() -> Box<[BlockId]> {
        // light = Gold
        // heavy = Iron
        [
            BlockId::LIGHT_WEIGHTED_PRESSURE_PLATE,
            BlockId::HEAVY_WEIGHTED_PRESSURE_PLATE,
        ]
        .into()
    }
}

impl BlockBehaviour for WeightedPressurePlateBlock {
    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            self.on_entity_collision_pp(args).await;
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            self.on_scheduled_tick_pp(args).await;
        })
    }

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            self.on_state_replaced_pp(args).await;
        })
    }

    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move { self.get_redstone_output(args.block, args.state.id) })
    }

    fn get_strong_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            if args.direction == BlockDirection::Up {
                return self.get_redstone_output(args.block, args.state.id);
            }
            0
        })
    }

    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !Self::can_pressure_plate_place_at(args.world, args.position) {
                args.world
                    .break_block(args.position, None, BlockFlags::NOTIFY_ALL)
                    .await;
            }
        })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        args.world
            .is_some_and(|world| Self::can_pressure_plate_place_at(world, args.position))
    }
}

impl PressurePlate for WeightedPressurePlateBlock {
    fn get_redstone_output(&self, block: &Block, state: BlockStateId) -> u8 {
        let props = PressurePlateProps::from_state_id(state, block);
        props.power
    }

    async fn calculate_redstone_output(&self, world: &World, block: &Block, pos: &BlockPos) -> u8 {
        // light = Gold
        // heavy = Iron
        let weight = if block == &Block::LIGHT_WEIGHTED_PRESSURE_PLATE {
            // Gold
            15
        } else {
            // Iron
            150
        };
        let aabb = detection_box_at(pos);
        let entity_count = world
            .get_all_at_box(&aabb)
            .into_iter()
            .filter(|entity| !entity.is_spectator())
            .count();
        weighted_signal_strength(entity_count, weight)
    }

    fn set_redstone_output(&self, block: &Block, state: &BlockState, output: u8) -> BlockStateId {
        let mut props = PressurePlateProps::from_state_id(state.id, block);
        props.power = output;
        props.to_state_id(block)
    }

    fn tick_rate(&self) -> u8 {
        10
    }
}

#[cfg(test)]
mod tests {
    use super::weighted_signal_strength;

    #[test]
    fn produces_vanilla_weighted_plate_levels() {
        assert_eq!(weighted_signal_strength(0, 15), 0);
        assert_eq!(weighted_signal_strength(1, 15), 1);
        assert_eq!(weighted_signal_strength(14, 15), 14);
        assert_eq!(weighted_signal_strength(15, 15), 15);

        assert_eq!(weighted_signal_strength(1, 150), 1);
        assert_eq!(weighted_signal_strength(10, 150), 1);
        assert_eq!(weighted_signal_strength(11, 150), 2);
        assert_eq!(weighted_signal_strength(150, 150), 15);
        assert_eq!(weighted_signal_strength(151, 150), 15);
    }
}
