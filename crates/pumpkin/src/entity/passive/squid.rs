use std::f32::consts::{PI, TAU};
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityStatus;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ai::util::goal_utils,
    living::LivingEntity,
    mob::{Mob, MobEntity},
};

/// Represents a Squid, a passive aquatic mob that swims by pulsing its tentacles.
///
/// Wiki: <https://minecraft.wiki/w/Squid>
pub struct SquidEntity {
    pub mob_entity: MobEntity,
    pub movement: SquidMovement,
}

impl SquidEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        Arc::new(Self {
            mob_entity: MobEntity::new(entity),
            movement: SquidMovement::new(),
        })
    }
}

impl Mob for SquidEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.movement.tick(self);
    }

    fn custom_travel(&self, caller: &dyn EntityBase) -> bool {
        SquidMovement::travel(self, caller)
    }

    fn mob_is_pushed_by_fluids(&self) -> bool {
        false
    }
}

/// How vanilla squid get about, which is nothing like other mobs: no navigation,
/// just a heading picked now and then and a shove along it once per tentacle
/// stroke.
pub struct SquidMovement {
    movement_vector: AtomicCell<Vector3<f64>>,
    tentacle_movement: AtomicCell<f32>,
    tentacle_speed: AtomicCell<f32>,
}

impl Default for SquidMovement {
    fn default() -> Self {
        Self::new()
    }
}

impl SquidMovement {
    #[must_use]
    pub fn new() -> Self {
        Self {
            movement_vector: AtomicCell::new(Vector3::default()),
            tentacle_movement: AtomicCell::new(0.0),
            tentacle_speed: AtomicCell::new(random_tentacle_speed(&mut rand::rng())),
        }
    }

    /// One tick of vanilla's `Squid.aiStep`, with its two goals folded in:
    /// `SquidFleeGoal` steers away from whoever hurt the squid, and otherwise
    /// `SquidRandomMovementGoal` picks a new heading about every 50 ticks.
    pub fn tick(&self, mob: &dyn Mob) {
        let living = &mob.get_mob_entity().living_entity;
        let entity = &living.entity;
        let in_water = entity.touching_water.load(Ordering::Relaxed);
        let mut rng = mob.get_random();

        let fleeing = in_water && self.flee(living);
        if !fleeing
            && (rng.random_range(0..50) == 0
                || !in_water
                || self.movement_vector.load() == Vector3::default())
        {
            let angle = rng.random::<f32>() * TAU;
            self.movement_vector.store(Vector3::new(
                f64::from(angle.cos() * 0.2),
                f64::from(-0.1 + rng.random::<f32>() * 0.2),
                f64::from(angle.sin() * 0.2),
            ));
        }

        let mut tentacle = self.tentacle_movement.load() + self.tentacle_speed.load();
        if tentacle > TAU {
            tentacle -= TAU;
            if rng.random_range(0..10) == 0 {
                self.tentacle_speed.store(random_tentacle_speed(&mut rng));
            }
            // Keeps the client's stroke animation in step with the shoves.
            entity
                .world
                .load()
                .send_entity_status(entity, EntityStatus::SquidAnimSynch, None);
        }
        self.tentacle_movement.store(tentacle);

        if in_water {
            // Plain stores throughout: the tracker syncs motion, and set_velocity
            // would send a packet every tick.
            if tentacle < PI {
                if tentacle / PI > 0.75 {
                    entity.velocity.store(self.movement_vector.load());
                }
            } else {
                entity.velocity.store(entity.velocity.load() * 0.9);
            }
            // The body swings round to face where the squid is going.
            let velocity = entity.velocity.load();
            let heading = -(velocity.x.atan2(velocity.z) as f32).to_degrees();
            let body_yaw = entity.body_yaw.load();
            let body_yaw = body_yaw + (heading - body_yaw) * 0.1;
            entity.body_yaw.store(body_yaw);
            entity.yaw.store(body_yaw);
        } else {
            let fall = entity.velocity.load().y - living.get_attribute_value(&Attributes::GRAVITY);
            entity.velocity.store(Vector3::new(0.0, fall * 0.98, 0.0));
        }
    }

    /// Vanilla's `Squid.travel`: a squid only ever moves by the velocity `tick`
    /// gave it.
    pub fn travel(mob: &dyn Mob, caller: &dyn EntityBase) -> bool {
        mob.get_mob_entity().living_entity.make_move(caller);
        true
    }

    /// `SquidFleeGoal`: while whoever last hurt the squid is within ten blocks,
    /// head straight away from them. Returns whether the squid is fleeing.
    fn flee(&self, living: &LivingEntity) -> bool {
        let entity = &living.entity;
        let attacker_id = living.last_attacker_id.load(Ordering::Relaxed);
        // Vanilla forgets an attacker a hundred ticks after the hit.
        let since_hit =
            entity.age.load(Ordering::Relaxed) - living.last_attacked_time.load(Ordering::Relaxed);
        if attacker_id == 0 || since_hit > 100 {
            return false;
        }
        let world = entity.world.load();
        let Some(attacker) = world.get_entity_by_id(attacker_id) else {
            return false;
        };
        let pos = entity.pos.load();
        let mut away = pos - attacker.get_entity().pos.load();
        if away.length_squared() >= 100.0 {
            return false;
        }

        let probe = BlockPos::floored(pos.x + away.x, pos.y + away.y, pos.z + away.z);
        let open_air = world.get_block_state(&probe).is_air();
        if goal_utils::is_water(&world, &probe) || open_air {
            let length = away.length();
            if length > 0.0 {
                // Vanilla normalizes a copy of the offset here and discards it,
                // so the scale applies to the raw offset.
                let mut scale = 3.0;
                if length > 5.0 {
                    scale -= (length - 5.0) / 5.0;
                }
                if scale > 0.0 {
                    away = away * scale;
                }
            }
            if open_air {
                away.y = 0.0;
            }
            self.movement_vector.store(away / 20.0);
        }
        true
    }
}

fn random_tentacle_speed(rng: &mut impl RngExt) -> f32 {
    1.0 / (rng.random::<f32>() + 1.0) * 0.2
}
