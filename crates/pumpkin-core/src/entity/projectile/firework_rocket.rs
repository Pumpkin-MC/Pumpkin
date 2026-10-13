use crate::{
    entity::{Entity, EntityBase, projectile::ThrownItemEntity},
    server::Server,
    world::World,
};
use pumpkin_data::data_component_impl::FireworksImpl;
use pumpkin_data::entity::EntityStatus;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::codec::optional_int::OptionalInt;
use pumpkin_util::{
    math::vector3::Vector3,
    random::{RandomGenerator, RandomImpl, get_seed, xoroshiro128::Xoroshiro},
};
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

const GRAVITY: f64 = 0.0;

pub struct FireworkRocketEntity {
    entity: ThrownItemEntity,
    life: AtomicU32,
    life_time: AtomicU32,
    // The client renders the rocket's model and particle trail from this - without it the entity
    // exists (and, when boosting, moves the player correctly) but is completely invisible.
    item_stack: RwLock<ItemStack>,
}

/// Vanilla adds ten ticks of lifetime per level of the stack's `Fireworks` flight-duration
/// component, before the random terms. A stack without the component (shouldn't normally
/// happen - every firework rocket item carries one) falls back to duration 1.
fn life_time_for(item_stack: &ItemStack, random: &mut RandomGenerator) -> u32 {
    let flight_duration = item_stack
        .get_data_component::<FireworksImpl>()
        .map_or(1, |fireworks| fireworks.flight_duration.max(1) as u32);

    10 * flight_duration + random.next_bounded_i32(6) as u32 + random.next_bounded_i32(7) as u32
}

impl FireworkRocketEntity {
    pub fn new(entity: Entity) -> Self {
        Self::new_with_item(entity, ItemStack::new(1, &Item::FIREWORK_ROCKET))
    }

    /// Same as [`Self::new`], but rendering the specific item stack the rocket was created from
    /// (so a rocket with custom fireworks data shows the right explosion trail) and without an
    /// owning shooter entity, for a rocket that wasn't thrown by anyone (e.g. dispensed).
    pub fn new_with_item(entity: Entity, item_stack: ItemStack) -> Self {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));

        entity.set_velocity(Vector3::new(
            random.next_triangular(0.0, 0.002_297),
            0.05,
            random.next_triangular(0.0, 0.002_297),
        ));
        let life_time = life_time_for(&item_stack, &mut random);
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
            item_stack: RwLock::new(item_stack),
        }
    }

    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        Self::new_shot_with_item(entity, shooter, ItemStack::new(1, &Item::FIREWORK_ROCKET))
    }

    /// Same as [`Self::new_shot`], but rendering the specific item stack that was used to fire
    /// it (so a rocket with custom fireworks data shows the right explosion trail).
    pub fn new_shot_with_item(entity: Entity, shooter: &Entity, item_stack: ItemStack) -> Self {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));

        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        thrown.entity.set_velocity(Vector3::new(
            random.next_triangular(0.0, 0.002_297),
            0.05,
            random.next_triangular(0.0, 0.002_297),
        ));
        let life_time = life_time_for(&item_stack, &mut random);

        let rocket = Self {
            entity: thrown,
            life: 0.into(),
            life_time: life_time.into(),
            item_stack: RwLock::new(item_stack),
        };

        rocket.entity.entity.set_synced_data(
            pumpkin_data::tracked_data::firework_rocket::ATTACHED_TO_TARGET,
            OptionalInt(Some(shooter.entity_id)),
        );

        rocket
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

        entity.remove();
    }
}

impl EntityBase for FireworkRocketEntity {
    fn init_data_tracker(&self) {
        let stack = self
            .item_stack
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::firework_rocket::ID_FIREWORKS_ITEM,
            ItemStackSerializer::from(stack.clone()),
        );
    }

    fn get_owner_id(&self) -> Option<i32> {
        self.entity.owner_id
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        self.entity.process_tick(caller);

        let entity = self.get_entity();
        let world = entity.world.load();
        let mut velocity = entity.velocity.load();

        // The rocket is only attached to its shooter (following them and boosting their
        // flight) while that shooter is actually gliding. A rocket that was block-launched,
        // dispensed, or hand-launched without gliding has either no owner at all or an owner
        // that isn't fall-flying - either way it's a free-flying rocket, same physics as the
        // ownerless case below.
        let mut attached_boost = false;
        if let Some(shooter_id) = self.entity.owner_id
            && let Some(shooter) = world.get_entity_by_id(shooter_id)
        {
            let shooter = shooter.get_entity();

            if shooter.is_fall_flying() {
                attached_boost = true;
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

        if !attached_boost {
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

#[cfg(test)]
mod test {
    use super::life_time_for;
    use pumpkin_data::data_component_impl::FireworksImpl;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_util::random::{RandomGenerator, get_seed, xoroshiro128::Xoroshiro};

    fn random() -> RandomGenerator {
        RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()))
    }

    /// Without a `Fireworks` component, lifetime falls back to duration 1: the same
    /// 10 + [0,6) + [0,7) range the rocket used before flight duration was accounted for.
    #[test]
    fn life_time_without_fireworks_component_uses_duration_one() {
        let stack = ItemStack::new(1, &Item::FIREWORK_ROCKET);
        let life_time = life_time_for(&stack, &mut random());
        assert!((10..=21).contains(&life_time));
    }

    /// Vanilla adds ten ticks per flight-duration level before the random terms, so a
    /// duration-3 rocket's lifetime always exceeds a duration-1 rocket's maximum.
    #[test]
    fn life_time_scales_with_flight_duration() {
        let mut stack = ItemStack::new(1, &Item::FIREWORK_ROCKET);
        stack.set_data_component(FireworksImpl::new(3, Vec::new()));

        let life_time = life_time_for(&stack, &mut random());
        assert!((30..=41).contains(&life_time));
    }
}
