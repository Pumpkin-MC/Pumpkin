//! Shared vanilla water-animal behavior.
//!
//! - [`WaterAnimalAir`] — the land-drowning air counter shared by every
//!   vanilla water animal (`WaterAnimal.handleAirSupply`, WaterAnimal.java:43-53
//!   and the identical `AgeableWaterCreature.handleAirSupply`,
//!   AgeableWaterCreature.java:41-51). Dolphins are NOT covered: they use a
//!   moistness counter instead of an air supply (Dolphin.java).
//! - [`SquidAi`] — the squid's goal-driven jet movement and flee behavior
//!   (Squid.java:119-171, 232-321), which never touches the navigator.

use std::sync::Arc;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::sync::atomic::{AtomicBool, AtomicI32};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityStatus;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::EntityBase;
use crate::entity::effect::Effect;
use crate::entity::mob::MobEntity;

/// Vanilla max/reset air supply: `Entity.getMaxAirSupply()` default, and the
/// literal reset value in `WaterAnimal.handleAirSupply` (WaterAnimal.java:51).
pub const MAX_AIR: i32 = 300;
/// `LivingEntity.shouldTakeDrowningDamage()`: `getAirSupply() <= -20`
/// (LivingEntity.java:502-503).
pub const DROWN_AIR_THRESHOLD: i32 = -20;
/// `hurtServer(level, damageSources().drown(), 2.0f)` (WaterAnimal.java:48).
pub const DROWN_DAMAGE: f32 = 2.0;

/// Air-supply state for water animals, making them drown on land.
///
/// Vanilla `WaterAnimal.baseTick` captures the pre-tick air supply and passes
/// it to `handleAirSupply` (WaterAnimal.java:56-64): while alive and out of
/// water the supply drops by 1 per tick; at `-20` it resets to `0` and deals
/// 2.0 drowning damage; in water (or when dead) it snaps back to 300.
pub struct WaterAnimalAir {
    air: AtomicI32,
}

impl Default for WaterAnimalAir {
    fn default() -> Self {
        Self::new()
    }
}

impl WaterAnimalAir {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            air: AtomicI32::new(MAX_AIR),
        }
    }

    #[must_use]
    pub fn air(&self) -> i32 {
        self.air.load(Relaxed)
    }

    /// Pure counter step, `WaterAnimal.handleAirSupply` (WaterAnimal.java:43-53).
    /// Returns the new air supply and whether drowning damage is due this tick.
    #[must_use]
    pub const fn advance(air: i32, alive: bool, in_water: bool) -> (i32, bool) {
        if alive && !in_water {
            // setAirSupply(preTickAirSupply - 1) — WaterAnimal.java:45.
            let air = air - 1;
            if air <= DROWN_AIR_THRESHOLD {
                // setAirSupply(0) + drown damage — WaterAnimal.java:46-48.
                (0, true)
            } else {
                (air, false)
            }
        } else {
            // In water or dead: setAirSupply(300) — WaterAnimal.java:51.
            (MAX_AIR, false)
        }
    }

    /// Per-tick hook, vanilla `WaterAnimal.baseTick` → `handleAirSupply`
    /// (WaterAnimal.java:56-64).
    pub async fn tick(&self, mob: &MobEntity, caller: &Arc<dyn EntityBase>) {
        let living = &mob.living_entity;
        let entity = &living.entity;
        // isAlive() — WaterAnimal.java:44.
        let alive = !living.dead.load(Relaxed) && living.health.load() > 0.0;
        // isInWater() == wasTouchingWater in vanilla; Pumpkin's equivalent flag.
        let in_water = entity.touching_water.load(SeqCst);

        let prev = self.air.load(Relaxed);
        let (next, drown) = Self::advance(prev, alive, in_water);
        if next != prev {
            self.air.store(next, Relaxed);
            // Vanilla syncs DATA_AIR_SUPPLY_ID on every setAirSupply change
            // (Entity.java:2697-2703).
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::AIR_SUPPLY_ID,
                    MetaDataType::INT,
                    VarInt(next),
                )],
                None,
            );
        }
        if drown {
            caller
                .damage(caller.as_ref(), DROWN_DAMAGE, DamageType::DROWN)
                .await;
        }
    }

    /// Vanilla persists the air supply as the `Air` short tag (Entity.java:2011).
    pub fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        nbt.put_short("Air", self.air.load(Relaxed) as i16);
    }

    /// Read the `Air` tag, defaulting to the max supply (Entity.java:2078).
    pub fn read_nbt(&self, nbt: &pumpkin_nbt::compound::NbtCompound) {
        let air = nbt.get_short("Air").map_or(MAX_AIR, i32::from);
        self.air.store(air, Relaxed);
    }
}

