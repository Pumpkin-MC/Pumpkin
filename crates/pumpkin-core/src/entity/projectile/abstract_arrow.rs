use pumpkin_data::entity::EntityType;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockStateId;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::CEntityVelocity;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::signum;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::block::BlockStateCodec;
use pumpkin_world::clip::{ClipBlock, ClipContext, ClipFluid};

use super::owner::ProjectileOwner;
use super::{HitOutcome, ProjectileHit, compute_margin, hit_target_or_deflect_self};
use crate::entity::{Entity, EntityBase};
use crate::server::Server;

/// How long an arrow lies in the ground before it despawns.
pub const DESPAWN_TIME: u32 = 1200;
const AIR_INERTIA: f64 = 0.99;
const GRAVITY: f64 = 0.05;
/// Ticks a stuck arrow shakes after landing, it can't be picked up meanwhile.
pub const SHAKE_TICKS: u8 = 7;
/// Half the size of the box that has to be free of blocks before a stuck arrow falls.
const FALL_CHECK_MARGIN: f64 = 0.06;

/// Flight state arrows and tridents share.
pub struct ArrowFlight {
    pub in_ground: AtomicBool,
    pub in_ground_time: AtomicU32,
    pub life: AtomicU32,
    pub shake_time: AtomicU8,
    /// The block the arrow is stuck in, it falls once that block changes.
    pub last_state: AtomicCell<Option<BlockStateId>>,
    /// Entities a piercing arrow already went through.
    pub pierced: Mutex<Vec<i32>>,
}

impl Default for ArrowFlight {
    fn default() -> Self {
        Self::new()
    }
}

impl ArrowFlight {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            in_ground: AtomicBool::new(false),
            in_ground_time: AtomicU32::new(0),
            life: AtomicU32::new(0),
            shake_time: AtomicU8::new(0),
            last_state: AtomicCell::new(None),
            pierced: Mutex::new(Vec::new()),
        }
    }

    #[must_use]
    pub fn is_in_ground(&self) -> bool {
        self.in_ground.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn has_pierced(&self, entity: &Entity) -> bool {
        self.pierced
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .contains(&entity.entity_id)
    }

    pub fn reset_pierced(&self) {
        self.pierced
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
    }

    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_short("life", self.life.load(Ordering::Relaxed) as i16);
        if let Some(state) = self.last_state.load() {
            nbt.put_compound("inBlockState", BlockStateCodec::state_to_nbt(state));
        }
        nbt.put_byte("shake", self.shake_time.load(Ordering::Relaxed) as i8);
        nbt.put_bool("inGround", self.is_in_ground());
    }

    pub fn read_nbt(&self, nbt: &NbtCompound) {
        if let Some(life) = nbt
            .get_short("life")
            .map(i32::from)
            .or_else(|| nbt.get_int("life"))
        {
            self.life.store(life.max(0) as u32, Ordering::Relaxed);
        }
        self.last_state.store(
            nbt.get_compound("inBlockState")
                .and_then(BlockStateCodec::state_from_nbt),
        );
        if let Some(shake) = nbt.get_byte("shake") {
            self.shake_time.store(shake.max(0) as u8, Ordering::Relaxed);
        }
        if let Some(in_ground) = nbt.get_bool("inGround") {
            self.in_ground.store(in_ground, Ordering::Relaxed);
        }
    }
}

/// Whether `entity_type` is one of the `AbstractArrow` subclasses.
#[must_use]
pub fn is_abstract_arrow(entity_type: &EntityType) -> bool {
    entity_type == &EntityType::ARROW
        || entity_type == &EntityType::SPECTRAL_ARROW
        || entity_type == &EntityType::TRIDENT
}

/// An arrow or a trident.
pub trait AbstractArrow: EntityBase {
    fn flight(&self) -> &ArrowFlight;

    fn owner(&self) -> &ProjectileOwner;

    fn water_inertia(&self) -> f64;

    fn pierce_level(&self) -> u8 {
        0
    }

    /// Changes `in_ground` and tells the clients.
    fn set_in_ground(&self, in_ground: bool);

    fn tick_despawn(&self) {
        count_down_despawn(self);
    }

    /// Every entity on the segment the arrow can hit, nearest first is not guaranteed.
    fn find_hit_entities(
        &self,
        from: Vector3<f64>,
        to: Vector3<f64>,
    ) -> Vec<(Arc<dyn EntityBase>, Vector3<f64>)> {
        get_many_entity_hit_result(self.get_entity(), from, to, |other| {
            can_hit_entity(self, other)
        })
    }
}

/// Sticks the arrow into the block it hit, backed out a little so it pokes out of the face.
pub fn stick_in_block<A: AbstractArrow + ?Sized>(arrow: &A, sound: Sound) {
    let entity = arrow.get_entity();
    let velocity = entity.velocity.load();
    let offset = Vector3::new(signum(velocity.x), signum(velocity.y), signum(velocity.z))
        * f64::from(0.05f32);
    entity.set_pos(entity.pos.load() - offset);
    entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
    arrow.play_entity_sound(sound, 1.0, hit_sound_pitch());
    arrow.set_in_ground(true);
    arrow
        .flight()
        .shake_time
        .store(SHAKE_TICKS, Ordering::Relaxed);
    arrow.flight().reset_pierced();
}

