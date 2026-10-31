use crate::block::blocks::plant::PlantBlockBase;
use crate::block::{
    BlockBehaviour, BlockMetadata, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnPlaceArgs, OnScheduledTickArgs, RandomTickArgs,
};
use crate::world::World;
use pumpkin_data::block_properties::TwistingVinesProperties;
use pumpkin_data::{Block, BlockDirection, BlockId, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;

pub struct TwistingVinesBlock;

impl BlockMetadata for TwistingVinesBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::TWISTING_VINES, BlockId::TWISTING_VINES_PLANT].into()
    }
}

impl TwistingVinesBlock {
    fn get_head_pos(world: &World, pos: &BlockPos, block: &Block) -> Option<BlockPos> {
        if block == &Block::TWISTING_VINES {
            return Some(*pos);
        }
        let mut cursor = pos.up();
        while world.is_in_build_limit(cursor) {
            let current = world.get_block(&cursor);
            if current == &Block::TWISTING_VINES_PLANT {
                cursor = cursor.up();
            } else if current == &Block::TWISTING_VINES {
                return Some(cursor);
            } else {
                return None;
            }
        }
        None
    }

    fn get_blocks_to_grow_when_bonemealed() -> usize {
        let mut grow_probability = 1.0;
        let mut count = 0;
        while rand::rng().random::<f64>() < grow_probability {
            count += 1;
            grow_probability *= 0.826;
        }
        count
    }
}

impl BlockBehaviour for TwistingVinesBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if args.block == &Block::TWISTING_VINES {
            let above_block = args.world.get_block(&args.position.up());
            if above_block == &Block::TWISTING_VINES || above_block == &Block::TWISTING_VINES_PLANT
            {
                return Block::TWISTING_VINES_PLANT.default_state.id;
            }
            let mut props = TwistingVinesProperties::default(&Block::TWISTING_VINES);
            props.age = rand::rng().random_range(0..25);
            props.to_state_id(&Block::TWISTING_VINES)
        } else {
            Block::TWISTING_VINES_PLANT.default_state.id
        }
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        if args.block == &Block::TWISTING_VINES {
            let state_id = args.world.get_block_state_id(args.position);
            let mut props = TwistingVinesProperties::from_state_id(state_id);
            if props.age < 25 && rand::rng().random::<f64>() < 0.1 {
                let growth_pos = args.position.up();
                if args.world.get_block_state(&growth_pos).is_air() {
                    props.age += 1;
                    args.world.set_block_state(
                        &growth_pos,
                        props.to_state_id(&Block::TWISTING_VINES),
                        BlockFlags::NOTIFY_ALL,
                    );
                }
            }
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !<Self as PlantBlockBase>::can_place_at(self, args.world.as_ref(), args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.block == &Block::TWISTING_VINES {
            if args.direction == BlockDirection::Down {
                if <Self as PlantBlockBase>::can_place_at(self, args.world, args.position) {
                    let above_block = args.world.get_block(&args.position.up());
                    if above_block == &Block::TWISTING_VINES
                        || above_block == &Block::TWISTING_VINES_PLANT
                    {
                        return Block::TWISTING_VINES_PLANT.default_state.id;
                    }
                } else {
                    args.world.schedule_block_tick(
                        args.block,
                        *args.position,
                        1,
                        TickPriority::Normal,
                    );
                }
            }
            if args.direction == BlockDirection::Up {
                let neighbor_block = args.world.get_block(args.neighbor_position);
                if neighbor_block == &Block::TWISTING_VINES
                    || neighbor_block == &Block::TWISTING_VINES_PLANT
                {
                    return Block::TWISTING_VINES_PLANT.default_state.id;
                }
            }
        } else {
            if args.direction == BlockDirection::Down
                && !<Self as PlantBlockBase>::can_place_at(self, args.world, args.position)
            {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
            }
            if args.direction == BlockDirection::Up {
                let neighbor_block = args.world.get_block(args.neighbor_position);
                if neighbor_block != &Block::TWISTING_VINES
                    && neighbor_block != &Block::TWISTING_VINES_PLANT
                {
                    let mut props = TwistingVinesProperties::default(&Block::TWISTING_VINES);
                    props.age = rand::rng().random_range(0..25);
                    return props.to_state_id(&Block::TWISTING_VINES);
                }
            }
        }
        args.state_id
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        Self::get_head_pos(args.world, args.position, args.block).is_some_and(|head_pos| {
            let growth_pos = head_pos.up();
            args.world.is_in_build_limit(growth_pos)
                && args.world.get_block_state(&growth_pos).is_air()
        })
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        if let Some(head_pos) = Self::get_head_pos(args.world, args.position, args.block) {
            let head_state_id = args.world.get_block_state_id(&head_pos);
            let mut next_age =
                (TwistingVinesProperties::from_state_id(head_state_id).age + 1).min(25);
            let blocks_to_grow = Self::get_blocks_to_grow_when_bonemealed();
            let mut forward_pos = head_pos.up();
            for _ in 0..blocks_to_grow {
                if !args.world.is_in_build_limit(forward_pos)
                    || !args.world.get_block_state(&forward_pos).is_air()
                {
                    break;
                }
                let mut props = TwistingVinesProperties::default(&Block::TWISTING_VINES);
                props.age = next_age;
                args.world.set_block_state(
                    &forward_pos,
                    props.to_state_id(&Block::TWISTING_VINES),
                    BlockFlags::NOTIFY_ALL,
                );
                forward_pos = forward_pos.up();
                next_age = (next_age + 1).min(25);
            }
        }
    }
}

impl PlantBlockBase for TwistingVinesBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let (support_block, support_block_state) = block_accessor.get_block_and_state(pos);

        if support_block == &Block::TWISTING_VINES || support_block == &Block::TWISTING_VINES_PLANT
        {
            return true;
        }
        support_block_state.is_side_solid(BlockDirection::Up)
    }
}