/// `Squid.getDefaultGravity()` — 0.08 (Squid.java:113-116).
const SQUID_GRAVITY: f64 = 0.08;
/// `LivingEntity.getAirDrag()` — 0.98 for non-omnidirectional air movers with
/// no AIR_DRAG_MODIFIER attribute (LivingEntity.java:2366-2368); applied to the
/// out-of-water fall in Squid.aiStep (Squid.java:167).
const SQUID_AIR_DRAG: f64 = 0.98;
/// Vanilla degrees-per-radian literal used in Squid.aiStep (Squid.java:158,161).
const RAD_TO_DEG: f64 = 57.295776;
/// `SquidFleeGoal.SQUID_FLEE_SPEED` (Squid.java:259).
const FLEE_SPEED: f64 = 3.0;
/// `SquidFleeGoal.SQUID_FLEE_MIN_DISTANCE` (Squid.java:260).
const FLEE_MIN_DISTANCE: f64 = 5.0;
/// `SquidFleeGoal.SQUID_FLEE_MAX_DISTANCE` squared: 10.0² used as `< 100.0`
/// (Squid.java:261, 275).
const FLEE_MAX_DISTANCE_SQ: f64 = 100.0;
/// Vanilla forgets `lastHurtByMob` 100 ticks after the hit
/// (LivingEntity.java:487-491).
const LAST_HURT_BY_TIMEOUT: i32 = 100;

#[derive(Clone, Copy)]
struct HurtBy {
    entity_id: i32,
    age: i32,
}

/// Server-side port of the squid's movement AI.
///
/// Vanilla squid register only `SquidRandomMovementGoal` (priority 0) and
/// `SquidFleeGoal` (priority 1) (Squid.java:64-68); neither uses navigation.
/// Movement is a stored impulse vector applied in `aiStep` on the tentacle
/// pulse rhythm (Squid.java:119-171), and `travel` just moves by the current
/// delta movement (Squid.java:203-206). Out of water the squid sinks straight
/// down and flops (Squid.java:162-170).
pub struct SquidAi {
    /// `Squid.movementVector` (Squid.java:56).
    movement_vector: AtomicCell<Vector3<f64>>,
    /// Shadow of vanilla `deltaMovement`. Pumpkin's shared travel path applies
    /// its own drag to `entity.velocity` after moving; vanilla Squid.travel
    /// applies none (Squid.java:203-206), so the authoritative delta lives here
    /// and is written into `entity.velocity` every tick before the move runs.
    delta: AtomicCell<Vector3<f64>>,
    /// `Squid.tentacleMovement` — jet pulse clock (Squid.java:49).
    tentacle_movement: AtomicCell<f32>,
    /// `Squid.tentacleSpeed` (Squid.java:54).
    tentacle_speed: AtomicCell<f32>,
    /// `Squid.rotateSpeed` (Squid.java:55). Kept for fidelity with the vanilla
    /// state machine (Squid.java:145-154); the roll animation itself is
    /// client-side.
    rotate_speed: AtomicCell<f32>,
    /// `Squid.xBodyRot` (Squid.java:45) — needed server-side for the ink burst
    /// direction (Squid.java:182-186).
    x_body_rot: AtomicCell<f32>,
    /// `LivingEntity.yBodyRot` as updated by Squid.aiStep (Squid.java:158-159).
    body_yaw: AtomicCell<f32>,
    /// `LivingEntity.lastHurtByMob` + timestamp analog
    /// (LivingEntity.java:295, 642).
    hurt_by: AtomicCell<Option<HurtBy>>,
    /// `SquidFleeGoal.fleeTicks` (Squid.java:262).
    flee_ticks: AtomicI32,
    /// Whether the flee goal was running last tick (drives `start()` semantics,
    /// Squid.java:281-283).
    fleeing: AtomicBool,
}

