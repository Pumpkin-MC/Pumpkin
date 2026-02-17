use pumpkin_data::{damage::DamageType, effect::StatusEffect};

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
                let effect = pumpkin_data::potion::Effect {
                    effect_type: &StatusEffect::WITHER,
                    duration: 40,
                    amplifier: 1,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: true,
                };
                if let Some(player) = args.entity.get_player() {
                    player.send_effect(effect.clone()).await;
                };
                living_entity.add_effect(effect).await;
                //FIXME: player should receive damage every half second. somewhy executing below code gives like 1 damage per second.
                args.entity
                    .damage(args.entity, 1.0, DamageType::WITHER)
                    .await;
            }
        })
    }
}
