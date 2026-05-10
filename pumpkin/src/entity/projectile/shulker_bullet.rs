use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::particle::Particle;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_protocol::java::client::play::{CEntityPositionSync, CEntityVelocity};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::mob::shulker::Axis;
use crate::entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage};
use crate::server::Server;

// Direction ordinal constants
const DIR_DOWN: u8 = 0;
const DIR_UP: u8 = 1;
const DIR_NORTH: u8 = 2;
const DIR_SOUTH: u8 = 3;
const DIR_WEST: u8 = 4;
const DIR_EAST: u8 = 5;
const DIR_NONE: u8 = 255;

const SPEED: f64 = 0.15;

pub struct ShulkerBulletEntity {
    pub entity: Entity,
    pub owner_id: i32,
    /// Entity id of the final target; -1 = no target, 0+ = valid target
    target_id: AtomicI32,
    /// Current movement direction (direction ordinal, or `DIR_NONE`)
    current_dir: AtomicU8,
    /// How many ticks remain on the current axis segment
    flight_steps: AtomicI32,
    /// Steering velocity components (updated each tick)
    target_delta_x: AtomicCell<f64>,
    target_delta_y: AtomicCell<f64>,
    target_delta_z: AtomicCell<f64>,
    has_hit: AtomicBool,
    /// Tick counter; bullet is discarded after ~150 ticks if it never reaches its target
    age: AtomicI32,
}

use std::sync::atomic::AtomicU8;

impl ShulkerBulletEntity {
    pub fn new(
        owner: &Entity,
        target_id: i32,
        target_pos: Vector3<f64>,
        invalid_axis: Axis,
    ) -> Self {
        let world = owner.world.load();

        // Position at the center of the shulker's bounding box
        let bb = owner.bounding_box.load();
        let origin = Vector3::new(
            (bb.min.x + bb.max.x) * 0.5,
            (bb.min.y + bb.max.y) * 0.5,
            (bb.min.z + bb.max.z) * 0.5,
        );

        let entity = Entity::from_uuid(
            Uuid::new_v4(),
            (*world).clone(),
            origin,
            &EntityType::SHULKER_BULLET,
        );

        let bullet = Self {
            entity,
            owner_id: owner.entity_id,
            target_id: AtomicI32::new(target_id),
            current_dir: AtomicU8::new(DIR_UP),
            flight_steps: AtomicI32::new(0),
            target_delta_x: AtomicCell::new(0.0),
            target_delta_y: AtomicCell::new(0.0),
            target_delta_z: AtomicCell::new(0.0),
            has_hit: AtomicBool::new(false),
            age: AtomicI32::new(0),
        };

        // Prime target_delta and flight_steps so movement begins on tick 1.
        let avoid_axis = invalid_axis as u8;
        bullet.select_next_dir(avoid_axis, Some(target_pos));

        bullet
    }