impl Default for SquidAi {
    fn default() -> Self {
        Self::new()
    }
}

impl SquidAi {
    #[must_use]
    pub fn new() -> Self {
        Self {
            movement_vector: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
            delta: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
            tentacle_movement: AtomicCell::new(0.0),
            // Initial roll: 1 / (nextFloat + 1) * 0.2 (Squid.java:61).
            tentacle_speed: AtomicCell::new(1.0 / (rand::random::<f32>() + 1.0) * 0.2),
            rotate_speed: AtomicCell::new(0.0),
            x_body_rot: AtomicCell::new(0.0),
            body_yaw: AtomicCell::new(0.0),
            hurt_by: AtomicCell::new(None),
            flee_ticks: AtomicI32::new(0),
            fleeing: AtomicBool::new(false),
        }
    }

    /// `Squid.hasMovementVector()` — lengthSqr > 1.0E-5 (Squid.java:217-219).
    fn has_movement_vector(&self) -> bool {
        self.movement_vector.load().length_squared() > f64::from(1.0E-5f32)
    }

    /// Per-tick entry point, called from the squid's `mob_tick` so the velocity
    /// written here is what `LivingEntity::tick` moves with — matching the
    /// vanilla order (goals, then Squid.aiStep math, then travel).
    pub async fn tick(&self, mob: &MobEntity) {
        let age = mob.living_entity.entity.age.load(Relaxed);

        // SquidRandomMovementGoal does not require updates every tick, so the
        // goal selector runs it every other tick (Goal.java:51-56).
        if age % 2 == 0 {
            self.random_movement_tick(mob);
        }
        // SquidFleeGoal.requiresUpdateEveryTick() == true (Squid.java:286-288),
        // and its higher index means it overwrites the random vector.
        self.flee_tick(mob);

        self.movement_tick(mob).await;
    }

    /// `SquidRandomMovementGoal.tick` (Squid.java:246-254).
    fn random_movement_tick(&self, mob: &MobEntity) {
        // getNoActionTime() > 100 stops idle far-away squid (Squid.java:247-249).
        if mob.no_action_time() > 100 {
            self.movement_vector.store(Vector3::new(0.0, 0.0, 0.0));
            return;
        }
        let touching_water = mob.living_entity.entity.touching_water.load(SeqCst);
        let mut rng = rand::rng();
        // nextInt(reducedTickDelay(50)) — 25 when ticked every other tick
        // (Squid.java:250, Goal.java:55-56).
        if rng.random_range(0..25) == 0 || !touching_water || !self.has_movement_vector() {
            self.movement_vector
                .store(Self::random_movement_vector(rng.random(), rng.random()));
        }
    }

    /// New random impulse (Squid.java:251-252): a horizontal direction scaled
    /// by 0.2 with a vertical component in `[-0.1, 0.1)`.
    #[must_use]
    fn random_movement_vector(angle_roll: f32, y_roll: f32) -> Vector3<f64> {
        let angle = f64::from(angle_roll) * std::f64::consts::TAU;
        Vector3::new(
            angle.cos() * 0.2,
            -0.1 + f64::from(y_roll) * 0.2,
            angle.sin() * 0.2,
        )
    }

