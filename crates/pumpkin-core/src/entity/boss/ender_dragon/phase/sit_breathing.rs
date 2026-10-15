use super::EnderDragonPhase;
use crate::entity::{
    Entity, area_effect_cloud::AreaEffectCloudEntity, boss::ender_dragon::EnderDragonEntity,
};
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct SitBreathingPhase;

impl super::Phase for SitBreathingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::SittingFlaming
    }

    fn is_sitting(&self) -> bool {
        true
    }

    fn begin(&self, dragon: &EnderDragonEntity) {
        *dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        *dragon
            .breathing_timer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
        *dragon
            .flame_count
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) += 1;
    }

    fn tick(&self, dragon: &EnderDragonEntity) {
        let mut timer = dragon
            .breathing_timer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *timer += 1;
        let flame_ticks = *timer;
        drop(timer);

        if flame_ticks >= 200 {
            let count = *dragon
                .flame_count
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if count >= 4 {
                dragon.set_phase(EnderDragonPhase::Takeoff);
            } else {
                dragon.set_phase(EnderDragonPhase::SittingScanning);
            }
            return;
        }

        if flame_ticks == 10 {
            let entity = &dragon.mob_entity.living_entity.entity;
            let world = entity.world.load();
            let head_pos = dragon.parts[0].entity.pos.load();
            let dragon_pos = entity.pos.load();

            let mut look = Vector3::new(head_pos.x - dragon_pos.x, 0.0, head_pos.z - dragon_pos.z);
            if look.length_squared() > 1e-6 {
                look = look.normalize();
            }

            let cloud_x = head_pos.x + look.x * 2.5;
            let cloud_z = head_pos.z + look.z * 2.5;
            let mut cloud_y = head_pos.y;

            // Search downwards for ground level
            let initial_y = cloud_y;
            let mut check_pos = BlockPos::new(cloud_x.round() as i32, cloud_y.round() as i32, cloud_z.round() as i32);
            while world.get_block(&check_pos).is_air() {
                cloud_y -= 1.0;
                if cloud_y < 0.0 {
                    cloud_y = initial_y;
                    break;
                }
                check_pos = BlockPos::new(cloud_x.round() as i32, cloud_y.round() as i32, cloud_z.round() as i32);
            }
            cloud_y = cloud_y.floor() + 1.0;

            let cloud_pos = Vector3::new(cloud_x, cloud_y, cloud_z);
            let cloud_entity = Entity::new(world.clone(), cloud_pos, &EntityType::AREA_EFFECT_CLOUD);
            let cloud = AreaEffectCloudEntity::create(
                cloud_entity,
                pumpkin_data::item_stack::ItemStack::new(
                    0,
                    &pumpkin_data::item::Item::DRAGON_BREATH,
                ),
                vec![(
                    &pumpkin_data::effect::StatusEffect::INSTANT_DAMAGE,
                    1,
                    0,
                    false,
                    true,
                    true,
                )],
                200, // duration 200 ticks
                5.0, // radius 5.0
                20,  // reapplication delay
                20,  // wait time
                0.5, // radius on use
                -100, // duration on use
            );
            world.spawn_entity(cloud);
        }
    }
}
