use std::sync::atomic::AtomicBool;

use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

use crate::{
    entity::{
        Entity, EntityBase,
        area_effect_cloud::AreaEffectCloudEntity,
        living::LivingEntity,
        projectile::{ProjectileHit, ThrownItemEntity, fireball::INITIAL_ACCELERATION_POWER},
    },
    server::Server,
};

const GRAVITY: f64 = 0.0;
const SPLASH_RANGE_SQ: f64 = 16.0; // 4 blocks radius

pub struct DragonFireballEntity {
    pub thrown: ThrownItemEntity,
}

impl DragonFireballEntity {
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

impl EntityBase for DragonFireballEntity {
    fn get_owner_id(&self) -> Option<i32> {
        self.thrown.owner_id
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        self.thrown.process_tick(caller);
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
        let hit_pos = hit.hit_pos();
        let world = self.get_entity().world.load();

        // 1. Play explosion sound
        world.play_sound(
            Sound::EntityDragonFireballExplode,
            SoundCategory::Hostile,
            &hit_pos,
        );

        // 2. Spawn dragon breath particles
        world.spawn_particle(
            hit_pos,
            Vector3::new(1.0, 1.0, 1.0),
            0.1,
            50,
            Particle::DragonBreath,
        );

        // 3. Check for nearby entities in splash range
        let mut cloud_pos = hit_pos;
        for player in world.players.load().iter() {
            let p_pos = player.living_entity.entity.pos.load();
            let dx = p_pos.x - hit_pos.x;
            let dy = (p_pos.y - hit_pos.y).abs();
            let dz = p_pos.z - hit_pos.z;
            if dx * dx + dz * dz < SPLASH_RANGE_SQ && dy <= 2.0 {
                cloud_pos = p_pos;
                let _ = player.damage(self, 6.0, DamageType::DRAGON_BREATH);
                break;
            }
        }

        // 4. Create and spawn Dragon Breath AreaEffectCloud
        let cloud_entity = Entity::new(world.clone(), cloud_pos, &EntityType::AREA_EFFECT_CLOUD);
        let cloud = AreaEffectCloudEntity::create(
            cloud_entity,
            ItemStack::new(0, &Item::DRAGON_BREATH),
            vec![(&StatusEffect::INSTANT_DAMAGE, 1, 1, false, true, true)],
            600,
            3.0,
            20,
            20,
            0.5,
            -100,
        );
        world.spawn_entity(cloud);

        // 5. Remove projectile
        self.get_entity().remove();
    }
}
