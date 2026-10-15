use std::sync::atomic::AtomicBool;

use crate::{
    entity::{
        Entity, EntityBase, experience_orb::ExperienceOrbEntity, projectile::ThrownItemEntity,
    },
    server::Server,
};
use pumpkin_protocol::java::client::play::CWorldEvent;
use pumpkin_util::math::{
    position::BlockPos,
    vector2::{Vector2, to_chunk_pos},
    vector3::Vector3,
};

const GRAVITY: f64 = 0.07;
const POTION_SPLASH_EVENT: i32 = 2002;
const POTION_SPLASH_SOUND_EVENT: i32 = 1053;
const EXP_BOTTLE_POTION_COLOR: i32 = -13_083_194;

pub struct ExperienceBottleEntity {
    pub thrown: ThrownItemEntity,
}

impl ExperienceBottleEntity {
    #[must_use]
    pub fn new(entity: Entity) -> Self {
        entity.set_velocity(Vector3::new(0.0, 0.1, 0.0));
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
        thrown.entity.set_velocity(Vector3::new(0.0, 0.1, 0.0));
        Self { thrown }
    }
}

impl EntityBase for ExperienceBottleEntity {
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

    fn on_hit(&self, hit: crate::entity::projectile::ProjectileHit) {
        let world = self.get_entity().world.load();
        let hit_pos = hit.hit_pos();

        let block_pos = BlockPos(Vector3::new(
            hit_pos.x.floor() as i32,
            hit_pos.y.floor() as i32,
            hit_pos.z.floor() as i32,
        ));
        let chunk_pos = to_chunk_pos(&Vector2::new(block_pos.0.x, block_pos.0.z));

        // Vanilla ThrownExperienceBottle.java: levelEvent 2002 with color -13083194
        world.broadcast_to_chunk(
            chunk_pos,
            &CWorldEvent::new(POTION_SPLASH_EVENT, block_pos, EXP_BOTTLE_POTION_COLOR, false),
        );

        // Vanilla: levelEvent 1053 (potion splash sound) if not silent
        if !self.get_entity().is_silent() {
            world.broadcast_to_chunk(
                chunk_pos,
                &CWorldEvent::new(POTION_SPLASH_SOUND_EVENT, block_pos, 0, false),
            );
        }

        // Vanilla ThrownExperienceBottle.java: 3 + random.nextInt(5) + random.nextInt(5)
        let r1 = rand::random::<u32>() % 5;
        let r2 = rand::random::<u32>() % 5;
        let xp_amount = 3 + r1 + r2;

        ExperienceOrbEntity::spawn(&world, hit_pos, xp_amount);
    }
}