    #[allow(clippy::similar_names)]
    fn select_next_dir(&self, avoid_axis: u8, target_pos_opt: Option<Vector3<f64>>) {
        const Y_OFFSET: f64 = 0.5;

        let pos = self.entity.pos.load();
        let cur_bp = self.entity.block_pos.load();
        let world = self.entity.world.load();

        // Compute the target block position
        let tbp = target_pos_opt.map_or_else(
            || BlockPos::new(cur_bp.0.x, cur_bp.0.y - 1, cur_bp.0.z), // Default: below
            |tp| {
                BlockPos::new(
                    // Target provided
                    tp.x.floor() as i32,
                    (tp.y + Y_OFFSET).floor() as i32,
                    tp.z.floor() as i32,
                )
            },
        );

        // ... and the "steering" target point.
        let target_x = f64::from(tbp.0.x) + 0.5;
        let target_y = f64::from(tbp.0.y) + Y_OFFSET;
        let target_z = f64::from(tbp.0.z) + 0.5;

        // If target block center is more than 2 blocks from bullet -> pick axis step.
        let tbp_cx = f64::from(tbp.0.x) + 0.5;
        let tbp_cy = f64::from(tbp.0.y) + 0.5;
        let tbp_cz = f64::from(tbp.0.z) + 0.5;
        let center_dist_sq =
            (tbp_cx - pos.x).powi(2) + (tbp_cy - pos.y).powi(2) + (tbp_cz - pos.z).powi(2);
        let target_is_far = center_dist_sq > 4.0;

        let (chosen_tx, chosen_ty, chosen_tz, selected_dir) = if target_is_far {
            // Pick an axis-aligned direction toward the target
            let mut options: Vec<u8> = Vec::new();

            if avoid_axis != 0 {
                if cur_bp.0.x < tbp.0.x && is_empty_block(&world, &cur_bp, DIR_EAST) {
                    options.push(DIR_EAST);
                } else if cur_bp.0.x > tbp.0.x && is_empty_block(&world, &cur_bp, DIR_WEST) {
                    options.push(DIR_WEST);
                }
            }
            if avoid_axis != 1 {
                if cur_bp.0.y < tbp.0.y && is_empty_block(&world, &cur_bp, DIR_UP) {
                    options.push(DIR_UP);
                } else if cur_bp.0.y > tbp.0.y && is_empty_block(&world, &cur_bp, DIR_DOWN) {
                    options.push(DIR_DOWN);
                }
            }
            if avoid_axis != 2 {
                if cur_bp.0.z < tbp.0.z && is_empty_block(&world, &cur_bp, DIR_SOUTH) {
                    options.push(DIR_SOUTH);
                } else if cur_bp.0.z > tbp.0.z && is_empty_block(&world, &cur_bp, DIR_NORTH) {
                    options.push(DIR_NORTH);
                }
            }

            // If options empty, pick a random direction (up to 5 tries for an
            // unobstructed one); ignore avoidAxis in this fallback.
            let sel = if options.is_empty() {
                let mut r = Self::random_dir();
                for _ in 0..4 {
                    if is_empty_block(&world, &cur_bp, r) {
                        break;
                    }
                    r = Self::random_dir();
                }
                r
            } else {
                options[rand::rng().random_range(0..options.len())]
            };

            (
                pos.x + f64::from(dir_step_x_s(sel)),
                pos.y + f64::from(dir_step_y_s(sel)),
                pos.z + f64::from(dir_step_z_s(sel)),
                sel,
            )
        } else {
            // If close, set selection=null and aim at the actual target position directly.
            (target_x, target_y, target_z, DIR_NONE)
        };

        // Compute normalized steering delta scaled by SPEED.
        let xa = chosen_tx - pos.x;
        let ya = chosen_ty - pos.y;
        let za = chosen_tz - pos.z;
        let dist = (xa * xa + ya * ya + za * za).sqrt();

        if dist == 0.0 {
            self.target_delta_x.store(0.0);
            self.target_delta_y.store(0.0);
            self.target_delta_z.store(0.0);
        } else {
            self.target_delta_x.store(xa / dist * SPEED);
            self.target_delta_y.store(ya / dist * SPEED);
            self.target_delta_z.store(za / dist * SPEED);
        }

        self.current_dir.store(selected_dir, Ordering::Relaxed);
        let steps = 10 + rand::rng().random_range(0..5) * 10;
        self.flight_steps.store(steps, Ordering::Relaxed);
    }

    fn random_dir() -> u8 {
        rand::rng().random_range(0u8..6)
    }
}

// Free-standing step helpers (used inside async closures that can't capture self.*)
const fn dir_step_x_s(dir: u8) -> i32 {
    match dir {
        DIR_EAST => 1,
        DIR_WEST => -1,
        _ => 0,
    }
}
const fn dir_step_y_s(dir: u8) -> i32 {
    match dir {
        DIR_UP => 1,
        DIR_DOWN => -1,
        _ => 0,
    }
}
const fn dir_step_z_s(dir: u8) -> i32 {
    match dir {
        DIR_SOUTH => 1,
        DIR_NORTH => -1,
        _ => 0,
    }
}

/// Returns the axis ordinal (0=X, 1=Y, 2=Z) of a direction.
/// Returns 255 for `DIR_NONE` or any unknown value.
const fn dir_axis(dir: u8) -> u8 {
    match dir {
        DIR_EAST | DIR_WEST => 0,
        DIR_UP | DIR_DOWN => 1,
        DIR_NORTH | DIR_SOUTH => 2,
        _ => 255,
    }
}

/// `true` if the block one step in `dir` from `from` is air (the bullet
/// can pass through it). Unloaded chunks are treated as empty.
fn is_empty_block(world: &crate::world::World, from: &BlockPos, dir: u8) -> bool {
    let nb = BlockPos::new(
        from.0.x + dir_step_x_s(dir),
        from.0.y + dir_step_y_s(dir),
        from.0.z + dir_step_z_s(dir),
    );
    world
        .get_block_state_if_loaded(&nb)
        .is_none_or(pumpkin_data::BlockState::is_air)
}