    /// `SquidFleeGoal` (Squid.java:257-321), including the `canUse` gate.
    fn flee_tick(&self, mob: &MobEntity) {
        let entity = &mob.living_entity.entity;
        let age = entity.age.load(Relaxed);
        let world = entity.world.load();

        // lastHurtByMob expires after 100 ticks or when the attacker is gone
        // (LivingEntity.java:487-491).
        let attacker = self.hurt_by.load().and_then(|hurt| {
            if age.wrapping_sub(hurt.age) > LAST_HURT_BY_TIMEOUT {
                self.hurt_by.store(None);
                return None;
            }
            world.get_entity_by_id(hurt.entity_id)
        });
        let alive_attacker = attacker.filter(|attacker| {
            attacker
                .get_living_entity()
                .is_none_or(|living| living.health.load() > 0.0)
        });

        let pos = entity.pos.load();
        // canUse: in water, attacker set, distanceToSqr < 100 (Squid.java:272-278).
        let target = alive_attacker.filter(|attacker| {
            entity.touching_water.load(SeqCst)
                && pos.squared_distance_to_vec(&attacker.get_entity().pos.load())
                    < FLEE_MAX_DISTANCE_SQ
        });
        let Some(attacker) = target else {
            self.fleeing.store(false, Relaxed);
            return;
        };

        // start() resets fleeTicks (Squid.java:281-283).
        if !self.fleeing.swap(true, Relaxed) {
            self.flee_ticks.store(0, Relaxed);
        }
        let flee_ticks = self.flee_ticks.fetch_add(1, Relaxed) + 1; // Squid.java:292

        let attacker_pos = attacker.get_entity().pos.load();
        let flee = pos - attacker_pos; // Squid.java:297
        // Probe the block one flee-vector away (Squid.java:298-299).
        let probe = BlockPos::floored(pos.x + flee.x, pos.y + flee.y, pos.z + flee.z);
        let block_air = world.get_block_state(&probe).is_air();
        let fluid_water = world
            .get_fluid(&probe)
            .has_tag(&tag::Fluid::MINECRAFT_WATER);
        if fluid_water || block_air {
            // Squid.java:300-315.
            self.movement_vector
                .store(Self::flee_movement_vector(flee, block_air));
        }

        // Bubble trail while fleeing (Squid.java:317-319).
        if flee_ticks % 10 == 5 {
            world.spawn_particle(pos, Vector3::new(0.0, 0.0, 0.0), 0.0, 1, Particle::Bubble);
        }
    }

    /// Flee vector shaping (Squid.java:301-315). NOTE: vanilla calls
    /// `fleeTo.normalize()` and discards the immutable result (Squid.java:303),
    /// so the vector is deliberately NOT normalized here either.
    #[must_use]
    fn flee_movement_vector(flee: Vector3<f64>, target_block_is_air: bool) -> Vector3<f64> {
        let mut flee = flee;
        let length = flee.length();
        if length > 0.0 {
            let mut speed = FLEE_SPEED; // Squid.java:304
            if length > FLEE_MIN_DISTANCE {
                // Squid.java:305-307.
                speed -= (length - FLEE_MIN_DISTANCE) / FLEE_MIN_DISTANCE;
            }
            if speed > 0.0 {
                flee = flee * speed; // Squid.java:308-310
            }
        }
        if target_block_is_air {
            // Never flee upward into air (Squid.java:312-314).
            flee.y = 0.0;
        }
        flee * (1.0 / 20.0) // movementVector = fleeTo / 20 (Squid.java:315)
    }

