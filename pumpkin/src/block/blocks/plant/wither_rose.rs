use crate::block::{GetStateForNeighborUpdateArgs, blocks::plant::PlantBlockBase};
use pumpkin_data::{
    effect::StatusEffect,
    entity::EntityType,
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::Difficulty;
use pumpkin_world::BlockStateId;

use crate::block::{BlockBehaviour, BlockFuture, CanPlaceAtArgs, OnEntityCollisionArgs};
#[pumpkin_block("minecraft:wither_rose")]
pub struct WitherRoseBlock;

impl BlockBehaviour for WitherRoseBlock {
    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(living_entity) = args.entity.get_living_entity() {
                if args.world.level_info.load().difficulty == Difficulty::Peaceful {
                    return;
                }
                let entity_type = args.entity.get_entity().entity_type;
                if entity_type == &EntityType::ENDER_DRAGON
                    || entity_type == &EntityType::WITHER
                    || entity_type == &EntityType::WITHER_SKELETON
                {
                    return;
                }
                let effect = pumpkin_data::potion::Effect {
                    effect_type: &StatusEffect::WITHER,
                    duration: 40,
                    amplifier: 0,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: true,
                };
                if let Some(player) = args.entity.get_player() {
                    player.send_effect(effect.clone()).await;
                }
                living_entity.add_effect(effect).await;
            }
        })
    }
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
        })
    }
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            <Self as PlantBlockBase>::get_state_for_neighbor_update(
                self,
                args.world,
                args.position,
                args.state_id,
            )
            .await
        })
    }
}

impl PlantBlockBase for WitherRoseBlock {
    async fn can_plant_on_top(
        &self,
        block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        let support_block = block_accessor.get_block(pos).await;
        support_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_WITHER_ROSE)
    }
}
