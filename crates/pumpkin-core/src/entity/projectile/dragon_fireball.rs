use std::sync::Arc;

use pumpkin_data::{entity::EntityType, world::WorldEvent};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    entity::{
        Entity, EntityBase,
        area_effect_cloud::AreaEffectCloudEntity,
        boss::ender_dragon::EnderDragonPart,
        living::LivingEntity,
        projectile::{HurtingProjectile, ProjectileHit, ThrownItemEntity, owner::ProjectileOwner},
    },
    server::Server,
};

pub const SPLASH_RANGE: f64 = 4.0;
const SPLASH_RANGE_Y: f64 = 2.0;
const SPLASH_RANGE_SQ: f64 = SPLASH_RANGE * SPLASH_RANGE;

pub struct DragonFireballEntity {
    pub thrown: HurtingProjectile,
}

impl DragonFireballEntity {
    #[must_use]
    pub const fn new(entity: Entity) -> Self {
        let thrown = ThrownItemEntity {
            entity,
            owner: ProjectileOwner::none(),
            gravity: 0.0,
        };
        Self {
            thrown: HurtingProjectile::new(thrown, HurtingProjectile::DEFAULT_ACCELERATION_POWER),
        }
    }

    /// Mirrors `new DragonFireball(level, mob, direction)` followed by positioning by the caller.
    #[must_use]
    pub fn new_shot(entity: Entity, shooter: &Entity, direction: Vector3<f64>) -> Self {
        let thrown = HurtingProjectile::new(
            ThrownItemEntity::owned(entity, shooter, 0.0),
            HurtingProjectile::DEFAULT_ACCELERATION_POWER,
        );
        thrown.assign_directional_movement(direction);
        Self { thrown }
    }

    fn is_owned_entity(&self, other: &dyn EntityBase) -> bool {
        if self.thrown.owner.matches(other.get_entity()) {
            return true;
        }
        if let Some(part) = other.cast_any().downcast_ref::<EnderDragonPart>() {
            return self.thrown.owner.uuid() == Some(part.dragon_uuid);
        }
        false
    }
}

impl EntityBase for DragonFireballEntity {
    fn can_hit(&self) -> bool {
        super::is_redirectable(self.get_entity())
    }

    fn get_projectile_owner(&self) -> Option<&ProjectileOwner> {
        Some(&self.thrown.owner)
    }

    fn on_deflection(&self, by_attack: bool) {
        self.thrown.on_deflection(by_attack);
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        self.thrown.write_nbt(nbt);
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        self.thrown.read_nbt(nbt);
    }

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        self.thrown.process_tick(
            caller,
            server,
            HurtingProjectile::INERTIA,
            HurtingProjectile::LIQUID_INERTIA,
            false,
            |other| !self.is_owned_entity(other),
        );
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) {
        if let ProjectileHit::Entity { ref entity, .. } = hit
            && self.is_owned_entity(entity.as_ref())
        {
            return;
        }

        let entity = self.get_entity();
        let world = entity.world.load();
        let fireball_pos = entity.pos.load();
        let search_box =
            entity
                .bounding_box
                .load()
                .expand(SPLASH_RANGE, SPLASH_RANGE_Y, SPLASH_RANGE);

        let mut cloud_pos = fireball_pos;
        let mut candidates = world.get_entities_at_box(&search_box);
        for player in world.get_players_at_box(&search_box) {
            candidates.push(player as Arc<dyn EntityBase>);
        }

        for candidate in candidates {
            if candidate.get_living_entity().is_none()
                || !candidate.get_entity().is_alive()
                || candidate.is_spectator()
                || self.is_owned_entity(candidate.as_ref())
            {
                continue;
            }
            let cand_pos = candidate.get_entity().pos.load();
            if fireball_pos.squared_distance_to_vec(&cand_pos) < SPLASH_RANGE_SQ {
                cloud_pos = cand_pos;
                break;
            }
        }

        let owner = self.thrown.owner.get(&world);
        let living_owner = owner
            .as_ref()
            .filter(|o| o.get_living_entity().is_some())
            .map(|o| o.get_entity());

        let cloud_entity = Entity::new(world.clone(), cloud_pos, &EntityType::AREA_EFFECT_CLOUD);
        let cloud = AreaEffectCloudEntity::create_dragon_breath(cloud_entity, living_owner);

        let splash_data = if entity.is_silent() { -1 } else { 1 };
        world.sync_world_event(
            WorldEvent::ParticlesDragonFireballSplash,
            entity.block_pos.load(),
            splash_data,
        );
        world.spawn_entity(cloud);
    }
}
