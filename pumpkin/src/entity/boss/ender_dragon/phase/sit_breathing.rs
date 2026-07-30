use super::EnderDragonPhase;
use crate::entity::{
    Entity, area_effect_cloud::AreaEffectCloudEntity, boss::ender_dragon::EnderDragonEntity,
};
use futures::future::BoxFuture;
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;

pub struct SitFlamingPhase;

impl super::Phase for SitFlamingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::SittingFlaming
    }

    fn is_sitting_or_hovering(&self) -> bool {
        true
    }

    fn begin<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            *dragon.breathing_timer.lock().await = 0;
            *dragon.sitting_flaming_times_run.lock().await += 1;
            *dragon.target_location.lock().await = None;
            *dragon.breath_cloud.lock().await = None;
        })
    }

    fn end<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let cloud_uuid = *dragon.breath_cloud.lock().await;
            if let Some(uuid) = cloud_uuid {
                let world = dragon.mob_entity.living_entity.entity.world.load();
                if let Some(entity) = world
                    .entities
                    .load()
                    .iter()
                    .find(|e| e.get_entity().entity_uuid == uuid)
                {
                    entity.get_entity().remove().await;
                }
                *dragon.breath_cloud.lock().await = None;
            }
        })
    }

    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            // Java never repositions the dragon here either; it just stays put.
            let mut timer = dragon.breathing_timer.lock().await;
            *timer += 1;

            if *timer >= 200 {
                let times_run = *dragon.sitting_flaming_times_run.lock().await;
                *timer = 0;
                drop(timer);
                if times_run >= 4 {
                    dragon.set_phase(EnderDragonPhase::Takeoff).await;
                } else {
                    dragon.set_phase(EnderDragonPhase::SittingScanning).await;
                }
                return;
            }

            if *timer == 10 {
                let entity = &dragon.mob_entity.living_entity.entity;
                let pos = entity.pos.load();
                let world = entity.world.load();

                let head_pos = dragon.parts[0].entity.pos.load();
                let dir_x = head_pos.x - pos.x;
                let dir_z = head_pos.z - pos.z;
                let dir_len = dir_x.hypot(dir_z);
                let (dir_x, dir_z) = if dir_len > 1e-6 {
                    (dir_x / dir_len, dir_z / dir_len)
                } else {
                    (0.0, 0.0)
                };

                let cloud_x = head_pos.x + dir_x * 2.5;
                let cloud_z = head_pos.z + dir_z * 2.5;
                let head_y = head_pos.y;

                let mut ground_y = head_y;
                let mut check_y = head_y as i32;
                while check_y >= 0 {
                    let check_pos = pumpkin_util::math::position::BlockPos::new(
                        cloud_x as i32,
                        check_y,
                        cloud_z as i32,
                    );
                    if !world.get_block(&check_pos).is_air() {
                        ground_y = check_y as f64 + 1.0;
                        break;
                    }
                    check_y -= 1;
                }

                let cloud_pos = Vector3::new(cloud_x, ground_y, cloud_z);
                let cloud_entity =
                    Entity::new(world.clone(), cloud_pos, &EntityType::AREA_EFFECT_CLOUD);
                let cloud = AreaEffectCloudEntity::create(
                    cloud_entity,
                    pumpkin_data::item_stack::ItemStack::new(
                        0,
                        &pumpkin_data::item::Item::DRAGON_BREATH,
                    ),
                    vec![(
                        &pumpkin_data::effect::StatusEffect::INSTANT_DAMAGE,
                        0, // amplifier (Java default is 0, not 1)
                        0,
                        false,
                        true,
                        true,
                    )],
                    200, // duration
                    5.0, // radius
                    20,  // reapplication delay (default)
                    20,  // wait time (default)
                    0.0, // radius on use (default, Java does not set this)
                    0,   // duration on use (default, Java does not set this)
                );
                let cloud_uuid = cloud.get_entity().entity_uuid;
                *dragon.breath_cloud.lock().await = Some(cloud_uuid);
                // Java: `setRadiusGrowth` is NOT called → stays at radius 5.0
                if let Some(cloud_ref) = cloud.cast_any().downcast_ref::<AreaEffectCloudEntity>() {
                    *cloud_ref.radius_on_tick.lock().await = 0.0;
                    // Java: `lv.setParticleType(DragonBreathParticleEffect.of(ParticleTypes.DRAGON_BREATH, 1.0F))`
                    let power_bytes = 1.0f32.to_be_bytes();
                    *cloud_ref.custom_particle.lock().await = Some((
                        pumpkin_data::particle::Particle::DragonBreath as i32,
                        power_bytes.to_vec(),
                    ));
                }
                world.spawn_entity(cloud).await;
            }
        })
    }
}
