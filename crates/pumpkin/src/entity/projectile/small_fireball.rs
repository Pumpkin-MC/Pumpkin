use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::{
    entity::{
        Entity, EntityBase,
        projectile::{ProjectileHit, ThrownItemEntity},
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
    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        Self { thrown }
    }
}

impl EntityBase for SmallFireballEntity {
    fn tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>, server: &'a Server) {
        self.thrown.process_tick(caller, server);
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
        match hit {
            ProjectileHit::Entity { ref entity, .. } => {
                entity.get_entity().set_on_fire_for(5.0);
                let _ = entity.damage(
                    entity.as_ref(),
                    5.0,
                    pumpkin_data::damage::DamageType::FIREBALL,
                );
            }
            ProjectileHit::Block { pos, face, .. } => {
                // Try to place fire
                let block_to_place = match face {
                    pumpkin_data::BlockDirection::Up => pos.up(),
                    pumpkin_data::BlockDirection::Down => pos.down(),
                    pumpkin_data::BlockDirection::North => pos.north(),
                    pumpkin_data::BlockDirection::South => pos.south(),
                    pumpkin_data::BlockDirection::West => pos.west(),
                    pumpkin_data::BlockDirection::East => pos.east(),
                };
                let world = self.get_entity().world.load();
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
