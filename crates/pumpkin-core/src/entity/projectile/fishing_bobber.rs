use crate::entity::projectile::owner::ProjectileOwner;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crate::entity::experience_orb::ExperienceOrbEntity;
use crate::entity::item::ItemEntity;
use crate::entity::projectile::{
    ProjectileHit, get_hit_result_on_move_vector, hit_target_or_deflect_self,
};
use crate::world::loot::{LootContextParameters, LootTableHandle, generate_loot_from_handle};
use crate::{
    entity::{Entity, EntityBase, living::LivingEntity, player::Player},
    server::Server,
};
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_protocol::bedrock::server::ActorEventID;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::clip::ClipBlock;
use rand::RngExt;

pub struct FishingBobberEntity {
    pub entity: Entity,
    pub owner: ProjectileOwner,
    pub hooked_entity_id: AtomicI32,
    pub in_ground: AtomicBool,
    pub has_hit: AtomicBool,
    pub wait_countdown: AtomicI32,
    pub bite_countdown: AtomicI32,
}

impl FishingBobberEntity {
    const WATER_INERTIA: f64 = 0.8;
    const AIR_INERTIA: f64 = 0.92;
    const GRAVITY: f64 = 0.03;

    pub fn new(entity: Entity, owner: &Player) -> Self {
        Self::new_with_rod(entity, owner, 0)
    }

    pub fn new_with_rod(entity: Entity, owner: &Player, time_reduction: i32) -> Self {
        let mut owner_pos = owner.living_entity.entity.pos.load();
        owner_pos.y += owner.living_entity.entity.get_eye_height() - 0.1;
        entity.set_pos(owner_pos);

        let wait_ticks = (rand::rng().random_range(100..=600) - time_reduction).max(0);

        Self {
            entity,
            owner: ProjectileOwner::new(Some(&owner.living_entity.entity)),
            hooked_entity_id: AtomicI32::new(0),
            in_ground: AtomicBool::new(false),
            has_hit: AtomicBool::new(false),
            wait_countdown: AtomicI32::new(wait_ticks),
            bite_countdown: AtomicI32::new(0),
        }
    }

    pub fn reel_in(&self, player: &Player, rod: &ItemStack) -> i32 {
        let world = self.entity.world.load();
        let hooked_id = self.hooked_entity_id.load(Ordering::Relaxed);
        let mut dmg = 0;

        if hooked_id != 0
            && let Some(hooked) = world.get_entity_by_id(hooked_id)
        {
            let player_pos = player.get_entity().pos.load();
            let bobber_pos = self.entity.pos.load();
            let delta = player_pos - bobber_pos;
            let motion =
                delta
                    .multiply(0.1, 0.1, 0.1)
                    .add_raw(0.0, delta.length().sqrt() * 0.08, 0.0);
            hooked.get_entity().add_velocity(motion);
            world.send_entity_status(
                &self.entity,
                EntityStatus::FishingRodReelIn,
                ActorEventID::FishhookHookTime,
            );
            dmg = if hooked.get_entity().entity_type == &EntityType::ITEM {
                3
            } else {
                5
            };
        } else if self.bite_countdown.load(Ordering::Relaxed) > 0 {
            let luck =
                crate::enchantment::helper::EnchantmentHelper::modify_fishing_luck_bonus(rod, 0.0)
                    + player.living_entity.get_attribute_value(&Attributes::LUCK) as f32;
            let bobber_pos = self.entity.pos.load();
            let player_pos = player.get_entity().pos.load();
            let params = LootContextParameters {
                position: Some(bobber_pos),
                tool: Some(rod.clone()),
                this_entity: Some(self),
                world: Some(&world),
                luck,
                ..Default::default()
            };
            let handle = world
                .server
                .upgrade()
                .and_then(|s| {
                    s.datapack_manager
                        .get_loot_table("minecraft:gameplay/fishing")
                })
                .unwrap_or(LootTableHandle::Static(
                    &pumpkin_data::loot_table::GAMEPLAY_FISHING,
                ));
            let items = generate_loot_from_handle(&handle, 0, &params);
            let delta = player_pos - bobber_pos;
            let vy = delta.y * 0.1 + delta.length().sqrt() * 0.08;

            for item_stack in items {
                if item_stack.is_empty() {
                    continue;
                }
                if item_stack
                    .item
                    .has_tag(&pumpkin_data::tag::Item::MINECRAFT_FISHES)
                {
                    player.increment_stat(
                        pumpkin_data::statistic::StatisticCategory::Custom,
                        pumpkin_data::statistic::CustomStatistic::FishCaught as i32,
                        1,
                    );
                }
                player.trigger_advancement(
                    crate::entity::player::advancement::trigger::AdvancementTrigger::FishedItem {
                        item_id: format!("minecraft:{}", item_stack.item.registry_key),
                    },
                );
                let item_entity = ItemEntity::new_with_velocity(
                    Entity::new(world.clone(), bobber_pos, &EntityType::ITEM),
                    item_stack,
                    Vector3::new(delta.x * 0.1, vy, delta.z * 0.1),
                    0,
                );
                world.spawn_entity(Arc::new(item_entity));
            }

            let xp_amount = rand::rng().random_range(1..=6);
            ExperienceOrbEntity::spawn(&world, player_pos.add_raw(0.0, 0.5, 0.5), xp_amount);
            dmg = 1;
        }

        if self.in_ground.load(Ordering::Relaxed) {
            dmg = dmg.max(2);
        }

        dmg
    }

