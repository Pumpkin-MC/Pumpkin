use crate::{
    block::{
        BlockBehaviour, BlockMetadata, GetStateForNeighborUpdateArgs, OnPlaceArgs,
        OnScheduledTickArgs, PlacedArgs,
    },
    entity::falling::FallingEntity,
};
use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockState, BlockStateId,
    tag::{self, Taggable},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;

pub struct FallingBlock;

impl FallingBlock {
    #[must_use]
    pub fn can_fall_through(state: &BlockState, block: &Block) -> bool {
        state.is_air()
            || block.has_tag(&tag::Block::MINECRAFT_FIRE)
            || state.is_liquid()
            || state.replaceable()
    }

    #[must_use]
    pub fn touches_liquid(world: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        for dir in BlockDirection::all() {
            if dir == BlockDirection::Down {
                continue;
            }
            let neighbor = world.get_block_state(&pos.offset(dir.to_offset()));
            if neighbor.is_liquid() {
                return true;
            }
        }
        false
    }
}

impl BlockMetadata for FallingBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::GRAVEL,
            BlockId::SAND,
            BlockId::RED_SAND,
            BlockId::SUSPICIOUS_SAND,
            BlockId::SUSPICIOUS_GRAVEL,
            BlockId::WHITE_CONCRETE_POWDER,
            BlockId::ORANGE_CONCRETE_POWDER,
            BlockId::MAGENTA_CONCRETE_POWDER,
            BlockId::LIGHT_BLUE_CONCRETE_POWDER,
            BlockId::YELLOW_CONCRETE_POWDER,
            BlockId::LIME_CONCRETE_POWDER,
            BlockId::PINK_CONCRETE_POWDER,
            BlockId::GRAY_CONCRETE_POWDER,
            BlockId::LIGHT_GRAY_CONCRETE_POWDER,
            BlockId::CYAN_CONCRETE_POWDER,
            BlockId::PURPLE_CONCRETE_POWDER,
            BlockId::BLUE_CONCRETE_POWDER,
            BlockId::BROWN_CONCRETE_POWDER,
            BlockId::GREEN_CONCRETE_POWDER,
            BlockId::RED_CONCRETE_POWDER,
            BlockId::BLACK_CONCRETE_POWDER,
        ]
        .into()
    }
}

impl BlockBehaviour for FallingBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if args.block.has_tag(&tag::Block::MINECRAFT_CONCRETE_POWDERS)
            && Self::touches_liquid(args.world, args.position)
            && let Some(name) = args.block.name.strip_suffix("_powder")
            && let Some(concrete) = Block::from_name(name)
        {
            return concrete.default_state.id;
        }
        args.block.default_state.id
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        args.world
            .schedule_block_tick(args.block, *args.position, 2, TickPriority::Normal);
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.block.has_tag(&tag::Block::MINECRAFT_CONCRETE_POWDERS)
            && Self::touches_liquid(args.world, args.position)
            && let Some(name) = args.block.name.strip_suffix("_powder")
            && let Some(concrete) = Block::from_name(name)
        {
            return concrete.default_state.id;
        }

        args.world
            .schedule_block_tick(args.block, *args.position, 2, TickPriority::Normal);
        args.state_id
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let (block, state) = args.world.get_block_and_state(&args.position.down());
        if !Self::can_fall_through(state, block) || args.position.0.y < args.world.min_y {
            return;
        }
        let state = args.world.get_block_state(args.position);
        FallingEntity::replace_spawn(args.world, *args.position, state.id);
    }
}
