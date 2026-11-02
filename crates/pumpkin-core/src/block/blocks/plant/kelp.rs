use crate::block::blocks::plant::PlantBlockBase;
use crate::block::{
    BlockBehaviour, BlockMetadata, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnPlaceArgs, OnScheduledTickArgs, RandomTickArgs,
};
use crate::world::World;
use pumpkin_data::block_properties::{KelpProperties, WaterLikeProperties};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockId, BlockStateId, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;

pub struct KelpBlock;

impl BlockMetadata for KelpBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::KELP, BlockId::KELP_PLANT].into()
    }
}

impl KelpBlock {
    fn get_head_pos(world: &World, pos: &BlockPos, block: &Block) -> Option<BlockPos> {
        if block == &Block::KELP {
            return Some(*pos);
        }
        let mut cursor = pos.up();
        while world.is_in_build_limit(cursor) {
            let current = world.get_block(&cursor);
            if current == &Block::KELP_PLANT {
                cursor = cursor.up();
            } else if current == &Block::KELP {
                return Some(cursor);
            } else {
                return None;
            }
        }
        None
    }
}

impl BlockBehaviour for KelpBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if args.block == &Block::KELP {
            let above_block = args.world.get_block(&args.position.up());
            if above_block == &Block::KELP || above_block == &Block::KELP_PLANT {
                return Block::KELP_PLANT.default_state.id;
            }
            let mut props = KelpProperties::default(&Block::KELP);
            props.age = rand::rng().random_range(0..25);
            props.to_state_id(&Block::KELP)
        } else {
            Block::KELP_PLANT.default_state.id
        }
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        if args.block == &Block::KELP {
            let state_id = args.world.get_block_state_id(args.position);
            let mut props = KelpProperties::from_state_id(state_id);
            if props.age < 25 && rand::rng().random::<f64>() < 0.14 {
                let growth_pos = args.position.up();
                if args.world.get_block(&growth_pos) == &Block::WATER {
                    props.age += 1;
                    args.world.set_block_state(
                        &growth_pos,
                        props.to_state_id(&Block::KELP),
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
        if args.block == &Block::KELP {
            if args.direction == BlockDirection::Down {
                if <Self as PlantBlockBase>::can_place_at(self, args.world, args.position) {
                    let above_block = args.world.get_block(&args.position.up());
                    if above_block == &Block::KELP || above_block == &Block::KELP_PLANT {
                        return Block::KELP_PLANT.default_state.id;
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
                if neighbor_block == &Block::KELP || neighbor_block == &Block::KELP_PLANT {
                    return Block::KELP_PLANT.default_state.id;
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
                if neighbor_block != &Block::KELP && neighbor_block != &Block::KELP_PLANT {
                    let mut props = KelpProperties::default(&Block::KELP);
                    props.age = rand::rng().random_range(0..25);
                    return props.to_state_id(&Block::KELP);
                }
            }
        }
        args.world.schedule_fluid_tick(
            &Fluid::WATER,
            *args.position,
            Fluid::WATER.flow_speed as u8,
            TickPriority::Normal,
        );
        args.state_id
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        Self::get_head_pos(args.world, args.position, args.block).is_some_and(|head_pos| {
            let growth_pos = head_pos.up();
            args.world.is_in_build_limit(growth_pos)
                && args.world.get_block(&growth_pos) == &Block::WATER
        })
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        if let Some(head_pos) = Self::get_head_pos(args.world, args.position, args.block) {
            let head_state_id = args.world.get_block_state_id(&head_pos);
            let next_age = (KelpProperties::from_state_id(head_state_id).age + 1).min(25);
            let forward_pos = head_pos.up();
            if args.world.is_in_build_limit(forward_pos)
                && args.world.get_block(&forward_pos) == &Block::WATER
            {
                let mut props = KelpProperties::default(&Block::KELP);
                props.age = next_age;
                args.world.set_block_state(
                    &forward_pos,
                    props.to_state_id(&Block::KELP),
                    BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }
}

impl PlantBlockBase for KelpBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let support_pos = pos;
        let (replacing_block, replacing_block_state) =
            block_accessor.get_block_and_state(&pos.up());
        let (support_block, support_block_state) = block_accessor.get_block_and_state(support_pos);
        if replacing_block == &Block::WATER {
            let water_props = WaterLikeProperties::from_state_id(replacing_block_state.id);
            if water_props.level != 0 && water_props.level != 8 {
                return false;
            }
        } else if replacing_block != &Block::KELP_PLANT && replacing_block != &Block::KELP {
            return false;
        }
        if support_block.has_tag(&tag::Block::MINECRAFT_CANNOT_SUPPORT_KELP) {
            return false;
        }
        if support_block == &Block::KELP || support_block == &Block::KELP_PLANT {
            return true;
        }
        support_block_state.is_side_solid(BlockDirection::Up)
    }
}
