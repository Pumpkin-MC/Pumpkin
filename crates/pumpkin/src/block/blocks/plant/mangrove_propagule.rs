use std::sync::Arc;

use pumpkin_data::{
    Block, BlockStateId,
    block_properties::{BlockProperties, MangrovePropaguleLikeProperties},
    tag,
    tag::Taggable,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::{
    BlockBehaviour, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    RandomTickArgs,
};
use crate::world::World;

pub const MAX_AGE: u8 = 4;

#[pumpkin_block("mangrove_propagule")]
pub struct MangrovePropaguleBlock;

impl MangrovePropaguleBlock {
    #[must_use]
    pub const fn is_hanging(props: &MangrovePropaguleLikeProperties) -> bool {
        props.hanging
    }

    #[must_use]
    pub const fn is_fully_grown(props: &MangrovePropaguleLikeProperties) -> bool {
        props.age == MAX_AGE
    }

    #[must_use]
    pub fn can_survive(
        world: &dyn BlockAccessor,
        pos: &BlockPos,
        props: &MangrovePropaguleLikeProperties,
    ) -> bool {
        if props.hanging {
            let above_pos = pos.up();
            let above_block = world.get_block(&above_pos);
            above_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_HANGING_MANGROVE_PROPAGULE)
        } else {
            let below_pos = pos.down();
            let below_block = world.get_block(&below_pos);
            below_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_MANGROVE_PROPAGULE)
        }
    }

    #[must_use]
    pub fn create_new_hanging_propagule(age: u8) -> BlockStateId {
        let mut props = MangrovePropaguleLikeProperties::default(&Block::MANGROVE_PROPAGULE);
        props.hanging = true;
        props.age = age.min(MAX_AGE);
        props.to_state_id(&Block::MANGROVE_PROPAGULE)
    }

    fn advance_tree(
        world: &Arc<World>,
        pos: &BlockPos,
        block: &Block,
        mut props: MangrovePropaguleLikeProperties,
    ) {
        if props.stage == 0 {
            props.stage = 1;
            world.set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL);
        } else {
            use crate::plugin::api::events::world::structure_grow::{StructureGrowEvent, TreeType};
            let mut event = StructureGrowEvent::new(*pos, TreeType::Mangrove, false);
            if let Some(server) = world.server.upgrade() {
                server.plugin_manager.fire_blocking(&server, &mut event);
            }
        }
    }
}

impl BlockBehaviour for MangrovePropaguleBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let state_id = args.block_accessor.get_block_state_id(args.position);
        let props = MangrovePropaguleLikeProperties::from_state_id(state_id, args.block);
        Self::can_survive(args.block_accessor, args.position, &props)
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props =
            MangrovePropaguleLikeProperties::from_state_id(args.block.default_state.id, args.block);
        props.hanging = false;
        props.age = MAX_AGE;
        props.stage = 0;
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = MangrovePropaguleLikeProperties::from_state_id(args.state_id, args.block);
        if !Self::can_survive(args.world, args.position, &props) {
            return Block::AIR.default_state.id;
        }
        args.state_id
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let mut props = MangrovePropaguleLikeProperties::from_state_id(state_id, args.block);
        if !props.hanging {
            if rand::random::<u8>().is_multiple_of(7) {
                Self::advance_tree(args.world, args.position, args.block, props);
            }
        } else if props.age < MAX_AGE {
            props.age += 1;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
        }
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        let props = MangrovePropaguleLikeProperties::from_state_id(args.state_id, args.block);
        !props.hanging || props.age < MAX_AGE
    }

    fn is_bonemeal_success(&self, args: BonemealArgs<'_>) -> bool {
        let props = MangrovePropaguleLikeProperties::from_state_id(args.state_id, args.block);
        if props.hanging {
            props.age < MAX_AGE
        } else {
            rand::random::<f32>() < 0.45
        }
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        {
            let mut props =
                MangrovePropaguleLikeProperties::from_state_id(args.state_id, args.block);
            if props.hanging && props.age < MAX_AGE {
                props.age += 1;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );
            } else {
                Self::advance_tree(args.world, args.position, args.block, props);
            }
        }
    }
}
