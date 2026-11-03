use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::entity::experience_orb::ExperienceOrbEntity;
use crate::entity::projectile::owner::ProjectileOwner;
use crate::entity::projectile::{ProjectileHit, ThrownItemEntity};
use crate::entity::{Entity, EntityBase};
use crate::server::Server;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::world::WorldEvent;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

const GRAVITY: f64 = 0.07;
const WATER_COLOR: i32 = -13_083_194; // 0xFF385DC6

pub struct ExperienceBottleEntity {
    pub thrown: ThrownItemEntity,
    pub item_stack: RwLock<ItemStack>,
}

impl ExperienceBottleEntity {
    pub fn new(entity: Entity) -> Self {
        Self {
            thrown: ThrownItemEntity {
                entity,
                owner: ProjectileOwner::none(),
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

    pub fn set_item_stack(&self, item_stack: ItemStack) {
        let mut write = self
            .item_stack
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *write = item_stack;
    }
}

impl EntityBase for ExperienceBottleEntity {
    fn get_projectile_owner(&self) -> Option<&ProjectileOwner> {
        Some(&self.thrown.owner)
    }

    fn init_data_tracker(&self) {
        let entity = self.get_entity();
        let stack = self
            .item_stack
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        entity.set_synced_data(
            pumpkin_data::tracked_data::experience_bottle::ITEM_STACK,
            ItemStackSerializer::from(stack.clone()),
        );
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

    fn on_hit(&self, hit: ProjectileHit) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let hit_pos = hit.hit_pos();
        let block_pos = BlockPos::floored(hit_pos.x, hit_pos.y, hit_pos.z);

        world.sync_world_event(
            WorldEvent::ParticlesSpellPotionSplash,
            block_pos,
            WATER_COLOR,
        );
        if !entity.silent.load(Ordering::Relaxed) {
            world.sync_world_event(WorldEvent::SoundSpellPotionSplash, block_pos, 0);
        }

        let mut rng = rand::rng();
        let xp_count = 3 + rng.random_range(0..5) + rng.random_range(0..5);
        ExperienceOrbEntity::spawn(&world, hit_pos, xp_count);
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
