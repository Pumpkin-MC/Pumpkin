use super::{Mob, PathAwareEntity};
use crate::entity::EntityBaseFuture;
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, NBTStorage, living::LivingEntity};
use crate::server::Server;
use pumpkin_data::damage::DamageType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_protocol::java::client::play::{CHeadRot, CUpdateEntityRot};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

impl<T: Mob + Send + 'static> EntityBase for T {
    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.mob_init_data_tracker().await;
            let world = self.get_mob_entity().living_entity.entity.world.load();
            crate::entity::mob::equipment::equip_mob_on_spawn(self as &dyn EntityBase, &world)
                .await;

            // Vanilla Zombie.finalizeSpawn rolls door breaking from regional
            // difficulty; a value restored from NBT wins.
            let mob_entity = self.get_mob_entity();
            if self.supports_break_door_goal() && !mob_entity.can_break_doors_loaded() {
                let difficulty = crate::entity::mob::equipment::RegionalDifficulty::at(
                    &world,
                    self.get_entity().pos.load(),
                );
                mob_entity.set_can_break_doors(
                    rand::random::<f32>() < difficulty.special_multiplier * 0.1,
                );
            }

            let entity_name = self.get_entity().entity_type.resource_name;
            if let Some(def) = crate::entity::mob::equipment::EQUIPMENT_REGISTRY.get(entity_name)
                && def.can_pick_up_loot
            {
                let difficulty = crate::entity::mob::equipment::RegionalDifficulty::at(
                    &world,
                    self.get_entity().pos.load(),
                );
                let pickup_chance = 0.55 * difficulty.special_multiplier;
                self.get_mob_entity()
                    .set_can_pick_up_loot(rand::random::<f32>() < pickup_chance);
            }
        })
    }

    fn set_variant_name(&self, name: &str) {
        self.mob_set_variant_name(name);
    }

    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let mob_entity = self.get_mob_entity();
            mob_entity.living_entity.entity.tick_leash().await;
            mob_entity.tick_sun_burn().await;

            // Despawn far mobs so natural spawn caps free up (vanilla checkDespawn).
            if !self.requires_custom_persistence() && mob_entity.check_despawn().await {
                return;
            }

            if mob_entity.breeding_cooldown.load(Relaxed) > 0 {
                mob_entity.breeding_cooldown.fetch_sub(1, Relaxed);
            }

            if mob_entity.love_ticks.load(Relaxed) > 0 {
                let ticks = mob_entity.love_ticks.fetch_sub(1, Relaxed);
                if ticks % 10 == 0 {
                    let entity = &mob_entity.living_entity.entity;
                    let pos = entity.pos.load();
                    let world = entity.world.load();
                    world.spawn_particle(
                        pos + Vector3::new(0.0, f64::from(entity.height()) + 0.5, 0.0),
                        Vector3::new(0.5, 0.5, 0.5),
                        1.0,
                        1,
                        pumpkin_data::particle::Particle::Heart,
                    );
                }
            }

            self.mob_tick(caller).await;

            if mob_entity.living_entity.entity.is_removed() {
                return;
            }

            let age = mob_entity.living_entity.entity.age.load(Relaxed);
            let entity_id = mob_entity.living_entity.entity.entity_id;

            // 1. "Take" selectors out of the mutexes
            let mut target_selector = {
                let mut guard = mob_entity.target_selector.lock().unwrap();
                std::mem::take(&mut *guard)
            };
            let mut goals_selector = {
                let mut guard = mob_entity.goals_selector.lock().unwrap();
                std::mem::take(&mut *guard)
            };

            // 2. Perform AI logic (No locks held, so .await is safe!)
            if (age + entity_id) % 2 != 0 && age > 1 {
                target_selector.tick_goals(self, false).await;
                goals_selector.tick_goals(self, false).await;
            } else {
                target_selector.tick(self).await;
                goals_selector.tick(self).await;
            }

            // 3. "Put back" selectors
            {
                *mob_entity.target_selector.lock().unwrap() = target_selector;
                *mob_entity.goals_selector.lock().unwrap() = goals_selector;
            };

            // 4. Repeat for Navigator
            let mut navigator = {
                let mut guard = mob_entity.navigator.lock().unwrap();
                std::mem::take(&mut *guard)
            };

            navigator
                .tick(&mob_entity.living_entity, &mob_entity.move_control)
                .await;

            {
                *mob_entity.navigator.lock().unwrap() = navigator;
            };

            // Controllers are synchronous, so we can just use normal blocks
            {
                let mut look_control = mob_entity.look_control.lock().unwrap();
                look_control.tick(self);
            };

            {
                let mut move_control = mob_entity.move_control.lock().unwrap();
                move_control.tick(self);
            };

            mob_entity.living_entity.tick(caller, server).await;
            self.post_tick().await;

            // --- Packet logic remains the same ---
            let entity = &mob_entity.living_entity.entity;
            let yaw = (entity.yaw.load() * 256.0 / 360.0).rem_euclid(256.0) as u8;
            let pitch = (entity.pitch.load() * 256.0 / 360.0).rem_euclid(256.0) as u8;
            let head_yaw = (entity.head_yaw.load() * 256.0 / 360.0).rem_euclid(256.0) as u8;

            let last_yaw = mob_entity.last_sent_yaw.load(Relaxed);
            let last_pitch = mob_entity.last_sent_pitch.load(Relaxed);
            let last_head_yaw = mob_entity.last_sent_head_yaw.load(Relaxed);

            let chunk_pos = entity.chunk_pos.load();
            if yaw.abs_diff(last_yaw) >= 1 || pitch.abs_diff(last_pitch) >= 1 {
                let world = entity.world.load();
                world.broadcast_to_chunk(
                    chunk_pos,
                    &CUpdateEntityRot::new(
                        entity.entity_id.into(),
                        yaw,
                        pitch,
                        entity.on_ground.load(Relaxed),
                    ),
                );
                mob_entity.last_sent_yaw.store(yaw, Relaxed);
                mob_entity.last_sent_pitch.store(pitch, Relaxed);
            }

            if head_yaw.abs_diff(last_head_yaw) >= 1 {
                let world = entity.world.load();

                world.broadcast_to_chunk(
                    chunk_pos,
                    &CHeadRot::new(entity.entity_id.into(), head_yaw),
                );
                mob_entity.last_sent_head_yaw.store(head_yaw, Relaxed);
            }
        })
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn can_hit(&self) -> bool {
        true
    }

    fn damage_with_context<'a>(
        &'a self,
        caller: &'a dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            // pre_damage hook: allows mobs to dodge/cancel damage (e.g. enderman projectile dodge)
            if !self.pre_damage(damage_type, source).await {
                return false;
            }
            // Mob-specific damage modifier (e.g. shulker armor when closed).
            let amount = self.modify_incoming_damage(amount, damage_type);
            let damaged = self
                .get_mob_entity()
                .living_entity
                .damage_with_context(caller, amount, damage_type, position, source, cause)
                .await;
            if damaged {
                self.on_damage(damage_type, source).await;
            }
            damaged
        })
    }

    fn interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move { self.mob_interact(player, item_stack).await })
    }

    fn on_player_collision<'a>(&'a self, player: &'a Arc<Player>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move { self.mob_player_collision(player).await })
    }

    fn get_entity(&self) -> &Entity {
        &self.get_mob_entity().living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.get_mob_entity().living_entity)
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn is_in_love(&self) -> bool {
        self.get_mob_entity().is_in_love()
    }

    fn is_breeding_ready(&self) -> bool {
        self.get_mob_entity().is_breeding_ready()
    }

    fn reset_love(&self) {
        self.get_mob_entity().reset_love_ticks();
    }

    fn set_breeding_cooldown(&self, ticks: i32) {
        self.get_mob_entity()
            .breeding_cooldown
            .store(ticks, Relaxed);
    }

    fn is_panicking(&self) -> bool {
        self.get_path_aware_entity()
            .is_some_and(PathAwareEntity::is_panicking)
    }

    fn get_job_site_pos(&self) -> Option<pumpkin_util::math::position::BlockPos> {
        <T as Mob>::get_job_site(self)
    }

    fn get_home_pos(&self) -> Option<pumpkin_util::math::position::BlockPos> {
        <T as Mob>::get_home(self)
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn get_gravity(&self) -> f64 {
        self.get_mob_gravity()
    }

    fn get_y_velocity_drag(&self) -> Option<f64> {
        self.get_mob_y_velocity_drag()
    }

    fn get_experience_reward(&self, _killer: Option<&dyn EntityBase>) -> u32 {
        if self
            .get_entity()
            .age
            .load(std::sync::atomic::Ordering::Relaxed)
            < 0
        {
            return 0;
        }
        // TODO: apply enchantment processing like in vanilla
        Mob::get_base_experience_reward(self)
    }

    fn get_base_experience_reward(&self) -> u32 {
        Mob::get_base_experience_reward(self)
    }
}