    /// Server-relevant half of `Squid.aiStep` (Squid.java:119-171).
    async fn movement_tick(&self, mob: &MobEntity) {
        let living = &mob.living_entity;
        let entity = &living.entity;
        let world = entity.world.load();

        // Tentacle pulse clock (Squid.java:125-136). The animation itself is
        // client-side; entity event 19 keeps remote clients in phase
        // (Squid.java:134, handleEntityEvent Squid.java:209-215).
        let mut tentacle_movement = self.tentacle_movement.load() + self.tentacle_speed.load();
        if tentacle_movement > std::f32::consts::TAU {
            tentacle_movement -= std::f32::consts::TAU;
            let mut rng = rand::rng();
            // 1-in-10 re-roll of the pulse speed (Squid.java:131-133).
            if rng.random_range(0..10) == 0 {
                self.tentacle_speed
                    .store(1.0 / (rng.random::<f32>() + 1.0) * 0.2);
            }
            world.send_entity_status(entity, EntityStatus::SquidAnimSynch);
        }
        self.tentacle_movement.store(tentacle_movement);

        // Vanilla Entity.move zeroes delta-movement components on collision;
        // the actual move ran in LivingEntity::tick after our last write, so
        // fold the collision results into the shadow delta first.
        let mut delta = self.delta.load();
        if entity.horizontal_collision.load(SeqCst) {
            delta.x = 0.0;
            delta.z = 0.0;
        }
        if entity.on_ground.load(SeqCst) && delta.y < 0.0 {
            delta.y = 0.0;
        }

        if entity.touching_water.load(SeqCst) {
            // In-water jet cycle (Squid.java:137-161).
            if tentacle_movement < std::f32::consts::PI {
                let tentacle_scale = tentacle_movement / std::f32::consts::PI; // Squid.java:139
                if f64::from(tentacle_scale) > 0.75 {
                    // Thrust: apply the stored impulse (Squid.java:141-145).
                    delta = self.movement_vector.load();
                    self.rotate_speed.store(1.0);
                } else {
                    // Squid.java:147.
                    self.rotate_speed.store(self.rotate_speed.load() * 0.8);
                }
            } else {
                // Glide: decay 0.9 per tick (Squid.java:150-154).
                delta = delta * 0.9;
                self.rotate_speed.store(self.rotate_speed.load() * 0.99);
            }

            // Body rotation follows the movement (Squid.java:156-161).
            let horizontal = delta.horizontal_length();
            let mut body_yaw = self.body_yaw.load();
            body_yaw += ((-delta.x.atan2(delta.z) * RAD_TO_DEG) as f32 - body_yaw) * 0.1;
            self.body_yaw.store(body_yaw);
            entity.yaw.store(body_yaw); // setYRot(yBodyRot), Squid.java:159
            entity.head_yaw.store(body_yaw);
            entity.body_yaw.store(body_yaw);
            let x_body_rot = self.x_body_rot.load();
            self.x_body_rot.store(
                x_body_rot + ((-horizontal.atan2(delta.y) * RAD_TO_DEG) as f32 - x_body_rot) * 0.1,
            );
        } else {
            // Out of water: sink straight down and flop — never pathfind
            // (Squid.java:162-170).
            let mut y_delta = delta.y;
            // Levitation overrides the fall (Squid.java:166).
            let levitation: Option<Effect> = living
                .get_effect(&pumpkin_data::effect::StatusEffect::LEVITATION)
                .await;
            y_delta = levitation.map_or(y_delta - SQUID_GRAVITY, |effect| {
                0.05 * f64::from(i32::from(effect.amplifier) + 1)
            });
            // setDeltaMovement(0, yd * airDrag, 0) — Squid.java:167.
            delta = Vector3::new(0.0, y_delta * SQUID_AIR_DRAG, 0.0);
            let x_body_rot = self.x_body_rot.load();
            // Squid.java:169.
            self.x_body_rot
                .store(x_body_rot + (-90.0 - x_body_rot) * 0.02);
        }

        self.delta.store(delta);
        entity.velocity.store(delta);
    }

