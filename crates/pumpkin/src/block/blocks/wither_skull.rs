use std::sync::Arc;

use pumpkin_data::{Block, BlockDirection, BlockStateId, entity::EntityType, world::WorldEvent};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::entities::skull::SkullBlockEntity;
use crate::{
    block::{BlockBehaviour, OnPlaceArgs, PlacedArgs, blocks::skull_block::SkullBlock},
    entity::{Entity, boss::wither::WitherEntity},
    world::World,
};

pub struct WitherPattern {
    blocks: [BlockPos; 7],
    center: BlockPos,
}

#[must_use]
pub fn find_wither_pattern(world: &Arc<World>, skull_pos: &BlockPos) -> Option<WitherPattern> {
    let is_soul_block = |block: &Block| block == &Block::SOUL_SAND || block == &Block::SOUL_SOIL;
    let is_skull =
        |pos: &BlockPos| pos == skull_pos || world.get_block(pos) == &Block::WITHER_SKELETON_SKULL;

    for dir in [BlockDirection::North, BlockDirection::West] {
        let opposite = dir.opposite();
        for center in [
            *skull_pos,
            skull_pos.offset(opposite.to_offset()),
            skull_pos.offset(dir.to_offset()),
        ] {
            let top_middle = center.down();
            let base = top_middle.down();
            let arm1 = top_middle.offset(dir.to_offset());
            let arm2 = top_middle.offset(opposite.to_offset());
            let skull1_pos = arm1.up();
            let skull2_pos = arm2.up();

            if is_soul_block(world.get_block(&top_middle))
                && is_soul_block(world.get_block(&base))
                && is_soul_block(world.get_block(&arm1))
                && is_soul_block(world.get_block(&arm2))
                && is_skull(&center)
                && is_skull(&skull1_pos)
                && is_skull(&skull2_pos)
            {
                return Some(WitherPattern {
                    blocks: [center, skull1_pos, skull2_pos, top_middle, arm1, arm2, base],
                    center: top_middle,
                });
            }
        }
    }

    None
}

fn spawn_wither(world: &Arc<World>, pattern: &WitherPattern) {
    for pos in pattern.blocks {
        world.set_block_state(&pos, Block::AIR.default_state.id, BlockFlags::NOTIFY_ALL);
        world.sync_world_event(
            WorldEvent::ParticlesDestroyBlock,
            pos,
            Block::SOUL_SAND.default_state.id.as_u16().into(),
        );
    }

    let entity = Entity::new(
        world.clone(),
        pattern.center.to_centered_f64(),
        &EntityType::WITHER,
    );
    let wither = WitherEntity::new(entity);
    wither.make_invulnerable();
    world.spawn_entity(wither);
}

#[pumpkin_block("wither_skeleton_skull")]
pub struct WitherSkeletonSkullBlock;

impl BlockBehaviour for WitherSkeletonSkullBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        SkullBlock::on_place(&SkullBlock, args)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        args.world
            .add_block_entity(Arc::new(SkullBlockEntity::new(*args.position)));

        if let Some(pattern) = find_wither_pattern(args.world, args.position) {
            spawn_wither(args.world, &pattern);
        }
    }
}
