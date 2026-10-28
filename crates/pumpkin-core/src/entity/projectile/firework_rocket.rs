use crate::{
    entity::{Entity, EntityBase, projectile::ThrownItemEntity},
    server::Server,
    world::World,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::FireworksImpl;
use pumpkin_data::entity::EntityStatus;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::codec::optional_int::OptionalInt;
use pumpkin_util::{
    math::vector3::Vector3,
    random::{RandomGenerator, RandomImpl, get_seed, xoroshiro128::Xoroshiro},
};
use std::sync::{
    RwLock,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

const GRAVITY: f64 = 0.0;

pub struct FireworkRocketEntity {
    pub entity: ThrownItemEntity,
    life: AtomicU32,
    life_time: AtomicU32,
    item_stack: RwLock<ItemStack>,
    attached_to_entity: Option<i32>,
    shot_at_angle: AtomicBool,
}

impl FireworkRocketEntity {
    fn compute_lifetime(stack: &ItemStack, random: &mut RandomGenerator) -> u32 {
        let flight_count = 1 + stack
            .get_data_component::<FireworksImpl>()
            .map_or(0, |fw| fw.flight_duration.max(0) as u32);
        10 * flight_count + random.next_bounded_i32(6) as u32 + random.next_bounded_i32(7) as u32
    }

    pub fn new(entity: Entity) -> Self {
        Self::new_with_stack(entity, ItemStack::new(1, &Item::FIREWORK_ROCKET))
    }

    pub fn new_with_stack(entity: Entity, stack: ItemStack) -> Self {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
        let life_time = Self::compute_lifetime(&stack, &mut random);

        entity.set_velocity(Vector3::new(
            random.next_triangular(0.0, 0.002_297),
            0.05,
            random.next_triangular(0.0, 0.002_297),
        ));
        Self {
            entity: ThrownItemEntity {
                entity,
                owner_id: None,
                collides_with_projectiles: false,
                has_hit: AtomicBool::new(false),
                gravity: GRAVITY,
            },
            life: 0.into(),
            life_time: life_time.into(),
            item_stack: RwLock::new(stack),
            attached_to_entity: None,
            shot_at_angle: AtomicBool::new(false),
        }
    }

    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        Self::new_shot_with_stack(entity, shooter, ItemStack::new(1, &Item::FIREWORK_ROCKET))
    }

    pub fn new_shot_with_stack(entity: Entity, shooter: &Entity, stack: ItemStack) -> Self {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
        let life_time = Self::compute_lifetime(&stack, &mut random);

        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        thrown.entity.set_velocity(Vector3::new(
            random.next_triangular(0.0, 0.002_297),
            0.05,
            random.next_triangular(0.0, 0.002_297),
        ));

        Self {
            entity: thrown,
            life: 0.into(),
            life_time: life_time.into(),
            item_stack: RwLock::new(stack),
            attached_to_entity: Some(shooter.entity_id),
            shot_at_angle: AtomicBool::new(false),
        }
    }

    pub fn new_shot_at_angle(entity: Entity, shooter: &Entity, stack: ItemStack) -> Self {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
        let life_time = Self::compute_lifetime(&stack, &mut random);

        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        Self {
            entity: thrown,
            life: 0.into(),
            life_time: life_time.into(),
            item_stack: RwLock::new(stack),
            attached_to_entity: None,
            shot_at_angle: AtomicBool::new(true),
        }
    }

    pub fn set_shot_at_angle(&self, shot_at_angle: bool) {
        self.shot_at_angle.store(shot_at_angle, Ordering::Relaxed);
    }

    pub fn explode_and_remove(&self, world: &World) {
        let entity = self.get_entity();
        if let Some(server) = world.server.upgrade() {
            let mut event =
                crate::plugin::api::events::entity::firework_explode::FireworkExplodeEvent {
                    entity_id: entity.entity_id,
                    cancelled: false,
                };
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return;
            }
        }
        world.send_entity_status(
            entity,
            EntityStatus::FireworksExplode,
            Some(ActorEventID::FireworksExplode),
        );
        self.deal_explosion_damage(world);

        entity.remove();
    }

    fn deal_explosion_damage(&self, world: &World) {
        let explosion_count = {
            let stack = self
                .item_stack
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            stack
                .get_data_component::<FireworksImpl>()
                .map_or(0, |fw| fw.explosions.len())
        };
        if explosion_count == 0 {
            return;
        }

        let damage_amount = 5.0 + (explosion_count as f32) * 2.0;
        let entity = self.get_entity();
        let owner = self
            .entity
            .owner_id
            .and_then(|id| world.get_entity_by_id(id));

        if let Some(attached_id) = self.attached_to_entity
            && let Some(attached) = world.get_entity_by_id(attached_id)
        {
            let _ = attached.damage_with_context(
                attached.as_ref(),
                damage_amount,
                DamageType::FIREWORKS,
                Some(entity.pos.load()),
                Some(self),
                owner.as_deref(),
            );
        }

        let rocket_pos = entity.pos.load();
        let search_box = entity.bounding_box.load().expand(5.0, 5.0, 5.0);
        let mut targets = world.get_entities_at_box(&search_box);
        for player in world.get_players_at_box(&search_box) {
            targets.push(player as std::sync::Arc<dyn EntityBase>);
        }

        for target in targets {
            let target_entity = target.get_entity();
            if target_entity.entity_id == entity.entity_id
                || Some(target_entity.entity_id) == self.attached_to_entity
                || target.get_living_entity().is_none()
            {
                continue;
            }
            let target_pos = target_entity.pos.load();
            let dist_sq = rocket_pos.squared_distance_to_vec(&target_pos);
            if dist_sq > 25.0 {
                continue;
            }
            let dist = dist_sq.sqrt();
            let damage = damage_amount * ((5.0 - dist) / 5.0).max(0.0).sqrt() as f32;
            if damage > 0.0 {
                let _ = target.damage_with_context(
                    target.as_ref(),
                    damage,
                    DamageType::FIREWORKS,
                    Some(rocket_pos),
                    Some(self),
                    owner.as_deref(),
                );
            }
        }
    }
}