    /// Damage reaction: record the attacker (`LivingEntity.setLastHurtByMob`,
    /// LivingEntity.java:642) and burst ink if a recent attacker exists
    /// (`Squid.hurtServer`, Squid.java:174-180).
    pub fn on_hurt(
        &self,
        mob: &MobEntity,
        source: Option<&dyn EntityBase>,
        ink_particle: Particle,
        squirt_sound: Sound,
        is_baby: bool,
    ) {
        let entity = &mob.living_entity.entity;
        if let Some(source) = source.filter(|source| source.get_living_entity().is_some()) {
            self.hurt_by.store(Some(HurtBy {
                entity_id: source.get_entity().entity_id,
                age: entity.age.load(Relaxed),
            }));
        }
        // Knockback was just written to entity.velocity by the damage path;
        // fold it into the shadow delta so it is not overwritten next tick.
        self.delta.store(entity.velocity.load());
        if self.hurt_by.load().is_some() {
            self.spawn_ink(mob, ink_particle, squirt_sound, is_baby);
        }
    }

    /// `Squid.spawnInk` (Squid.java:188-197): squirt sound plus 30 directional
    /// ink particles fired backwards along the body axis.
    fn spawn_ink(
        &self,
        mob: &MobEntity,
        ink_particle: Particle,
        squirt_sound: Sound,
        is_baby: bool,
    ) {
        let entity = &mob.living_entity.entity;
        let world = entity.world.load();
        // makeSound(getSquirtSound()) — Squid.java:189. Vanilla plays it at
        // volume 0.4 (getSoundVolume, Squid.java:99-101); Pumpkin's
        // Entity::play_sound has no volume parameter, so it plays at 1.0.
        entity.play_sound(squirt_sound);

        let pos = entity.pos.load();
        let base = self.rotate_vector(Vector3::new(0.0, -1.0, 0.0)) + pos; // Squid.java:190
        let mut rng = rand::rng();
        for _ in 0..30 {
            // Squid.java:191-195.
            let direction = self.rotate_vector(Vector3::new(
                f64::from(rng.random::<f32>()) * 0.6 - 0.3,
                -1.0,
                f64::from(rng.random::<f32>()) * 0.6 - 0.3,
            ));
            let offset_scale = if is_baby { 0.1f32 } else { 0.3 }; // Squid.java:193
            let offset = direction * f64::from(offset_scale + rng.random::<f32>() * 2.0);
            // sendParticles(..., count 0, dx, dy, dz, 0.1): count 0 turns the
            // offset into a direction (Squid.java:195).
            world.spawn_particle(
                Vector3::new(base.x, base.y + 0.5, base.z),
                offset.to_f32(),
                0.1,
                0,
                ink_particle,
            );
        }
    }

