use uuid::Uuid;

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::effect::{EffectFuture, MobEffect};
use crate::entity::living::LivingEntity;
use crate::entity::mob::slime::SlimeEntity;
use crate::entity::r#type::from_type;

pub struct OozingMobEffect;

impl MobEffect for OozingMobEffect {
    fn on_mob_death<'a>(
        &'a self,
        living: &'a LivingEntity,
        _amplifier: u8,
        _damage_type: &'a DamageType,
    ) -> EffectFuture<'a, ()> {
        Box::pin(async move {
            // Slimes are immune
            if living.entity.entity_type == &EntityType::SLIME {
                return;
            }

            let world = living.entity.world.load();
            let pos = living.entity.pos.load();
            let spawn_pos = Vector3::new(pos.x, pos.y + 0.5, pos.z);

            // Spawns 2 slimes of size 2 (medium slimes)
            for _ in 0..2 {
                let entity_arc = from_type(&EntityType::SLIME, spawn_pos, &world, Uuid::new_v4());
                let entity = entity_arc.get_entity();
                entity.set_pos(spawn_pos);
                entity.yaw.store(rand::random::<f32>() * 360.0);
                entity.pitch.store(0.0);

                if let Some(slime) = entity_arc.cast_any().downcast_ref::<SlimeEntity>() {
                    slime.set_size(2, true);
                }

                world.spawn_entity(entity_arc).await;
            }
        })
    }
}
