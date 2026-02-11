use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;

use crate::block::{
    BlockBehaviour, BlockFuture, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
};

use super::FireBlockBase;
use crate::block::OnEntityCollisionArgs;
use crate::entity::EntityBase;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use rand::RngExt;
use std::sync::atomic::Ordering;

#[pumpkin_block("minecraft:soul_fire")]
pub struct SoulFireBlock;

impl SoulFireBlock {
    #[must_use]
    pub fn is_soul_base(block: &Block) -> bool {
        block.has_tag(&tag::Block::MINECRAFT_SOUL_FIRE_BASE_BLOCKS)
    }
}

impl BlockBehaviour for SoulFireBlock {
    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let base_entity = args.entity.get_entity();
            if !base_entity.entity_type.fire_immune
                && !base_entity.fire_immune.load(Ordering::Relaxed)
            {
                let ticks = base_entity.fire_ticks.load(Ordering::Relaxed);

                // Timer logic
                if ticks < 0 {
                    base_entity.fire_ticks.store(ticks + 1, Ordering::Relaxed);
                } else if base_entity.entity_type == &EntityType::PLAYER {
                    let rnd_ticks = rand::rng().random_range(1..3);
                    base_entity
                        .fire_ticks
                        .store(ticks + rnd_ticks, Ordering::Relaxed);
                }

                // Apply fire ticks
                if base_entity.fire_ticks.load(Ordering::Relaxed) >= 0 {
                    base_entity.set_on_fire_for(8.0);
                }

                // Apply extra soul fire damage (and item damage)
                base_entity
                    .damage(args.entity, 1.0, DamageType::IN_FIRE)
                    .await;
            }
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !Self::is_soul_base(args.world.get_block(&args.position.down()).await) {
                return Block::AIR.default_state.id;
            }

            args.state_id
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            Self::is_soul_base(args.block_accessor.get_block(&args.position.down()).await)
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            FireBlockBase::broken(args.world, *args.position).await;
        })
    }
}