impl ShulkerBulletEntity {
    /// Create a `ShulkerBulletEntity` from a pre-existing Entity (e.g. loaded from
    /// disk or spawned via the /summon command with no context).
    pub const fn orphan(entity: Entity) -> Self {
        Self {
            entity,
            owner_id: 0,
            target_id: AtomicI32::new(-1),
            current_dir: AtomicU8::new(DIR_NONE),
            flight_steps: AtomicI32::new(0),
            target_delta_x: AtomicCell::new(0.0),
            target_delta_y: AtomicCell::new(0.0),
            target_delta_z: AtomicCell::new(0.0),
            has_hit: AtomicBool::new(false),
            age: AtomicI32::new(0),
        }
    }
}

impl NBTStorage for ShulkerBulletEntity {}

impl EntityBase for ShulkerBulletEntity {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    /// Any hit destroys the bullet (melee, arrow, etc.).
    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            // Guard against double-hit
            if self.has_hit.swap(true, Ordering::SeqCst) {
                return false;
            }
            let entity = &self.entity;
            let world = entity.world.load();
            let pos = entity.pos.load();
            world
                .play_sound_fine(
                    Sound::EntityShulkerBulletHit,
                    SoundCategory::Hostile,
                    &pos,
                    1.0,
                    1.0,
                )
                .await;
            world
                .spawn_particle(
                    pos,
                    Vector3::new(0.2, 0.2, 0.2),
                    0.0,
                    2,
                    Particle::Explosion,
                )
                .await;
            entity.remove().await;
            true
        })
    }

    #[allow(clippy::too_many_lines)]
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        _server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.has_hit.load(Ordering::Relaxed) {
                return;
            }

            // Discard bullet after 150 ticks if it never hits anything
            let age = self.age.fetch_add(1, Ordering::Relaxed) + 1;
            if age > 150 {
                if !self.has_hit.swap(true, Ordering::SeqCst) {
                    let entity = &self.entity;
                    let world = entity.world.load();
                    let pos = entity.pos.load();
                    world
                        .play_sound_fine(
                            Sound::EntityShulkerBulletHit,
                            SoundCategory::Hostile,
                            &pos,
                            1.0,
                            1.0,
                        )
                        .await;
                    world
                        .spawn_particle(
                            pos,
                            Vector3::new(0.2, 0.2, 0.2),
                            0.0,
                            2,
                            Particle::Explosion,
                        )
                        .await;
                    entity.remove().await;
                }
                return;
            }

            let entity = &self.entity;
            let world = entity.world.load();

            let target_id = self.target_id.load(Ordering::Relaxed);
            let target_opt = if target_id >= 0 {
                world.get_entity_by_id(target_id)
            } else {
                None
            };

            // Apply gravity only if target is null, dead, or a spectator.
            let target_alive = target_opt
                .as_ref()
                .is_some_and(|t| t.get_entity().is_alive());

            if !target_alive && target_id >= 0 {
                // Target ID is set but entity was not found (dead/left the world).
                // Only permanently clear the target when we're sure it's gone.
                if let Some(t) = &target_opt {
                    if !t.get_entity().is_alive() {
                        self.target_id.store(-1, Ordering::Relaxed);
                    }
                } else {
                    // Not found -> clear target
                    self.target_id.store(-1, Ordering::Relaxed);
                }
            }

            if target_alive {
                // Accelerate target deltas x 1.025, clamped
                let mut tdx = (self.target_delta_x.load() * 1.025).clamp(-1.0, 1.0);
                let mut tdy = (self.target_delta_y.load() * 1.025).clamp(-1.0, 1.0);
                let mut tdz = (self.target_delta_z.load() * 1.025).clamp(-1.0, 1.0);
                self.target_delta_x.store(tdx);
                self.target_delta_y.store(tdy);
                self.target_delta_z.store(tdz);

                // Clamp initialised sentinel values
                if tdx.abs() < 1e-10 && tdy.abs() < 1e-10 && tdz.abs() < 1e-10 {
                    tdx = 0.0;
                    tdy = 0.0;
                    tdz = 0.0;
                }

                // Lerp actual velocity toward steering delta
                let mut vel = entity.velocity.load();
                vel.x += (tdx - vel.x) * 0.2;
                vel.y += (tdy - vel.y) * 0.2;
                vel.z += (tdz - vel.z) * 0.2;
                entity.velocity.store(vel);
            } else {
                // No live target – apply gravity and drift
                let mut vel = entity.velocity.load();
                vel.y -= 0.04;
                entity.velocity.store(vel);
            }

            let vel = entity.velocity.load();
            let old_pos = entity.pos.load();
            let new_pos = old_pos.add(&vel);
            entity.set_pos(new_pos);

            // Broadcast position and velocity
            world
                .broadcast_packet_all(&CEntityPositionSync::new(
                    entity.entity_id.into(),
                    new_pos,
                    vel,
                    entity.yaw.load(),
                    entity.pitch.load(),
                    false,
                ))
                .await;
            world
                .broadcast_packet_all(&CEntityVelocity::new(entity.entity_id.into(), vel))
                .await;

            // Check for block collisions
            let new_bp = entity.block_pos.load();
            let state = world.get_block_state(&new_bp).await;
            if !state.is_air() && state.is_solid() {
                if !self.has_hit.swap(true, Ordering::SeqCst) {
                    let pos = entity.pos.load();
                    world
                        .play_sound_fine(
                            Sound::EntityShulkerBulletHit,
                            SoundCategory::Hostile,
                            &pos,
                            1.0,
                            1.0,
                        )
                        .await;
                    world
                        .spawn_particle(
                            pos,
                            Vector3::new(0.2, 0.2, 0.2),
                            0.0,
                            2,
                            Particle::Explosion,
                        )
                        .await;
                    entity.remove().await;
                }
                return;
            }

            // Check for entity collisions
            let bullet_bb = entity.bounding_box.load().expand(0.1, 0.1, 0.1);
            let nearby_entities = world.get_entities_at_box(&bullet_bb);
            let nearby_players = world.get_players_at_box(&bullet_bb);
            let nearby: Vec<Arc<dyn crate::entity::EntityBase>> = nearby_entities
                .into_iter()
                .chain(
                    nearby_players
                        .into_iter()
                        .map(|p| p as Arc<dyn crate::entity::EntityBase>),
                )
                .collect();
            for hit_entity in nearby {
                let he = hit_entity.get_entity();
                // Skip self
                if he.entity_id == entity.entity_id {
                    continue;
                }
                // Never hit the owner shulker
                if he.entity_id == self.owner_id {
                    continue;
                }
                // Must be alive
                if !he.is_alive() {
                    continue;
                }
                // Must be a living entity
                let Some(living) = hit_entity.get_living_entity() else {
                    continue;
                };
                if !living.entity.is_alive() {
                    continue;
                }

                if self.has_hit.swap(true, Ordering::SeqCst) {
                    break;
                }

                // Deal 4 (MOB_PROJECTILE) damage
                let owner_arc = world.get_entity_by_id(self.owner_id);
                let damaged = hit_entity
                    .damage_with_context(
                        hit_entity.as_ref(),
                        4.0,
                        DamageType::MOB_PROJECTILE,
                        None,
                        owner_arc.as_deref(),
                        Some(caller.as_ref()),
                    )
                    .await;

                if damaged {
                    // Apply levitation for 200 ticks
                    living
                        .add_effect(Effect {
                            effect_type: &StatusEffect::LEVITATION,
                            duration: 200,
                            amplifier: 0,
                            ambient: false,
                            show_particles: true,
                            show_icon: true,
                            blend: false,
                        })
                        .await;
                }

                let pos = entity.pos.load();
                world
                    .spawn_particle(
                        pos,
                        Vector3::new(0.2, 0.2, 0.2),
                        0.0,
                        2,
                        Particle::Explosion,
                    )
                    .await;
                entity.remove().await;
                break;
            }

            if !target_alive || self.has_hit.load(Ordering::Relaxed) {
                return;
            }

            let target_pos = target_opt.as_ref().map(|t| t.get_entity().pos.load());

            let raw_dir = self.current_dir.load(Ordering::Relaxed);
            let avoid_axis = dir_axis(raw_dir);

            // Decrement flight-step counter; re-select direction when it hits 0
            let steps = self.flight_steps.fetch_sub(1, Ordering::Relaxed) - 1;
            if steps <= 0 {
                self.select_next_dir(avoid_axis, target_pos);
            }

            // Check the block immediately ahead.
            // If it is solid we must re-select; if we've aligned axes with the target we also re-select.
            let dir = self.current_dir.load(Ordering::Relaxed);
            if dir != DIR_NONE {
                let cur_bp = entity.block_pos.load();
                if !is_empty_block(&world, &cur_bp, dir) {
                    // Solid obstacle -> navigate around it
                    self.select_next_dir(dir_axis(dir), target_pos);
                } else if let Some(tp) = target_pos {
                    let axis = dir_axis(dir);
                    let tbp = BlockPos::new(
                        tp.x.floor() as i32,
                        tp.y.floor() as i32,
                        tp.z.floor() as i32,
                    );
                    let cur_bp2 = entity.block_pos.load();
                    let reached = match axis {
                        0 => cur_bp2.0.x == tbp.0.x,
                        1 => cur_bp2.0.y == tbp.0.y,
                        _ => cur_bp2.0.z == tbp.0.z,
                    };
                    if reached {
                        // Aligned on this axis -> switch to next best axis
                        self.select_next_dir(axis, Some(tp));
                    }
                }
            }
        })
    }
}