/// Pitch of the sound an arrow makes when it hits something.
#[must_use]
pub fn hit_sound_pitch() -> f32 {
    1.2 / rand::random::<f32>().mul_add(0.2, 0.9)
}

/// Removes the arrow once it lay in the ground for [`DESPAWN_TIME`] ticks.
pub fn count_down_despawn<A: AbstractArrow + ?Sized>(arrow: &A) {
    if arrow.flight().life.fetch_add(1, Ordering::Relaxed) + 1 >= DESPAWN_TIME {
        arrow.get_entity().remove();
    }
}

/// The owner rule plus the entities a piercing arrow already went through.
pub fn can_hit_entity<A: AbstractArrow + ?Sized>(arrow: &A, other: &dyn EntityBase) -> bool {
    let world = arrow.get_entity().world.load();
    if other.get_player().is_some()
        && let Some(owner) = arrow.owner().get_player(&world)
        && !(owner.as_ref() as &dyn EntityBase).do_teams_allow_damage(other)
    {
        return false;
    }
    other.can_be_hit_by_projectile()
        && arrow.owner().can_hit(&world, other.get_entity())
        && !arrow.flight().has_pierced(other.get_entity())
}

/// One tick of an arrow or trident.
pub fn tick<A: AbstractArrow + ?Sized>(arrow: &A, caller: &dyn EntityBase, server: &Server) {
    let entity = arrow.get_entity();
    let flight = arrow.flight();
    let world = entity.world.load();
    let physics = !entity.no_physics.load(Ordering::Relaxed);
    let block_pos = entity.block_pos.load();
    let state = world.get_block_state(&block_pos);
    if physics && !state.is_air() {
        let position = entity.pos.load();
        if state
            .get_block_collision_shapes_at(&block_pos)
            .any(|shape| shape.shift(block_pos.0.to_f64()).contains(position))
        {
            entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
            arrow.set_in_ground(true);
        }
    }

    let shake = flight.shake_time.load(Ordering::Relaxed);
    if shake > 0 {
        flight.shake_time.store(shake - 1, Ordering::Relaxed);
    }

    if entity.is_in_water_or_rain() {
        entity.extinguish();
    }

    if flight.is_in_ground() && physics {
        if flight.last_state.load() != Some(state.id) && should_fall(arrow) {
            start_falling(arrow);
        } else {
            arrow.tick_despawn();
        }
        flight.in_ground_time.fetch_add(1, Ordering::Relaxed);
        if entity.is_alive() {
            entity.tick_block_collisions(caller);
        }
        // A stuck arrow skips the base tick, so it keeps burning until it is picked up.
        entity.update_last_pos();
        entity.set_on_fire(entity.fire_ticks.load(Ordering::Relaxed) > 0);
        return;
    }

    // The base tick stores the last position, so it has to run before the move.
    entity.tick(caller, server);
    if entity.is_removed() {
        return;
    }
    flight.in_ground_time.store(0, Ordering::Relaxed);
    let in_water = entity.touching_water.load(Ordering::Relaxed);
    if in_water {
        entity
            .velocity
            .store(entity.velocity.load() * arrow.water_inertia());
    }

    super::update_rotation(entity, !physics);
    arrow.owner().tick(entity);

    let original = entity.pos.load();
    let movement = entity.velocity.load();
    if physics {
        let block_hit = world.clip_including_border(&ClipContext::new(
            original,
            original + movement,
            ClipBlock::Collider,
            ClipFluid::None,
        ));
        step_move_and_hit(arrow, caller, server, block_hit);
    } else {
        entity.set_pos(original + movement);
        entity.tick_block_collisions(caller);
    }

    if !in_water {
        entity.velocity.store(entity.velocity.load() * AIR_INERTIA);
    }
    if physics && !flight.is_in_ground() {
        let mut velocity = entity.velocity.load();
        velocity.y -= GRAVITY;
        entity.velocity.store(velocity);
    }

    let packet = CEntityVelocity::new(entity.entity_id.into(), entity.velocity.load());
    world.broadcast_to_chunk(entity.chunk_pos.load(), &packet);
}

