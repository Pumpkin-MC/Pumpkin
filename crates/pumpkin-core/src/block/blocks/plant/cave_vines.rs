use pumpkin_data::block_properties::{CaveVinesLikeProperties, CaveVinesPlantLikeProperties};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection, BlockId, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;

use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockMetadata, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    NormalUseArgs, OnPlaceArgs, OnScheduledTickArgs, RandomTickArgs,
};

pub struct CaveVinesBlock;

impl BlockMetadata for CaveVinesBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::CAVE_VINES, BlockId::CAVE_VINES_PLANT].into()
    }
}

impl CaveVinesBlock {
    #[must_use]
    pub fn can_survive(world: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let support_pos = pos.up();
        let (support_block, support_state) = world.get_block_and_state(&support_pos);
        if support_block == &Block::CAVE_VINES || support_block == &Block::CAVE_VINES_PLANT {
            return true;
        }
        support_state.is_side_solid(BlockDirection::Down)
    }
}

impl BlockBehaviour for CaveVinesBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        Self::can_survive(args.block_accessor, args.position)
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if args.block == &Block::CAVE_VINES {
            let below_block = args.world.get_block(&args.position.down());
            if below_block == &Block::CAVE_VINES || below_block == &Block::CAVE_VINES_PLANT {
                return Block::CAVE_VINES_PLANT.default_state.id;
            }
            let mut props = CaveVinesLikeProperties::default(&Block::CAVE_VINES);
            props.age = rand::rng().random_range(0..25);
            props.to_state_id(&Block::CAVE_VINES)
        } else {
            Block::CAVE_VINES_PLANT.default_state.id
        }
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        if args.block == &Block::CAVE_VINES {
            let state_id = args.world.get_block_state_id(args.position);
            let mut props = CaveVinesLikeProperties::from_state_id(state_id);
            if props.age < 25 && rand::rng().random::<f64>() < 0.1 {
                let growth_pos = args.position.down();
                if args.world.get_block_state(&growth_pos).is_air() {
                    props.age += 1;
                    props.berries = rand::rng().random::<f32>() < 0.11;
                    args.world.set_block_state(
                        &growth_pos,
                        props.to_state_id(&Block::CAVE_VINES),
                        BlockFlags::NOTIFY_ALL,
                    );
                }
            }
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !Self::can_survive(args.world.as_ref(), args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.block == &Block::CAVE_VINES {
            if args.direction == BlockDirection::Up {
                if Self::can_survive(args.world, args.position) {
                    let below_block = args.world.get_block(&args.position.down());
                    if below_block == &Block::CAVE_VINES || below_block == &Block::CAVE_VINES_PLANT
                    {
                        let head_props = CaveVinesLikeProperties::from_state_id(args.state_id);
                        let mut body_props =
                            CaveVinesPlantLikeProperties::default(&Block::CAVE_VINES_PLANT);
                        body_props.berries = head_props.berries;
                        return body_props.to_state_id(&Block::CAVE_VINES_PLANT);
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
            if args.direction == BlockDirection::Down {
                let neighbor_block = args.world.get_block(args.neighbor_position);
                if neighbor_block == &Block::CAVE_VINES
                    || neighbor_block == &Block::CAVE_VINES_PLANT
                {
                    let head_props = CaveVinesLikeProperties::from_state_id(args.state_id);
                    let mut body_props =
                        CaveVinesPlantLikeProperties::default(&Block::CAVE_VINES_PLANT);
                    body_props.berries = head_props.berries;
                    return body_props.to_state_id(&Block::CAVE_VINES_PLANT);
                }
            }
        } else {
            if args.direction == BlockDirection::Up && !Self::can_survive(args.world, args.position)
            {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
            }
            if args.direction == BlockDirection::Down {
                let neighbor_block = args.world.get_block(args.neighbor_position);
                if neighbor_block != &Block::CAVE_VINES
                    && neighbor_block != &Block::CAVE_VINES_PLANT
                {
                    let body_props = CaveVinesPlantLikeProperties::from_state_id(args.state_id);
                    let mut head_props = CaveVinesLikeProperties::default(&Block::CAVE_VINES);
                    head_props.age = rand::rng().random_range(0..25);
                    head_props.berries = body_props.berries;
                    return head_props.to_state_id(&Block::CAVE_VINES);
                }
            }
        }
        args.state_id
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        if args.block == &Block::CAVE_VINES {
            let mut props = CaveVinesLikeProperties::from_state_id(state_id);
            if props.berries {
                props.berries = false;
                args.world
                    .drop_stack(args.position, ItemStack::new(1, &Item::GLOW_BERRIES));
                let pitch = rand::rng().random_range(0.8f32..1.2f32);
                args.world.play_sound_fine(
                    Sound::BlockCaveVinesPickBerries,
                    SoundCategory::Blocks,
                    &args.position.to_centered_f64(),
                    1.0,
                    pitch,
                );
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );
                return BlockActionResult::SuccessServer;
            }
        } else if args.block == &Block::CAVE_VINES_PLANT {
            let mut props = CaveVinesPlantLikeProperties::from_state_id(state_id);
            if props.berries {
                props.berries = false;
                args.world
                    .drop_stack(args.position, ItemStack::new(1, &Item::GLOW_BERRIES));
                let pitch = rand::rng().random_range(0.8f32..1.2f32);
                args.world.play_sound_fine(
                    Sound::BlockCaveVinesPickBerries,
                    SoundCategory::Blocks,
                    &args.position.to_centered_f64(),
                    1.0,
                    pitch,
                );
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );
                return BlockActionResult::SuccessServer;
            }
        }
        BlockActionResult::Pass
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        if args.block == &Block::CAVE_VINES {
            !CaveVinesLikeProperties::from_state_id(args.state_id).berries
        } else if args.block == &Block::CAVE_VINES_PLANT {
            !CaveVinesPlantLikeProperties::from_state_id(args.state_id).berries
        } else {
            false
        }
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        if args.block == &Block::CAVE_VINES {
            let mut props = CaveVinesLikeProperties::from_state_id(args.state_id);
            props.berries = true;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
        } else if args.block == &Block::CAVE_VINES_PLANT {
            let mut props = CaveVinesPlantLikeProperties::from_state_id(args.state_id);
            props.berries = true;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
        }
    }
}
