use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use crate::plugin::player::egg_throw::PlayerEggThrowEvent;
use crate::{
    entity::{Entity, EntityBase, projectile::ThrownItemEntity, r#type::from_type},
    server::Server,
};
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

const MAX_EGG_HATCH_EVENT_SPAWNS: usize = 16;
const GRAVITY: f64 = 0.03;

pub struct EggEntity {
    pub thrown: ThrownItemEntity,
    pub item_stack: RwLock<ItemStack>,
}

impl EggEntity {
    pub fn new(entity: Entity) -> Self {
        // Default velocity slightly upward for thrown egg
        entity.set_velocity(Vector3::new(0.0, 0.1, 0.0));
        let thrown = ThrownItemEntity {
            entity,
            owner_id: None,
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
            gravity: GRAVITY,
        };

        Self {
            thrown,
            item_stack: RwLock::new(ItemStack::new(1, &Item::EGG)),
        }
    }

    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        // Default slight upward velocity
        thrown.entity.set_velocity(Vector3::new(0.0, 0.1, 0.0));

        Self {
            thrown,
            item_stack: RwLock::new(ItemStack::new(1, &Item::EGG)),
        }
    }

    /// Set the item stack shown by this thrown egg
    pub fn set_item_stack(&self, item_stack: ItemStack) {
        let mut write = self
            .item_stack
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *write = item_stack;
    }
}

impl EntityBase for EggEntity {
    fn init_data_tracker(&self) {
        let entity = self.get_entity();
        let stack = self
            .item_stack
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        // Sync the item stack so the client renders the correct color/variant
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::egg::ITEM_STACK,
                &ItemStackSerializer::from(stack.clone()),
            )],
            None,
        );
    }

    fn tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>, server: &'a Server) {
        self.thrown.process_tick(caller, server);
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }
    fn on_hit(&self, hit: crate::entity::projectile::ProjectileHit) {
        let world = self.get_entity().world.load();
        let hit_pos = hit.hit_pos();
        let normal = hit.normal();

        // Chicken spawn position offset slightly from hit position
        let spawn_pos = hit_pos.add(&normal.multiply(0.5, 0.5, 0.5));

        // Play egg break particles
        world.send_entity_status(
            self.get_entity(),
            EntityStatus::Death,
            Some(ActorEventID::Death),
        );

        // Decide spawn count per probabilities:
        // r == 0 -> spawn 4 (1/256)
        // r in 1..31 -> spawn 1 (31/256)
        // else -> 0
        let r: u8 = rand::random(); // 0..=255
        let mut to_spawn = if r == 0 { 4usize } else { usize::from(r < 32) };
        let mut hatching = to_spawn > 0;
        let mut hatching_type: &'static EntityType = &EntityType::CHICKEN;

        let owner_id = self.thrown.owner_id;
        let entity_uuid = self.get_entity().entity_uuid;
        let variant_name = {
            let stack = self
                .item_stack
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            stack
                .get_data_component::<pumpkin_data::data_component_impl::ChickenVariantImpl>()
                .map(|comp| comp.value.clone())
        };

        if let Some(owner_id) = owner_id
            && let Some(player) = world.get_player_by_id(owner_id)
            && let Some(server) = world.server.upgrade()
        {
            let mut event = PlayerEggThrowEvent::new(
                player,
                entity_uuid,
                hatching,
                to_spawn as u8,
                hatching_type,
            );
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                hatching = false;
            } else {
                hatching = event.hatching;
                to_spawn = (event.num_hatches as usize).min(MAX_EGG_HATCH_EVENT_SPAWNS);
                hatching_type = event.hatching_type;
            }
        }

        if hatching && to_spawn > 0 {
            for _ in 0..to_spawn {
                let mob = from_type(hatching_type, spawn_pos, &world, Uuid::new_v4());

                let yaw = rand::random::<f32>() * 360.0;
                let new_entity = mob.get_entity();
                new_entity.set_rotation(yaw, 0.0);
                new_entity.set_age(-24000);
                if let Some(name) = &variant_name {
                    mob.set_variant_name(name);
                }

                world.spawn_entity(mob);
            }
        }
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
