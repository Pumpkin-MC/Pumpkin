use std::sync::atomic::AtomicBool;

use pumpkin_util::math::vector3::Vector3;

use crate::{
    entity::{
        Entity, EntityBase,
        projectile::{ProjectileHit, ThrownItemEntity, fireball::INITIAL_ACCELERATION_POWER},
    },
    server::Server,
};

const GRAVITY: f64 = 0.0;

pub struct SmallFireballEntity {
    pub thrown: ThrownItemEntity,
}

impl SmallFireballEntity {
    #[must_use]
    pub const fn new(entity: Entity) -> Self {
        let thrown = ThrownItemEntity {
            entity,
            owner_id: None,
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
            gravity: GRAVITY,
        };

        Self { thrown }
    }

    #[must_use]
    pub fn new_shot(entity: Entity, shooter: &Entity, direction: Vector3<f64>) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        let accel = INITIAL_ACCELERATION_POWER;
        thrown
            .entity
            .velocity
            .store(direction.normalize().multiply(accel, accel, accel));
        Self { thrown }
    }
}

impl EntityBase for SmallFireballEntity {
    fn get_owner_id(&self) -> Option<i32> {
        self.thrown.owner_id
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        self.thrown.process_tick(caller);
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) {
        let world = self.get_entity().world.load();
        let owner = self
            .thrown
            .owner_id
            .and_then(|id| world.get_entity_by_id(id));

        match hit {
            ProjectileHit::Entity {
                ref entity,
                hit_pos,
                ..
            } => {
                entity.get_entity().set_on_fire_for(5.0);
                let attacker_ref: &dyn EntityBase = owner.as_deref().unwrap_or(self);
                let _ = entity.damage_with_context(
                    attacker_ref,
                    5.0,
                    pumpkin_data::damage::DamageType::FIREBALL,
                    Some(hit_pos),
                    Some(self.get_entity()),
                    owner.as_deref(),
                );
            }
            ProjectileHit::Block { pos, face, .. } => {
                let block_to_place = pos.offset(face.to_offset());
                if world.get_block_state(&block_to_place).is_air() {
                    let fire_state = pumpkin_data::Block::FIRE.default_state.id;
                    world.set_block_state(
                        &block_to_place,
                        fire_state,
                        pumpkin_world::world::BlockFlags::NOTIFY_ALL,
                    );
                }
            }
        }
    }
}
