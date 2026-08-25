use uuid::Uuid;

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

use crate::entity::effect::{EffectFuture, MobEffect};
use crate::entity::living::LivingEntity;
use crate::entity::r#type::from_type;

pub struct InfestedMobEffect;

impl MobEffect for InfestedMobEffect {
    fn on_mob_hurt<'a>(
        &'a self,
        living: &'a LivingEntity,
        _amplifier: u8,
        _damage_type: &'a DamageType,
        _damage_amount: f32,
    ) -> EffectFuture<'a, ()> {
        Box::pin(async move {
            // Wither, ender dragon and silverfish are immune
            if living.entity.entity_type == &EntityType::WITHER
                || living.entity.entity_type == &EntityType::ENDER_DRAGON
                || living.entity.entity_type == &EntityType::SILVERFISH
            {
                return;
            }

            let world = living.entity.world.load();

            // 10% chance to spawn
            if rand::random::<f32>() <= 0.1 {
                let count = rand::random::<u32>() % 2 + 1;
                let bbox = living.entity.bounding_box.load();
                let center = Vector3::new(
                    f64::midpoint(bbox.min.x, bbox.max.x),
                    f64::midpoint(bbox.min.y, bbox.max.y),
                    f64::midpoint(bbox.min.z, bbox.max.z),
                );

                let rot = living.entity.rotation();
                let vx = rot.x * 0.3;
                let vy = rot.y * 0.45;
                let vz = rot.z * 0.3;

                for _ in 0..count {
                    let random_angle = (rand::random::<f32>() - 0.5) * std::f32::consts::PI;
                    let cos_a = random_angle.cos();
                    let sin_a = random_angle.sin();
                    let rx = vx * cos_a + vz * sin_a;
                    let rz = -vx * sin_a + vz * cos_a;

                    let silver = from_type(&EntityType::SILVERFISH, center, &world, Uuid::new_v4());

                    let entity = silver.get_entity();
                    entity.set_pos(center);
                    entity.yaw.store(rand::random::<f32>() * 360.0);
                    entity.pitch.store(0.0);
                    entity.velocity.store(Vector3::new(
                        f64::from(rx),
                        f64::from(vy),
                        f64::from(rz),
                    ));

                    world.spawn_entity(silver).await;
                    world.play_sound(Sound::EntitySilverfishHurt, SoundCategory::Hostile, &center);
                }
            }
        })
    }
}
