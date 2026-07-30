use std::sync::Arc;

use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage,
        area_effect_cloud::AreaEffectCloudEntity,
        projectile::{HurtingProjectileEntity, ProjectileHit},
    },
    server::Server,
};
use pumpkin_data::{effect::StatusEffect, item::Item, item_stack::ItemStack, world::WorldEvent};

/// `net.minecraft.entity.projectile.DragonFireballEntity`: on hit, doesn't explode -
/// instead spawns a growing dragon-breath `AreaEffectCloudEntity` (instant damage).
pub struct DragonFireballEntity {
    pub hurting: HurtingProjectileEntity,
}

impl DragonFireballEntity {
    #[must_use]
    pub const fn new(entity: Entity) -> Self {
        let hurting = HurtingProjectileEntity {
            entity,
            owner_id: None,
            acceleration_power: HurtingProjectileEntity::DEFAULT_ACCELERATION_POWER,
            has_hit: std::sync::atomic::AtomicBool::new(false),
            left_owner: std::sync::atomic::AtomicBool::new(false),
        };
        Self { hurting }
    }

    #[must_use]
    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        Self {
            hurting: HurtingProjectileEntity::new(
                entity,
                shooter,
                HurtingProjectileEntity::DEFAULT_ACCELERATION_POWER,
            ),
        }
    }
}

impl NBTStorage for DragonFireballEntity {}

impl EntityBase for DragonFireballEntity {
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move { self.hurting.process_tick(caller, server).await })
    }

    fn get_entity(&self) -> &Entity {
        &self.hurting.entity
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            // Java: `hitResult.getType() != ENTITY || !isOwner(entity)` - hitting our own
            // owner does not spawn a breath cloud.
            if let ProjectileHit::Entity { ref entity, .. } = hit
                && Some(entity.get_entity().entity_id) == self.hurting.owner_id
            {
                return;
            }

            let entity = self.get_entity();
            let world = entity.world.load();
            let hit_pos = hit.hit_pos();

            let search_box = entity.bounding_box.load().expand(4.0, 2.0, 4.0);
            let nearby_living: Vec<_> = world
                .get_entities_at_box(&search_box)
                .into_iter()
                .filter(|e| e.get_living_entity().is_some() && !e.is_spectator())
                .collect();

            // Java: first living entity within 16 blocks (squared) of the impact point
            // gets the cloud centred on it instead of the impact position.
            let mut cloud_pos = hit_pos;
            for candidate in &nearby_living {
                let cand_pos = candidate.get_entity().pos.load();
                if hit_pos.squared_distance_to_vec(&cand_pos) < 16.0 {
                    cloud_pos = cand_pos;
                    break;
                }
            }

            let cloud_entity = Entity::new(
                world.clone(),
                cloud_pos,
                &pumpkin_data::entity::EntityType::AREA_EFFECT_CLOUD,
            );
            let cloud = AreaEffectCloudEntity::create(
                cloud_entity,
                ItemStack::new(0, &Item::DRAGON_BREATH),
                vec![(&StatusEffect::INSTANT_DAMAGE, 1, 0, false, true, true)],
                600, // duration
                3.0, // radius
                20,  // reapplication delay (default)
                20,  // wait time (default)
                0.0, // radius on use (default, Java does not set this)
                0,   // duration on use (default, Java does not set this)
            );
            // `create()` defaults to a shrinking cloud; the dragon's breath cloud grows
            // from 3 to 7 blocks over its lifetime (`radiusGrowth = (7 - radius) / duration`).
            if let Some(cloud_ref) = cloud.cast_any().downcast_ref::<AreaEffectCloudEntity>() {
                *cloud_ref.radius_on_tick.lock().await = (7.0 - 3.0) / 600.0;
                // Java: `lv.setParticleType(DragonBreathParticleEffect.of(ParticleTypes.DRAGON_BREATH, 1.0F))`
                // DragonBreathParticleEffect encodes as: particle ID + float power
                let power_bytes = 1.0f32.to_be_bytes();
                *cloud_ref.custom_particle.lock().await = Some((
                    pumpkin_data::particle::Particle::DragonBreath as i32,
                    power_bytes.to_vec(),
                ));
            }

            world.sync_world_event(
                WorldEvent::ParticlesDragonFireballSplash,
                entity.block_pos.load(),
                0,
            );
            world.play_sound(
                pumpkin_data::sound::Sound::EntityDragonFireballExplode,
                pumpkin_data::sound::SoundCategory::Hostile,
                &entity.pos.load(),
            );
            world.spawn_entity(cloud).await;
            entity.remove().await;
        })
    }
}
