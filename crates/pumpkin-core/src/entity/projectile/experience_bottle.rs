use std::sync::RwLock;
use std::sync::atomic::AtomicBool;

use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::world::WorldEvent;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use rand::RngExt;

use crate::entity::experience_orb::ExperienceOrbEntity;
use crate::entity::projectile::{ProjectileHit, ThrownItemEntity};
use crate::entity::{Entity, EntityBase};
use crate::server::Server;

const GRAVITY: f64 = 0.07;
const SPLASH_COLOR: i32 = -13_083_194;

pub struct ExperienceBottleEntity {
    pub thrown: ThrownItemEntity,
    item_stack: RwLock<ItemStack>,
}

impl ExperienceBottleEntity {
    pub fn new(entity: Entity) -> Self {
        Self {
            thrown: ThrownItemEntity {
                entity,
                owner_id: None,
                collides_with_projectiles: false,
                has_hit: AtomicBool::new(false),
                gravity: GRAVITY,
            },
            item_stack: RwLock::new(ItemStack::new(1, &Item::EXPERIENCE_BOTTLE)),
        }
    }

    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        Self {
            thrown: ThrownItemEntity::new(entity, shooter, GRAVITY),
            item_stack: RwLock::new(ItemStack::new(1, &Item::EXPERIENCE_BOTTLE)),
        }
    }

    pub fn set_item_stack(&self, stack: ItemStack) {
        *self
            .item_stack
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = stack;
    }
}

impl EntityBase for ExperienceBottleEntity {
    fn get_entity(&self) -> &Entity {
        &self.thrown.entity
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn get_owner_id(&self) -> Option<i32> {
        self.thrown.owner_id
    }

    fn init_data_tracker(&self) {
        let stack = self
            .item_stack
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::experience_bottle::ITEM_STACK,
            ItemStackSerializer::from(stack.clone()),
        );
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        self.thrown.process_tick(caller);
    }

    fn on_hit(&self, hit: ProjectileHit) {
        let entity = self.get_entity();
        entity.set_pos(hit.hit_pos());
        let world = entity.world.load();
        let position = entity.block_pos.load();
        world.sync_world_event(
            WorldEvent::ParticlesSpellPotionSplash,
            position,
            SPLASH_COLOR,
        );
        if !entity.is_silent() {
            world.sync_world_event(WorldEvent::SoundSpellPotionSplash, position, 0);
        }
        let mut random = rand::rng();
        let amount = 3 + random.random_range(0..5) + random.random_range(0..5);
        let direction = match &hit {
            ProjectileHit::Block { face, .. } => {
                let offset = face.to_offset();
                pumpkin_util::math::vector3::Vector3::new(
                    f64::from(offset.x),
                    f64::from(offset.y),
                    f64::from(offset.z),
                )
            }
            ProjectileHit::Entity { .. } => entity.velocity.load().multiply(-1.0, -1.0, -1.0),
        };
        ExperienceOrbEntity::spawn_with_direction(&world, hit.hit_pos(), direction, amount);
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