/// Moves to the next hit along this tick's path and handles it. A piercing arrow keeps going
/// through the entities it hits until it reaches the block or stops piercing.
fn step_move_and_hit<A: AbstractArrow + ?Sized>(
    arrow: &A,
    caller: &dyn EntityBase,
    server: &Server,
    (block_hit, world_border): (pumpkin_world::clip::BlockHitResult, bool),
) {
    let entity = arrow.get_entity();
    let world = entity.world.load();
    while entity.is_alive() {
        let initial = entity.pos.load();
        let mut hits = arrow.find_hit_entities(initial, block_hit.location);
        hits.sort_by(|(a, _), (b, _)| {
            let a = initial.squared_distance_to_vec(&a.get_entity().pos.load());
            let b = initial.squared_distance_to_vec(&b.get_entity().pos.load());
            a.total_cmp(&b)
        });
        let next = hits
            .first()
            .map_or(block_hit.location, |(_, location)| *location);
        entity.set_pos(next);
        entity.tick_block_collisions(caller);

        if hits.is_empty() {
            if entity.is_alive() && block_hit.is_hit() {
                let state = world.get_block_state(&block_hit.block_pos);
                if !state.is_air() {
                    arrow.flight().last_state.store(Some(state.id));
                }
                hit_target_or_deflect_self(
                    caller,
                    ProjectileHit::Block {
                        pos: block_hit.block_pos,
                        face: block_hit.direction,
                        hit_pos: block_hit.location,
                        normal: entity.velocity.load().normalize() * -1.0,
                        world_border,
                    },
                    server,
                );
                entity.velocity_dirty.store(true, Ordering::Relaxed);
            }
            break;
        }
        if !entity.is_alive() || entity.no_physics.load(Ordering::Relaxed) {
            break;
        }
        let mut deflected = false;
        for (target, location) in hits {
            let hit = ProjectileHit::Entity {
                entity: target,
                hit_pos: location,
                normal: entity.velocity.load().normalize() * -1.0,
            };
            if hit_target_or_deflect_self(caller, hit, server) == HitOutcome::Deflected {
                deflected = true;
                break;
            }
            if !entity.is_alive() {
                break;
            }
        }
        entity.velocity_dirty.store(true, Ordering::Relaxed);
        if arrow.pierce_level() > 0 && !deflected {
            continue;
        }
        break;
    }
}

fn should_fall<A: AbstractArrow + ?Sized>(arrow: &A) -> bool {
    let entity = arrow.get_entity();
    let position = entity.pos.load();
    arrow.flight().is_in_ground()
        && entity
            .world
            .load()
            .is_space_empty(BoundingBox::new(position, position).expand_all(FALL_CHECK_MARGIN))
}

/// Drops out of the ground with a small random part of the old speed.
fn start_falling<A: AbstractArrow + ?Sized>(arrow: &A) {
    let entity = arrow.get_entity();
    arrow.set_in_ground(false);
    let velocity = entity.velocity.load();
    entity.velocity.store(Vector3::new(
        velocity.x * f64::from(rand::random::<f32>() * 0.2),
        velocity.y * f64::from(rand::random::<f32>() * 0.2),
        velocity.z * f64::from(rand::random::<f32>() * 0.2),
    ));
    arrow.flight().life.store(0, Ordering::Relaxed);
}

/// Every entity on the segment from `from` to `to`, hit directly or within the growing margin
/// as long as no block is in the way.
pub fn get_many_entity_hit_result(
    projectile: &Entity,
    from: Vector3<f64>,
    to: Vector3<f64>,
    can_hit: impl Fn(&dyn EntityBase) -> bool,
) -> Vec<(Arc<dyn EntityBase>, Vector3<f64>)> {
    let world = projectile.world.load();
    let delta = projectile.velocity.load();
    let search_box = projectile
        .bounding_box
        .load()
        .expand_towards(delta.x, delta.y, delta.z)
        .expand_all(1.0);
    let margin = compute_margin(projectile);
    let mut hits = Vec::new();
    for candidate in super::entities_in(&world, &search_box) {
        if candidate.get_entity().entity_id == projectile.entity_id || !can_hit(candidate.as_ref())
        {
            continue;
        }
        let bounding_box = candidate.get_entity().bounding_box.load();
        if let Some(hit) = bounding_box.clip(from, to) {
            hits.push((candidate, hit));
        } else if margin > 0.0
            && let Some(outside) = bounding_box.expand_all(margin).clip(from, to)
        {
            let mut towards = bounding_box.get_center();
            let (block_hit, _) = world.clip_including_border(&ClipContext::new(
                outside,
                towards,
                ClipBlock::Collider,
                ClipFluid::None,
            ));
            if block_hit.is_hit() {
                towards = block_hit.location;
            }
            if bounding_box.clip(outside, towards).is_some() {
                hits.push((candidate, outside));
            }
        }
    }
    hits
}

/// The nearest entity on the segment, using the growing margin.
pub fn get_entity_hit_result(
    projectile: &Entity,
    from: Vector3<f64>,
    to: Vector3<f64>,
    can_hit: impl Fn(&dyn EntityBase) -> bool,
) -> Option<(Arc<dyn EntityBase>, Vector3<f64>)> {
    let world = projectile.world.load();
    let delta = projectile.velocity.load();
    let search_box = projectile
        .bounding_box
        .load()
        .expand_towards(delta.x, delta.y, delta.z)
        .expand_all(1.0);
    super::nearest_entity_hit(projectile, &world, &search_box, from, to, can_hit)
}
