use crossbeam::atomic::AtomicCell;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use crate::world::game_event::GameEventContext;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::tracked_data;
use pumpkin_protocol::codec::var_int::VarInt;

pub struct VehicleEntity {
    pub entity: Entity,
    pub hurt_time: AtomicI32,
    pub hurt_dir: AtomicI32,
    pub damage: AtomicCell<f32>,
}

impl VehicleEntity {
    pub const fn new(entity: Entity) -> Self {
        Self {
            entity,
            hurt_time: AtomicI32::new(0),
            hurt_dir: AtomicI32::new(1),
            damage: AtomicCell::new(0.0),
        }
    }

    /// Vanilla `AbstractMinecart`/`AbstractBoat.tick` start: wobble fades, plugins see the tick.
    pub fn tick(&self) {
        let hurt_time = self.get_hurt_time();
        if hurt_time > 0 {
            self.set_hurt_time(hurt_time - 1);
        }
        let damage = self.get_damage();
        if damage > 0.0 {
            self.set_damage(damage - 1.0);
        }

        let mut update_event =
            crate::plugin::api::events::vehicle::vehicle_update::VehicleUpdateEvent::new(
                self.entity.entity_id,
            );
        if let Some(server) = self.entity.world.load().server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut update_event);
        }
    }

    pub async fn create(&self) {
        let mut create_event =
            crate::plugin::api::events::vehicle::vehicle_create::VehicleCreateEvent::new(
                self.entity.entity_id,
            );
        if let Some(server) = self.entity.world.load().server.upgrade() {
            server.plugin_manager.fire(&server, &mut create_event).await;
        }
    }

    pub async fn move_vehicle(
        &self,
        from: pumpkin_util::math::vector3::Vector3<f64>,
        to: pumpkin_util::math::vector3::Vector3<f64>,
    ) {
        let mut move_event =
            crate::plugin::api::events::vehicle::vehicle_move::VehicleMoveEvent::new(
                self.entity.entity_id,
                from,
                to,
            );
        if let Some(server) = self.entity.world.load().server.upgrade() {
            server.plugin_manager.fire(&server, &mut move_event).await;
        }
    }

    pub async fn collide_entity(&self, collided_entity_id: i32) {
        let mut base_event =
            crate::plugin::api::events::vehicle::vehicle_collision::VehicleCollisionEvent::new(
                self.entity.entity_id,
            );
        let mut collide_event = crate::plugin::api::events::vehicle::vehicle_entity_collision::VehicleEntityCollisionEvent::new(
            self.entity.entity_id,
            collided_entity_id,
        );
        if let Some(server) = self.entity.world.load().server.upgrade() {
            server.plugin_manager.fire(&server, &mut base_event).await;
            server
                .plugin_manager
                .fire(&server, &mut collide_event)
                .await;
        }
    }

    pub async fn collide_block(&self, block_pos: pumpkin_util::math::position::BlockPos) {
        let mut base_event =
            crate::plugin::api::events::vehicle::vehicle_collision::VehicleCollisionEvent::new(
                self.entity.entity_id,
            );
        let mut collide_event = crate::plugin::api::events::vehicle::vehicle_block_collision::VehicleBlockCollisionEvent::new(
            self.entity.entity_id,
            block_pos,
        );
        if let Some(server) = self.entity.world.load().server.upgrade() {
            server.plugin_manager.fire(&server, &mut base_event).await;
            server
                .plugin_manager
                .fire(&server, &mut collide_event)
                .await;
        }
    }

    pub fn set_damage(&self, damage: f32) {
        self.damage.store(damage);
        self.entity
            .set_synced_data(tracked_data::boat::ID_DAMAGE, damage);
    }

    pub fn get_damage(&self) -> f32 {
        self.damage.load()
    }

    pub fn set_hurt_time(&self, hurt_time: i32) {
        self.hurt_time.store(hurt_time, Ordering::Relaxed);
        self.entity
            .set_synced_data(tracked_data::boat::ID_HURT, VarInt(hurt_time));
    }

    pub fn get_hurt_time(&self) -> i32 {
        self.hurt_time.load(Ordering::Relaxed)
    }

    pub fn set_hurt_dir(&self, hurt_dir: i32) {
        self.hurt_dir.store(hurt_dir, Ordering::Relaxed);
        self.entity
            .set_synced_data(tracked_data::boat::ID_HURTDIR, VarInt(hurt_dir));
    }

    pub fn get_hurt_dir(&self) -> i32 {
        self.hurt_dir.load(Ordering::Relaxed)
    }

    /// Vanilla `hurtServer`/`activateMinecart` wobble: flips the side, restarts the shake.
    pub fn wobble(&self, damage: f32) {
        self.set_hurt_dir(-self.get_hurt_dir());
        self.set_hurt_time(10);
        self.set_damage(damage);
        self.entity.mark_hurt();
    }

    /// Vanilla `Entity.kill`: removes the vehicle and lets listeners know it died.
    pub fn kill(&self) {
        self.entity.remove();
        self.entity.world.load().emit_game_event_with_context(
            GameEvent::EntityDie.name(),
            self.entity.pos.load(),
            &GameEventContext {
                source_entity: Some(&self.entity),
            },
        );
    }

    pub fn kill_and_drop_self(&self) {
        self.kill();
        let world = self.entity.world.load();
        let entity_drops = world.level_info.load().game_rules.entity_drops;

        if entity_drops {
            let resource_name = self.entity.entity_type.resource_name;
            let key = format!("minecraft:entities/{resource_name}");
            if let Some(loot_table) = pumpkin_data::loot_table::get_loot_table(&key) {
                let pos = self.entity.block_pos.load();
                let seed: i64 = rand::random();
                for stack in crate::world::loot::generate_loot_with_context(
                    loot_table,
                    seed,
                    &crate::world::loot::LootContextParameters {
                        world: Some(&world),
                        ..Default::default()
                    },
                ) {
                    world.drop_stack(&pos, stack);
                }
            }
        }
    }

    pub fn damage_with_context(&self, amount: f32, source: Option<&dyn EntityBase>) -> bool {
        if !self.entity.is_alive() {
            return true;
        }
        if !self.damage_event_allows(amount, source) {
            return false;
        }
        // TODO: vanilla also checks `isInvulnerableToBase` and reads the creative attacker from
        // `source.getEntity()`; boats don't pass the damage type or the cause yet.
        let cause = source;
        let destroys = self.hurt(amount, cause, false);
        if destroys || creative_attacker(cause) {
            if !self.destroy_event_allows(source) {
                return false;
            }
            if destroys {
                self.kill_and_drop_self();
            } else {
                self.entity.remove();
            }
        }
        true
    }

    /// Fires `VehicleDamageEvent`, `false` when a plugin cancelled the hit.
    pub fn damage_event_allows(&self, amount: f32, source: Option<&dyn EntityBase>) -> bool {
        let attacker_id = source.map(|s| s.get_entity().entity_id);
        let mut damage_event =
            crate::plugin::api::events::vehicle::vehicle_damage::VehicleDamageEvent::new(
                self.entity.entity_id,
                amount,
                attacker_id,
            );
        if let Some(server) = self.entity.world.load().server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut damage_event);
        }
        !damage_event.cancelled
    }

    /// Fires `VehicleDestroyEvent`, `false` when a plugin kept the vehicle.
    pub fn destroy_event_allows(&self, source: Option<&dyn EntityBase>) -> bool {
        let attacker_id = source.map(|s| s.get_entity().entity_id);
        let mut destroy_event =
            crate::plugin::api::events::vehicle::vehicle_destroy::VehicleDestroyEvent::new(
                self.entity.entity_id,
                attacker_id,
            );
        if let Some(server) = self.entity.world.load().server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut destroy_event);
        }
        !destroy_event.cancelled
    }

    /// Vanilla `VehicleEntity.hurtServer` after the invulnerability check: the wobble, then
    /// `true` when the hit destroys the vehicle. A creative attacker that doesn't destroy it
    /// discards it instead.
    // TODO: vanilla `ignoreExplosion` spares vehicles from mob explosions without
    // `mobGriefing`; `is_immune_to_explosion` doesn't get the explosion yet.
    pub fn hurt(
        &self,
        amount: f32,
        cause: Option<&dyn EntityBase>,
        should_source_destroy: bool,
    ) -> bool {
        let damage = self.apply_damage_wobble(amount, cause);
        should_source_destroy || (!creative_attacker(cause) && damage > 40.0)
    }

    /// Vanilla `VehicleEntity.hurtServer` wobble without destroying the vehicle.
    fn apply_damage_wobble(&self, amount: f32, cause: Option<&dyn EntityBase>) -> f32 {
        let new_strength = self.get_damage() + amount * 10.0;
        self.wobble(new_strength);
        self.entity.world.load().emit_game_event_with_context(
            GameEvent::EntityDamage.name(),
            self.entity.pos.load(),
            &GameEventContext {
                source_entity: cause,
            },
        );
        new_strength
    }
}

/// Vanilla `source.getEntity() instanceof Player player && player.getAbilities().instabuild`.
pub fn creative_attacker(cause: Option<&dyn EntityBase>) -> bool {
    cause
        .and_then(EntityBase::get_player)
        .is_some_and(Player::is_creative)
}
