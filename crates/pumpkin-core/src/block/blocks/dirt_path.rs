use crate::block::{
    BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs, PathComputationType,
};
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block("minecraft:dirt_path")]
pub struct DirtPathBlock;

impl BlockBehaviour for DirtPathBlock {
    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        push_entities_up(args.world, args.position);
        args.world.set_block_state(
            args.position,
            Block::DIRT.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if !can_place_at(args.world, args.position) {
            return Block::DIRT.default_state.id;
        }

        args.block.default_state.id
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction == BlockDirection::Up && !can_place_at(args.world, args.position) {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
        }
        args.state_id
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.position)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

fn can_place_at(world: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    let (block, state) = world.get_block_and_state(&block_pos.up());
    !state.is_solid() || block.has_tag(&tag::Block::MINECRAFT_FENCE_GATES)
}

pub fn push_entities_up(world: &crate::world::World, block_pos: &BlockPos) {
    let aabb = BoundingBox::new(
        Vector3::new(
            block_pos.0.x as f64,
            block_pos.0.y as f64 + 0.9,
            block_pos.0.z as f64,
        ),
        Vector3::new(
            block_pos.0.x as f64 + 1.0,
            block_pos.0.y as f64 + 1.1,
            block_pos.0.z as f64 + 1.0,
        ),
    );
    for entity in world.get_entities_at_box(&aabb) {
        let pos = entity.get_entity().pos.load();
        if pos.y < block_pos.0.y as f64 + 1.0 {
            entity
                .get_entity()
                .set_pos(Vector3::new(pos.x, block_pos.0.y as f64 + 1.0, pos.z));
        }
    }
}
