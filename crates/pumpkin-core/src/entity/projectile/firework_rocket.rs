use crate::{
    entity::{Entity, EntityBase, projectile::ThrownItemEntity},
    server::Server,
    world::World,
};
use pumpkin_data::entity::EntityStatus;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_protocol::codec::optional_int::OptionalInt;
use pumpkin_util::{
    math::vector3::Vector3,
    random::{RandomGenerator, RandomImpl, get_seed, xoroshiro128::Xoroshiro},
};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

const GRAVITY: f64 = 0.0;

pub struct FireworkRocketEntity {
    pub entity: ThrownItemEntity,
    pub life: AtomicU32,
    pub life_time: AtomicU32,
    pub shot_at_angle: AtomicBool,
    pub attached_to_shooter: bool,
    pub explosions_count: u32,
}

impl FireworkRocketEntity {
    pub fn new(entity: Entity) -> Self {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));

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
            life_time: (10 + random.next_bounded_i32(6) as u32 + random.next_bounded_i32(7) as u32)
                .into(),
            shot_at_angle: AtomicBool::new(false),
            attached_to_shooter: false,
            explosions_count: 0,
        }
    }

    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));

        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        thrown.entity.set_velocity(Vector3::new(
            random.next_triangular(0.0, 0.002_297),
            0.05,
            random.next_triangular(0.0, 0.002_297),
        ));

        let rocket = Self {
            entity: thrown,
            life: 0.into(),
            life_time: (10 + random.next_bounded_i32(6) as u32 + random.next_bounded_i32(7) as u32)
                .into(),
            shot_at_angle: AtomicBool::new(false),
            attached_to_shooter: true,
            explosions_count: 0,
        };

        rocket.entity.entity.set_synced_data(
            pumpkin_data::tracked_data::firework_rocket::ATTACHED_TO_TARGET,
            OptionalInt(Some(shooter.entity_id)),
        );

        rocket
    }

    pub fn new_shot_projectile(
        entity: Entity,
        shooter: Option<&Entity>,
        rocket_stack: &pumpkin_data::item_stack::ItemStack,
        shot_at_angle: bool,
    ) -> Self {
        let mut flight_count = 1;
        let mut explosions_count = 0;
        if let Some(fireworks) =
            rocket_stack.get_data_component::<pumpkin_data::data_component_impl::FireworksImpl>()
        {
            flight_count += fireworks.flight_duration.max(0);
            explosions_count = fireworks.explosions.len() as u32;
        }

        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
        let life_time = (10 * flight_count
            + random.next_bounded_i32(6) as i32
            + random.next_bounded_i32(7) as i32) as u32;

        entity.set_synced_data(
            pumpkin_data::tracked_data::firework_rocket::SHOT_AT_ANGLE,
            shot_at_angle,
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::firework_rocket::ID_FIREWORKS_ITEM,
            pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer::from(
                rocket_stack.clone(),
            ),
        );

        let owner_id = shooter.map(|s| s.entity_id);
        let thrown = ThrownItemEntity {
            entity,
            owner_id,
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
            gravity: GRAVITY,
        };

        Self {
            entity: thrown,
            life: 0.into(),
            life_time: life_time.into(),
            shot_at_angle: AtomicBool::new(shot_at_angle),
            attached_to_shooter: false,
            explosions_count,
        }
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

        // Vanilla explosion damage: 5.0 + explosions.size() * 2.0
        if self.explosions_count > 0 {
            let damage_amount = 5.0 + (self.explosions_count as f32) * 2.0;
            let pos = entity.pos.load();
            let attacker_opt = self.entity.owner_id.and_then(|id| world.get_entity_by_id(id));
            let attacker_ref = attacker_opt.as_ref().map(|a| a.as_ref() as &dyn EntityBase);

            for living in world.get_nearby_entities(pos, 5.0).into_values() {
                let target_pos = living.get_entity().pos.load();
                let dist = target_pos.squared_distance_to(pos.x, pos.y, pos.z).sqrt();
                if dist <= 5.0 {
                    let damage = damage_amount * ((5.0 - dist) as f32 / 5.0).max(0.0).sqrt();
                    living.damage_with_context(
                        living.as_ref(),
                        damage,
                        pumpkin_data::damage::DamageType::FIREWORKS,
                        Some(pos),
                        attacker_ref,
                        Some(self),
                    );
                }
            }

            for player in world.get_nearby_players(pos, 5.0) {
                let target_pos = player.get_entity().pos.load();
                let dist = target_pos.squared_distance_to(pos.x, pos.y, pos.z).sqrt();
                if dist <= 5.0 {
                    let damage = damage_amount * ((5.0 - dist) as f32 / 5.0).max(0.0).sqrt();
                    player.damage_with_context(
                        player.as_ref(),
                        damage,
                        pumpkin_data::damage::DamageType::FIREWORKS,
                        Some(pos),
                        attacker_ref,
                        Some(self),
                    );
                }
            }
        }

        entity.remove();
    }
}

impl EntityBase for FireworkRocketEntity {
    fn get_owner_id(&self) -> Option<i32> {
        self.entity.owner_id
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        self.entity.process_tick(caller);

        let entity = self.get_entity();
        let world = entity.world.load();
        let mut velocity = entity.velocity.load();

        if self.attached_to_shooter {
            if let Some(shooter_id) = self.entity.owner_id {
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
            }
        } else if !self.shot_at_angle.load(Ordering::Relaxed) {
            velocity.x *= 1.15;
            velocity.z *= 1.15;
            velocity.y += 0.04;
            entity.set_velocity(velocity);
        }

        let current_life = self.life.fetch_add(1, Ordering::Relaxed);
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
