use crate::block::blocks::plant::PlantBlockBase;
use crate::block::blocks::plant::big_dripleaf::{BigDripleafBlock, is_water_source};
use crate::block::{
    BlockBehaviour, BonemealArgs, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnPlaceArgs, PlacedArgs,
};
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{Axis, DoubleBlockHalf, SmallDripleafLikeProperties};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

#[pumpkin_block("minecraft:small_dripleaf")]
pub struct SmallDripleafBlock;

impl SmallDripleafBlock {
    fn can_survive(
        &self,
        block_accessor: &dyn BlockAccessor,
        pos: &BlockPos,
        state_id: BlockStateId,
    ) -> bool {
        let props = SmallDripleafLikeProperties::from_state_id(state_id);
        if props.half == DoubleBlockHalf::Upper {
            let (below_block, below_state) = block_accessor.get_block_and_state(&pos.down());
            return below_block == &Block::SMALL_DRIPLEAF
                && SmallDripleafLikeProperties::from_state_id(below_state.id).half
                    == DoubleBlockHalf::Lower;
        }
        <Self as PlantBlockBase>::can_plant_on_top(self, block_accessor, &pos.down())
    }
}

impl BlockBehaviour for SmallDripleafBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let props = SmallDripleafLikeProperties::from_state_id(args.state.id);
        if props.half == DoubleBlockHalf::Upper {
            return self.can_survive(args.block_accessor, args.position, args.state.id);
        }

        if args.player.is_some() {
            let up_pos = args.position.up();
            if let Some(world) = args.world
                && up_pos.0.y > world.get_top_y()
            {
                return false;
            }
            let upper_state = args.block_accessor.get_block_state(&up_pos);
            if !upper_state.replaceable() {
                return false;
            }
        }

        self.can_survive(args.block_accessor, args.position, args.state.id)
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();
        let mut small_dripleaf_props = SmallDripleafLikeProperties::default(args.block);

        small_dripleaf_props.facing = facing;
        small_dripleaf_props.waterlogged = args.replacing.water_source();
        small_dripleaf_props.half = DoubleBlockHalf::Lower;

        small_dripleaf_props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = SmallDripleafLikeProperties::from_state_id(args.state_id);
        if props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }

        if args.direction.to_axis() == Axis::Y
            && (props.half == DoubleBlockHalf::Lower) == (args.direction == BlockDirection::Up)
        {
            let neighbor_block = args.world.get_block(args.neighbor_position);
            if neighbor_block != &Block::SMALL_DRIPLEAF
                || SmallDripleafLikeProperties::from_state_id(args.neighbor_state_id).half
                    == props.half
            {
                return Block::AIR.default_state.id;
            }
        }

        if props.half == DoubleBlockHalf::Lower
            && args.direction == BlockDirection::Down
            && !self.can_survive(args.world, args.position, args.state_id)
        {
            return Block::AIR.default_state.id;
        }

        if !self.can_survive(args.world, args.position, args.state_id) {
            return Block::AIR.default_state.id;
        }

        args.state_id
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let lower_small_dripleaf_props = SmallDripleafLikeProperties::from_state_id(args.state_id);
        if lower_small_dripleaf_props.half != DoubleBlockHalf::Lower {
            return;
        }

        let mut upper_small_dripleaf_props =
            SmallDripleafLikeProperties::default(&Block::SMALL_DRIPLEAF);

        let up_pos = args.position.up();
        upper_small_dripleaf_props.facing = lower_small_dripleaf_props.facing;
        upper_small_dripleaf_props.waterlogged = is_water_source(args.world.as_ref(), &up_pos);
        upper_small_dripleaf_props.half = DoubleBlockHalf::Upper;

        args.world.set_block_state(
            &up_pos,
            upper_small_dripleaf_props.to_state_id(&Block::SMALL_DRIPLEAF),
            BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
        );
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        let props = SmallDripleafLikeProperties::from_state_id(args.state.id);
        let other_block_pos = match props.half {
            DoubleBlockHalf::Upper => args.position.down(),
            DoubleBlockHalf::Lower => args.position.up(),
        };
        let (other_block, other_state_id) = args.world.get_block_and_state_id(&other_block_pos);
        if other_block == &Block::SMALL_DRIPLEAF {
            let other_props = SmallDripleafLikeProperties::from_state_id(other_state_id);
            if other_props.half != props.half {
                args.world.break_block(
                    &other_block_pos,
                    None,
                    BlockFlags::SKIP_DROPS
                        | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK
                        | BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }

    fn is_valid_bonemeal_target(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        let props = SmallDripleafLikeProperties::from_state_id(args.state_id);
        if props.half == DoubleBlockHalf::Lower {
            let above = args.position.up();
            let above_state = if is_water_source(args.world.as_ref(), &above) {
                Block::WATER.default_state.id
            } else {
                Block::AIR.default_state.id
            };
            args.world.set_block_state(
                &above,
                above_state,
                BlockFlags::NOTIFY_LISTENERS | BlockFlags::FORCE_STATE,
            );
            BigDripleafBlock::place_with_random_height(args.world, args.position, props.facing);
        } else {
            let below_pos = args.position.down();
            let (below_block, below_state_id) = args.world.get_block_and_state_id(&below_pos);
            if below_block == &Block::SMALL_DRIPLEAF {
                self.perform_bonemeal(BonemealArgs {
                    world: args.world,
                    block: below_block,
                    position: &below_pos,
                    state_id: below_state_id,
                });
            }
        }
    }
}

impl PlantBlockBase for SmallDripleafBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let support_block = block_accessor.get_block(pos);
        if support_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_SMALL_DRIPLEAF) {
            return true;
        }
        is_water_source(block_accessor, &pos.up())
            && support_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_VEGETATION)
    }
}
