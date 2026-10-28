use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockState, BlockStateId,
    entity::EntityType,
    tag::{self, Taggable},
    world::WorldEvent,
};
use pumpkin_util::{Difficulty, math::position::BlockPos};
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

use crate::block::entities::skull::SkullBlockEntity;
use crate::{
    block::{
        BlockBehaviour, BlockMetadata, OnNeighborUpdateArgs, OnPlaceArgs, PathComputationType,
        PlacedArgs, PlayerPlacedArgs, blocks::skull_block::SkullBlock,
    },
    entity::{Entity, boss::wither::WitherEntity},
    world::World,
};

pub struct WitherPattern {
    blocks: [BlockPos; 7],
    center: BlockPos,
}

#[must_use]
pub fn find_wither_pattern(world: &Arc<World>, skull_pos: &BlockPos) -> Option<WitherPattern> {
    if skull_pos.0.y < world.min_y || world.level_info.load().difficulty == Difficulty::Peaceful {
        return None;
    }

    let is_soul_block =
        |block: &Block| block.has_tag(&tag::Block::MINECRAFT_WITHER_SUMMON_BASE_BLOCKS);
    let is_skull = |pos: &BlockPos| {
        if pos == skull_pos {
            return true;
        }
        let block = world.get_block(pos);
        block == &Block::WITHER_SKELETON_SKULL || block == &Block::WITHER_SKELETON_WALL_SKULL
    };

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
            let under_arm1 = arm1.down();
            let under_arm2 = arm2.down();

            if is_soul_block(world.get_block(&top_middle))
                && is_soul_block(world.get_block(&base))
                && is_soul_block(world.get_block(&arm1))
                && is_soul_block(world.get_block(&arm2))
                && is_skull(&center)
                && is_skull(&skull1_pos)
                && is_skull(&skull2_pos)
                && world.get_block_state(&under_arm1).is_air()
                && world.get_block_state(&under_arm2).is_air()
            {
                return Some(WitherPattern {
                    blocks: [center, skull1_pos, skull2_pos, top_middle, arm1, arm2, base],
                    center: base,
                });
            }
        }
    }

    None
}

fn spawn_wither(world: &Arc<World>, pattern: &WitherPattern) {
    for pos in pattern.blocks {
        let state_id = world.get_block_state_id(&pos);
        world.set_block_state(
            &pos,
            Block::AIR.default_state.id,
            BlockFlags::NOTIFY_LISTENERS,
        );
        world.sync_world_event(
            WorldEvent::ParticlesDestroyBlock,
            pos,
            i32::from(state_id.as_u16()),
        );
    }

    let mut spawn_pos = pattern.center.to_centered_f64();
    spawn_pos.y = f64::from(pattern.center.0.y) + 0.55;
    let entity = Entity::new(world.clone(), spawn_pos, &EntityType::WITHER);
    let wither = WitherEntity::new(entity);
    wither.make_invulnerable();
    world.spawn_entity(wither);

    for pos in pattern.blocks {
        world.update_neighbors(&pos, None);
    }
}

pub struct WitherSkeletonSkullBlock;

impl BlockMetadata for WitherSkeletonSkullBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::WITHER_SKELETON_SKULL,
            BlockId::WITHER_SKELETON_WALL_SKULL,
        ]
        .into()
    }
}

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

    fn player_placed(&self, args: PlayerPlacedArgs<'_>) {
        if find_wither_pattern(args.world, args.position).is_some() {
            args.player.trigger_advancement(
                crate::entity::player::advancement::trigger::AdvancementTrigger::SummonedEntity {
                    entity_type: "minecraft:wither".to_string(),
                },
            );
        }
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        SkullBlock::on_neighbor_update(&SkullBlock, args);
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
