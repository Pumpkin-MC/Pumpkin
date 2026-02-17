use pumpkin_data::{effect::StatusEffect, entity::EntityType};
use pumpkin_util::Difficulty;

use crate::block::{Block, BlockBehaviour, BlockFuture, BlockMetadata, OnEntityCollisionArgs};

pub struct WitherRose;

impl BlockMetadata for WitherRose {
    fn ids() -> Box<[u16]> {
        [Block::WITHER_ROSE.id].into()
    }
}

impl BlockBehaviour for WitherRose {
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
}
