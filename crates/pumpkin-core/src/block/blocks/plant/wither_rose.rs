use crate::block::{GetStateForNeighborUpdateArgs, blocks::plant::PlantBlockBase};
use pumpkin_data::BlockStateId;
use pumpkin_data::{
    effect::StatusEffect,
    entity::EntityType,
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::Difficulty;

use crate::block::{BlockBehaviour, CanPlaceAtArgs, OnEntityCollisionArgs};
#[pumpkin_block("minecraft:wither_rose")]
pub struct WitherRoseBlock;

impl BlockBehaviour for WitherRoseBlock {
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
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
            living_entity.add_effect(effect);
        }
    }
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }
    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        <Self as PlantBlockBase>::get_state_for_neighbor_update(
            self,
            args.world,
            args.position,
            args.state_id,
        )
    }
}

impl PlantBlockBase for WitherRoseBlock {
    fn can_plant_on_top(
        &self,
        block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        let support_block = block_accessor.get_block(pos);
        support_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_WITHER_ROSE)
    }
}