    /// `Squid.rotateVector` (Squid.java:182-186): `xRot(xBodyRot°)` then
    /// `yRot(-yBodyRot°)`, with `Vec3.xRot`/`yRot` per Vec3.java:243-258.
    /// Vanilla uses the previous-tick rotations (`xBodyRotO`/`yBodyRotO`);
    /// the current values differ by at most one 0.1-lerp step.
    fn rotate_vector(&self, vec: Vector3<f64>) -> Vector3<f64> {
        let x_rot = f64::from(self.x_body_rot.load().to_radians());
        let (cos, sin) = (x_rot.cos(), x_rot.sin());
        let vec = Vector3::new(vec.x, vec.y * cos + vec.z * sin, vec.z * cos - vec.y * sin);
        let y_rot = f64::from((-self.body_yaw.load()).to_radians());
        let (cos, sin) = (y_rot.cos(), y_rot.sin());
        Vector3::new(vec.x * cos + vec.z * sin, vec.y, vec.z * cos - vec.x * sin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn air_drains_and_drowns_on_land() {
        // From a full 300 supply, the first drown tick lands when the counter
        // hits -20 (300 + 20 decrements → tick index 319), then every 20 ticks
        // after the reset to 0 (WaterAnimal.java:43-53, LivingEntity.java:502-503).
        let mut air = MAX_AIR;
        let mut damage_ticks = Vec::new();
        for tick in 0..400 {
            let (next, drown) = WaterAnimalAir::advance(air, true, false);
            air = next;
            if drown {
                damage_ticks.push(tick);
            }
        }
        assert_eq!(damage_ticks, vec![319, 339, 359, 379, 399]);
        assert_eq!(air, 0);
    }

    #[test]
    fn air_resets_in_water() {
        // setAirSupply(300) whenever in water (WaterAnimal.java:51).
        assert_eq!(WaterAnimalAir::advance(-5, true, true), (MAX_AIR, false));
        assert_eq!(WaterAnimalAir::advance(17, true, true), (MAX_AIR, false));
    }

    #[test]
    fn air_resets_when_dead() {
        // !isAlive() also takes the reset branch (WaterAnimal.java:44, 51).
        assert_eq!(WaterAnimalAir::advance(50, false, false), (MAX_AIR, false));
    }

    #[test]
    fn nbt_round_trip() {
        let air = WaterAnimalAir::new();
        let mut nbt = pumpkin_nbt::compound::NbtCompound::new();
        for _ in 0..40 {
            let (next, _) = WaterAnimalAir::advance(air.air(), true, false);
            air.air.store(next, Relaxed);
        }
        air.write_nbt(&mut nbt);
        assert_eq!(nbt.get_short("Air"), Some(260));

        let restored = WaterAnimalAir::new();
        restored.read_nbt(&nbt);
        assert_eq!(restored.air(), 260);

        // Missing tag falls back to the max supply (Entity.java:2078).
        let fresh = WaterAnimalAir::new();
        fresh.read_nbt(&pumpkin_nbt::compound::NbtCompound::new());
        assert_eq!(fresh.air(), MAX_AIR);
    }

    #[test]
    fn random_impulse_matches_vanilla_shape() {
        // Horizontal component has magnitude 0.2 and y lies in [-0.1, 0.1)
        // (Squid.java:251-252).
        for (angle_roll, y_roll) in [(0.0f32, 0.0f32), (0.25, 0.5), (0.99, 0.999)] {
            let vec = SquidAi::random_movement_vector(angle_roll, y_roll);
            assert!((vec.horizontal_length() - 0.2).abs() < 1.0e-9);
            assert!(vec.y >= -0.1 && vec.y < 0.1);
        }
    }

    #[test]
    fn flee_close_target_uses_full_speed() {
        // Within 5 blocks the speed stays 3.0 and the vector is scaled by
        // 3.0 / 20 without normalization (Squid.java:301-315).
        let vec = SquidAi::flee_movement_vector(Vector3::new(3.0, 0.0, 0.0), false);
        assert!((vec.x - 3.0 * FLEE_SPEED / 20.0).abs() < 1.0e-9);
        assert_eq!(vec.y, 0.0);
        assert_eq!(vec.z, 0.0);
    }

    #[test]
    fn flee_speed_fades_beyond_five_blocks() {
        // length 10 → speed 3 - (10-5)/5 = 2.0 (Squid.java:305-307).
        let vec = SquidAi::flee_movement_vector(Vector3::new(10.0, 0.0, 0.0), false);
        assert!((vec.x - 10.0 * 2.0 / 20.0).abs() < 1.0e-9);
    }

    #[test]
    fn flee_negative_speed_leaves_vector_unscaled() {
        // Beyond 20 blocks the computed speed is <= 0 and the scale step is
        // skipped (Squid.java:308-310); only the /20 applies.
        let vec = SquidAi::flee_movement_vector(Vector3::new(25.0, 0.0, 0.0), false);
        assert!((vec.x - 25.0 / 20.0).abs() < 1.0e-9);
    }

    #[test]
    fn flee_into_air_zeroes_vertical() {
        // Air target zeroes y before the /20 (Squid.java:312-315).
        let vec = SquidAi::flee_movement_vector(Vector3::new(3.0, 2.0, 0.0), true);
        assert_eq!(vec.y, 0.0);
        assert!(vec.x > 0.0);
    }
}