impl EntityBase for FireworkRocketEntity {
    fn get_owner_id(&self) -> Option<i32> {
        self.entity.owner_id
    }

    fn init_data_tracker(&self) {
        let entity = self.get_entity();
        let stack = self
            .item_stack
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        entity.set_synced_data(
            pumpkin_data::tracked_data::firework_rocket::ID_FIREWORKS_ITEM,
            ItemStackSerializer::from(stack),
        );
        if let Some(attached_id) = self.attached_to_entity {
            entity.set_synced_data(
                pumpkin_data::tracked_data::firework_rocket::ATTACHED_TO_TARGET,
                OptionalInt(Some(attached_id)),
            );
        }
        if self.shot_at_angle.load(Ordering::Relaxed) {
            entity.set_synced_data(
                pumpkin_data::tracked_data::firework_rocket::SHOT_AT_ANGLE,
                true,
            );
        }
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        self.entity.process_tick(caller);

        let entity = self.get_entity();
        let world = entity.world.load();
        let mut velocity = entity.velocity.load();

        if let Some(shooter_id) = self.attached_to_entity {
            if let Some(shooter) = world.get_entity_by_id(shooter_id) {
                let shooter = shooter.get_entity();

                if shooter.is_fall_flying() {
                    let mut boost_cancelled = false;
                    if let Some(player) = world.get_player_by_id(shooter_id)
                        && let Some(server) = world.server.upgrade()
                    {
                        let mut event = crate::plugin::api::events::player::player_elytra_boost::PlayerElytraBoostEvent {
                            player,
                            firework_id: entity.entity_id,
                            cancelled: false,
                        };
                        server.plugin_manager.fire_blocking(&server, &mut event);
                        if event.cancelled {
                            boost_cancelled = true;
                        }
                    }
                    if !boost_cancelled {
                        let rotation = shooter.rotation().to_f64();
                        let shooter_vel = shooter.velocity.load();

                        let new_shooter_vel =
                            shooter_vel + (rotation * 0.1 + (rotation * 1.5 - shooter_vel) * 0.5);

                        shooter.set_velocity(new_shooter_vel);

                        entity.set_pos(shooter.pos.load());
                        entity.set_velocity(new_shooter_vel);
                    }
                }
            }
        } else if !self.shot_at_angle.load(Ordering::Relaxed) {
            velocity.x *= 1.15;
            velocity.z *= 1.15;
            velocity.y += 0.04;
            entity.set_velocity(velocity);
        }

        let current_life = self.life.fetch_add(1, Ordering::Relaxed);
        if current_life == 0 && !entity.silent.load(Ordering::Relaxed) {
            world.play_sound_fine(
                Sound::EntityFireworkRocketLaunch,
                SoundCategory::Ambient,
                &entity.pos.load(),
                3.0,
                1.0,
            );
        }
        if current_life > self.life_time.load(Ordering::Relaxed) {
            self.explode_and_remove(&world);
        }
    }

    fn get_entity(&self) -> &crate::entity::Entity {
        &self.entity.entity
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, _hit: crate::entity::projectile::ProjectileHit) {
        let world = self.get_entity().world.load();
        self.explode_and_remove(&world);
    }
}