    pub fn process_tick(&self, caller: &dyn EntityBase, server: &Server) {
        let entity = self.get_entity();
        let world = entity.world.load();
        self.owner.tick(entity);

        if self.in_ground.load(Ordering::Relaxed) {
            return;
        }

        let hooked_id = self.hooked_entity_id.load(Ordering::Relaxed);
        if hooked_id != 0 {
            if let Some(hooked) = world.get_entity_by_id(hooked_id) {
                if hooked.get_entity().removed.load(Ordering::Relaxed) {
                    self.hooked_entity_id.store(0, Ordering::Relaxed);
                } else {
                    let mut hooked_pos = hooked.get_entity().pos.load();
                    hooked_pos.y += hooked.get_entity().get_eye_height() * 0.8;
                    entity.set_pos(hooked_pos);
                    return;
                }
            } else {
                self.hooked_entity_id.store(0, Ordering::Relaxed);
            }
        }

        let mut velocity = entity.velocity.load();

        if entity.touching_water.load(Ordering::Relaxed) {
            velocity.y += 0.02; // Buoyancy

            let bite = self.bite_countdown.load(Ordering::Relaxed);
            if bite > 0 {
                self.bite_countdown.store(bite - 1, Ordering::Relaxed);
                if bite % 5 == 0 {
                    world.spawn_particle(
                        entity.pos.load(),
                        Vector3::new(0.1f32, 0.1f32, 0.1f32),
                        0.0,
                        5,
                        pumpkin_data::particle::Particle::Bubble,
                    );
                }
            } else {
                let wait = self.wait_countdown.load(Ordering::Relaxed);
                if wait > 0 {
                    self.wait_countdown.store(wait - 1, Ordering::Relaxed);
                } else {
                    // Start bite
                    self.bite_countdown.store(40, Ordering::Relaxed);
                    self.wait_countdown
                        .store(rand::random::<i32>().abs() % 600 + 100, Ordering::Relaxed);

                    world.play_sound(
                        Sound::EntityFishingBobberSplash,
                        SoundCategory::Neutral,
                        &entity.pos.load(),
                    );
                }
            }
        } else {
            velocity.y -= Self::GRAVITY;
        }

        let inertia = if entity.touching_water.load(Ordering::Relaxed) {
            Self::WATER_INERTIA
        } else {
            Self::AIR_INERTIA
        };
        velocity = velocity.multiply(inertia, inertia, inertia);
        entity.velocity.store(velocity);

        // Hits are checked along the path first, then the move handles block collision.
        let hit = get_hit_result_on_move_vector(
            caller,
            |other| {
                (other.can_be_hit_by_projectile() && self.owner.can_hit(&world, other.get_entity()))
                    || (other.get_entity().is_alive()
                        && other.get_entity().entity_type == &EntityType::ITEM)
            },
            ClipBlock::Collider,
        );
        if let Some(hit) = hit {
            hit_target_or_deflect_self(caller, hit, server);
        }
        if self.hooked_entity_id.load(Ordering::Relaxed) != 0 {
            return;
        }

        entity.move_entity(caller, entity.velocity.load());
        if entity.on_ground.load(Ordering::Relaxed)
            || entity.horizontal_collision.load(Ordering::Relaxed)
        {
            self.in_ground.store(true, Ordering::Relaxed);
            entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
        }
    }

    fn set_hooked_entity(&self, hooked: &Entity) {
        self.hooked_entity_id
            .store(hooked.entity_id, Ordering::Relaxed);
        self.entity.set_synced_data(
            pumpkin_data::tracked_data::fishing_bobber::HOOKED_ENTITY,
            hooked.entity_id + 1,
        );
    }
}

impl EntityBase for FishingBobberEntity {
    fn should_bounce_on_world_border(&self) -> bool {
        true
    }

    fn get_projectile_owner(&self) -> Option<&ProjectileOwner> {
        Some(&self.owner)
    }

    fn entity_bedrock_spawn_metadata(
        &self,
    ) -> Option<pumpkin_protocol::bedrock::client::set_actor_data::SyncedActorDataList> {
        let mut meta =
            pumpkin_protocol::bedrock::client::set_actor_data::SyncedActorDataList::new();
        meta.set(
            pumpkin_protocol::bedrock::client::set_actor_data::entity_data_key::OWNER,
            pumpkin_protocol::bedrock::client::set_actor_data::MetadataValue::Int64(i64::from(
                self.owner
                    .get(&self.entity.world.load())
                    .map_or(0, |owner| owner.get_entity().entity_id),
            )),
        );
        Some(meta)
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
    fn on_hit(&self, hit: ProjectileHit) {
        self.has_hit.store(true, Ordering::Relaxed);
        match hit {
            ProjectileHit::Entity { entity, .. } => self.set_hooked_entity(entity.get_entity()),
            ProjectileHit::Block { hit_pos, .. } => {
                let entity = &self.entity;
                let distance = (hit_pos - entity.pos.load()).length();
                let velocity = entity.velocity.load().normalize() * distance;
                entity.velocity.store(velocity);
            }
        }
    }

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        self.process_tick(caller, server);
    }
}
