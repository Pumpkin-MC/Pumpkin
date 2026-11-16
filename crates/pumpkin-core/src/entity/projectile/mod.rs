use super::{Entity, EntityBase, living::LivingEntity};
use crate::block::blocks::bubble_column;
use crate::entity::projectile_deflection::{ProjectileDeflection, deflect};
use crate::server::Server;
use crate::world::World;
use crate::world::game_event::GameEventContext;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockDirection;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::CEntityVelocity;
use pumpkin_util::GameMode;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::clip::{ClipBlock, ClipContext, ClipFluid};
use std::{sync::Arc, sync::atomic::Ordering};
pub mod abstract_arrow;
pub mod arrow;
pub mod dragon_fireball;
pub mod egg;
pub mod ender_pearl;
pub mod evoker_fangs;
pub mod experience_bottle;
pub mod eye_of_ender;
pub mod fireball;
pub mod firework_rocket;
pub mod fishing_bobber;
pub mod lingering_potion;
pub mod llama_spit;
pub mod owner;
pub mod shulker_bullet;
pub mod small_fireball;
pub mod snowball;
pub mod splash_potion;
pub mod trident;
pub mod wind_charge;
pub mod wither_skull;

use pumpkin_data::item_stack::ItemStack;

/// Helper to apply projectile spawned enchantment effects matching vanilla `Projectile::applyOnProjectileSpawned`.
pub fn apply_on_projectile_spawned(
    projectile_entity: &Entity,
    pickup_item_stack: &ItemStack,
    weapon: Option<&ItemStack>,
    arrow: Option<&arrow::ArrowEntity>,
) {
    crate::enchantment::EnchantmentHelper::on_projectile_spawned(
        pickup_item_stack,
        projectile_entity,
        arrow,
    );
    if let Some(weapon) = weapon
        && weapon.item_count > 0
        && weapon.item.id != pickup_item_stack.item.id
    {
        crate::enchantment::EnchantmentHelper::on_projectile_spawned(
            weapon,
            projectile_entity,
            arrow,
        );
    }
}

pub struct ThrownItemEntity {
    pub entity: Entity,
    pub owner: owner::ProjectileOwner,
    pub gravity: f64,
}

impl ThrownItemEntity {
    /// Starts at the shooter's eye.
    pub fn new(entity: Entity, owner: &Entity, gravity: f64) -> Self {
        let mut owner_pos = owner.pos.load();
        owner_pos.y += owner.get_eye_height() - 0.1;
        entity.set_pos(owner_pos);
        Self::owned(entity, owner, gravity)
    }

    /// Keeps the position `entity` already has.
    #[must_use]
    pub fn owned(entity: Entity, owner: &Entity, gravity: f64) -> Self {
        Self {
            entity,
            owner: owner::ProjectileOwner::new(Some(owner)),
            gravity,
        }
    }

    pub fn set_velocity_from(&self, pitch: f32, yaw: f32, roll: f32, speed: f32, divergence: f32) {
        let yaw_rad = yaw.to_radians();
        let pitch_rad = pitch.to_radians();
        let roll_rad = (pitch + roll).to_radians();

        let x = -yaw_rad.sin() * pitch_rad.cos();
        let y = -roll_rad.sin();
        let z = yaw_rad.cos() * pitch_rad.cos();

        self.set_velocity(
            f64::from(x),
            f64::from(y),
            f64::from(z),
            f64::from(speed),
            f64::from(divergence),
        );
    }

    pub fn set_velocity(&self, x: f64, y: f64, z: f64, power: f64, uncertainty: f64) {
        fn next_triangular(mode: f64, deviation: f64) -> f64 {
            deviation.mul_add(rand::random::<f64>() - rand::random::<f64>(), mode)
        }
        let velocity = Vector3::new(x, y, z)
            .normalize()
            .add_raw(
                next_triangular(0.0, 0.017_227_5 * uncertainty),
                next_triangular(0.0, 0.017_227_5 * uncertainty),
                next_triangular(0.0, 0.017_227_5 * uncertainty),
            )
            .multiply(power, power, power);

        self.entity.velocity.store(velocity);
        let len = velocity.horizontal_length();
        self.entity.set_rotation(
            velocity.x.atan2(velocity.z) as f32 * 57.295_776,
            velocity.y.atan2(len) as f32 * 57.295_776,
        );
    }
}

impl ThrownItemEntity {
    /// Moves the projectile one tick and handles what it hits.
    pub fn process_tick(&self, caller: &dyn EntityBase, server: &Server) {
        let entity = self.get_entity();
        let first_tick = entity.first_tick.load(Ordering::Relaxed);
        // The base tick stores the last position, so it has to run before the move.
        entity.tick(caller, server);
        if entity.is_removed() {
            return;
        }
        let world = entity.world.load();

        self.owner.tick(entity);
        if first_tick {
            handle_first_tick_bubble_column(caller, &world);
        }

        let mut velocity = entity.velocity.load();
        velocity.y -= self.get_gravity();
        let inertia = if entity.touching_water.load(Ordering::Relaxed) {
            WATER_INERTIA
        } else {
            AIR_INERTIA
        };
        velocity = velocity * inertia;
        entity.velocity.store(velocity);

        let hit = get_hit_result_on_move_vector(
            caller,
            |other| self.can_hit_entity(&world, other),
            ClipBlock::Collider,
        );
        let new_pos = hit
            .as_ref()
            .map_or_else(|| entity.pos.load() + velocity, ProjectileHit::hit_pos);
        entity.set_pos(new_pos);
        update_rotation(entity, false);
        entity.tick_block_collisions(caller);

        let packet = CEntityVelocity::new(entity.entity_id.into(), velocity);
        world.broadcast_to_chunk(entity.chunk_pos.load(), &packet);

        if let Some(hit) = hit
            && entity.is_alive()
            && hit_target_or_deflect_self(caller, hit, server) == HitOutcome::Hit
        {
            entity.remove();
        }
    }

    fn can_hit_entity(&self, world: &World, other: &dyn EntityBase) -> bool {
        other.can_be_hit_by_projectile() && self.owner.can_hit(world, other.get_entity())
    }

    const fn get_entity(&self) -> &Entity {
        &self.entity
    }

    #[allow(dead_code, clippy::unused_self)]
    const fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }
    const fn get_gravity(&self) -> f64 {
        self.gravity
    }
}

/// A thrown projectile spawned inside a bubble column feels it right
/// away instead of a tick late.
fn handle_first_tick_bubble_column(projectile: &dyn EntityBase, world: &World) {
    let bounding_box = projectile.get_entity().bounding_box.load();
    let min = bounding_box.min.floor_to_i32();
    let max = bounding_box.max.floor_to_i32();
    for x in min.x..=max.x {
        for y in min.y..=max.y {
            for z in min.z..=max.z {
                let pos = BlockPos::new(x, y, z);
                let (block, state) = world.get_block_and_state(&pos);
                if block == &pumpkin_data::Block::BUBBLE_COLUMN {
                    bubble_column::entity_inside(world, &pos, state.id, projectile);
                }
            }
        }
    }
}

const WATER_INERTIA: f64 = 0.8;
const AIR_INERTIA: f64 = 0.99;

/// What became of a hit handed to [`hit_target_or_deflect_self`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitOutcome {
    /// The projectile hit its target.
    Hit,
    /// The target turned the projectile away instead.
    Deflected,
    /// The block was gone by the time the projectile got there.
    Ignored,
}

/// Deflects the projectile off an entity that deflects projectiles, otherwise hits the target.
pub fn hit_target_or_deflect_self(
    projectile: &dyn EntityBase,
    hit: ProjectileHit,
    server: &Server,
) -> HitOutcome {
    let world = projectile.get_entity().world.load();
    match &hit {
        ProjectileHit::Entity { entity, .. } => {
            let deflection = entity.deflection(projectile);
            if deflection != ProjectileDeflection::None {
                if let Some(owner) = projectile.get_projectile_owner()
                    && owner.last_deflected_by() != Some(entity.get_entity().entity_id)
                    && deflect(
                        projectile,
                        deflection,
                        Some(entity.as_ref()),
                        owner.uuid(),
                        false,
                        Vector3::new(1.0, 1.0, 1.0),
                    )
                {
                    owner.set_last_deflected_by(entity.get_entity());
                }
                return HitOutcome::Deflected;
            }
        }
        ProjectileHit::Block {
            pos, world_border, ..
        } => {
            if !world_border {
                if world.get_block_state(pos).is_air() {
                    return HitOutcome::Ignored;
                }
            } else if projectile.should_bounce_on_world_border()
                && deflect(
                    projectile,
                    ProjectileDeflection::Reverse,
                    None,
                    projectile
                        .get_projectile_owner()
                        .and_then(|owner| owner.uuid()),
                    false,
                    Vector3::new(0.2, 0.2, 0.2),
                )
            {
                return HitOutcome::Deflected;
            }
        }
    }
    on_hit(projectile, hit, server);
    HitOutcome::Hit
}

fn on_hit(projectile: &dyn EntityBase, hit: ProjectileHit, server: &Server) {
    let world = projectile.get_entity().world.load();
    let land_pos = match &hit {
        ProjectileHit::Entity {
            entity, hit_pos, ..
        } => {
            if is_redirectable(entity.get_entity())
                && let Some(owner) = projectile.get_projectile_owner()
            {
                let shooter = owner.get(&world);
                deflect(
                    entity.as_ref(),
                    ProjectileDeflection::AimDeflect,
                    shooter.as_deref(),
                    owner.uuid(),
                    true,
                    Vector3::new(1.0, 1.0, 1.0),
                );
            }
            *hit_pos
        }
        ProjectileHit::Block { pos, hit_pos, .. } => {
            let (block, state) = world.get_block_and_state(pos);
            world
                .block_registry
                .on_projectile_hit(block, &world, projectile, pos, state, hit_pos, server);
            pos.to_centered_f64()
        }
    };
    projectile.on_hit(hit);
    world.emit_game_event_with_context(
        GameEvent::ProjectileLand.name(),
        land_pos,
        &GameEventContext {
            source_entity: Some(projectile),
        },
    );
}

/// A projectile that speeds itself up along its direction instead of falling: fireballs, wither
/// skulls and wind charges.
pub struct HurtingProjectile {
    pub projectile: ThrownItemEntity,
    acceleration_power: AtomicCell<f64>,
}

impl std::ops::Deref for HurtingProjectile {
    type Target = ThrownItemEntity;

    fn deref(&self) -> &ThrownItemEntity {
        &self.projectile
    }
}

impl HurtingProjectile {
    pub const DEFAULT_ACCELERATION_POWER: f64 = 0.1;
    /// Inertia of fireballs and wither skulls that aren't dangerous.
    pub const INERTIA: f64 = 0.95;
    pub const LIQUID_INERTIA: f64 = 0.8;

    #[must_use]
    pub const fn new(projectile: ThrownItemEntity, acceleration_power: f64) -> Self {
        Self {
            projectile,
            acceleration_power: AtomicCell::new(acceleration_power),
        }
    }

    /// Points the projectile along `direction` at its acceleration speed.
    pub fn assign_directional_movement(&self, direction: Vector3<f64>) {
        let speed = self.acceleration_power();
        self.entity.velocity.store(direction.normalize() * speed);
        self.entity.velocity_dirty.store(true, Ordering::Relaxed);
    }

    #[must_use]
    pub fn acceleration_power(&self) -> f64 {
        self.acceleration_power.load()
    }

    pub fn set_acceleration_power(&self, power: f64) {
        self.acceleration_power.store(power);
    }

    /// A player's hit resets the acceleration, any other deflection halves it.
    pub fn on_deflection(&self, by_attack: bool) {
        if by_attack {
            self.set_acceleration_power(Self::DEFAULT_ACCELERATION_POWER);
        } else {
            self.set_acceleration_power(self.acceleration_power() * 0.5);
        }
    }

    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_double("acceleration_power", self.acceleration_power());
    }

    pub fn read_nbt(&self, nbt: &NbtCompound) {
        self.set_acceleration_power(
            nbt.get_double("acceleration_power")
                .unwrap_or(Self::DEFAULT_ACCELERATION_POWER),
        );
    }

    /// Moves the projectile one tick and handles what it hits. `should_burn` keeps it on fire,
    /// `can_hit` narrows down the entities it can hit.
    pub fn process_tick(
        &self,
        caller: &dyn EntityBase,
        server: &Server,
        inertia: f64,
        liquid_inertia: f64,
        should_burn: bool,
        can_hit: impl Fn(&dyn EntityBase) -> bool,
    ) {
        let entity = self.get_entity();
        entity.tick(caller, server);
        if entity.is_removed() {
            return;
        }
        self.owner.tick(entity);

        let velocity = entity.velocity.load();
        let inertia = if entity.touching_water.load(Ordering::Relaxed) {
            liquid_inertia
        } else {
            inertia
        };
        let velocity = (velocity + velocity.normalize() * self.acceleration_power()) * inertia;
        entity.velocity.store(velocity);

        let world = entity.world.load();
        if !world.is_loaded(&entity.block_pos.load()) {
            entity.remove();
            return;
        }

        let hit = get_hit_result_on_move_vector(
            caller,
            |other| {
                self.can_hit_entity(&world, other)
                    && !other.get_entity().no_physics.load(Ordering::Relaxed)
                    && can_hit(other)
            },
            ClipBlock::Collider,
        );
        let new_pos = hit
            .as_ref()
            .map_or_else(|| entity.pos.load() + velocity, ProjectileHit::hit_pos);
        rotate_towards_movement(entity, 0.2);
        entity.set_pos(new_pos);
        entity.tick_block_collisions(caller);
        if should_burn {
            entity.set_on_fire_for(1.0);
        }

        let packet = CEntityVelocity::new(entity.entity_id.into(), velocity);
        world.broadcast_to_chunk(entity.chunk_pos.load(), &packet);

        if let Some(hit) = hit
            && entity.is_alive()
            && hit_target_or_deflect_self(caller, hit, server) == HitOutcome::Hit
        {
            entity.remove();
        }
    }
}

/// Turns the projectile part of the way towards its movement.
fn rotate_towards_movement(entity: &Entity, speed: f32) {
    let velocity = entity.velocity.load();
    if velocity.length_squared() == 0.0 {
        return;
    }
    let yaw = velocity.z.atan2(velocity.x).to_degrees() as f32 + 90.0;
    let pitch = velocity.horizontal_length().atan2(velocity.y).to_degrees() as f32 - 90.0;
    let previous_yaw = wrap_towards(entity.yaw.load(), yaw);
    let previous_pitch = wrap_towards(entity.pitch.load(), pitch);
    entity.set_rotation(
        speed.mul_add(yaw - previous_yaw, previous_yaw),
        speed.mul_add(pitch - previous_pitch, previous_pitch),
    );
}

/// Shifts `previous` by whole turns until it is within half a turn of `rotation`.
fn wrap_towards(mut previous: f32, rotation: f32) -> f32 {
    while rotation - previous < -180.0 {
        previous -= 360.0;
    }
    while rotation - previous >= 180.0 {
        previous += 360.0;
    }
    previous
}

/// Knocks `hurt` straight away from the projectile instead of along its flight, for projectiles
/// that burst.
#[must_use]
pub fn knockback_away_from(projectile: &Entity, hurt: &Entity) -> Option<(f64, f64)> {
    let offset = hurt.pos.load() - projectile.pos.load();
    Some((offset.x, offset.z))
}

/// Only redirectable projectiles can be hit by other projectiles.
#[must_use]
pub fn is_redirectable(entity: &Entity) -> bool {
    entity
        .entity_type
        .has_tag(&tag::EntityType::MINECRAFT_REDIRECTABLE_PROJECTILE)
}

/// Whether the projectile may change the block at `pos`. A player's projectile follows spawn protection and the world border, a mob's needs
/// the `mobGriefing` game rule.
#[must_use]
pub fn may_interact(projectile: &dyn EntityBase, world: &World, pos: &BlockPos) -> bool {
    let Some(owner) = projectile.get_projectile_owner() else {
        return true;
    };
    if let Some(player) = owner.get_player(world) {
        let x = f64::from(pos.0.x);
        let z = f64::from(pos.0.z);
        return !world.is_in_spawn_protection(&player, pos)
            && world
                .worldborder
                .lock()
                .is_ok_and(|border| border.is_within_bounds(x, z, 0.0));
    }
    owner.uuid().is_none()
        || owner.get(world).is_none()
        || world.level_info.load().game_rules.mob_griefing
}

/// Whether the projectile may break blocks it hits. Projectiles carry no `can_break` predicate, so an adventure mode owner can't break anything.
#[must_use]
pub fn may_break(projectile: &dyn EntityBase, world: &World) -> bool {
    if !projectile
        .get_entity()
        .entity_type
        .has_tag(&tag::EntityType::MINECRAFT_IMPACT_PROJECTILES)
        || !world
            .level_info
            .load()
            .game_rules
            .projectiles_can_break_blocks
    {
        return false;
    }
    let owner = projectile
        .get_projectile_owner()
        .and_then(|owner| owner.get(world));
    let adventure = match owner.as_ref().map(|owner| owner.get_player()) {
        Some(Some(player)) => player.gamemode.load() == GameMode::Adventure,
        Some(None) => false,
        None => world.server.upgrade().is_some_and(|server| {
            server
                .defaultgamemode
                .lock()
                .is_ok_and(|default| default.gamemode == GameMode::Adventure)
        }),
    };
    !adventure
}

/// Entity boxes grow by up to 0.3 as the projectile ages.
fn compute_margin(entity: &Entity) -> f64 {
    let ticks = entity.tick_count.load(Ordering::Relaxed);
    f64::from(((ticks - 2) as f32 / 20.0).clamp(0.0, 0.3))
}

/// The first block along this tick's movement, or the nearest entity before it.
pub fn get_hit_result_on_move_vector(
    projectile: &dyn EntityBase,
    can_hit_entity: impl Fn(&dyn EntityBase) -> bool,
    clip_block: ClipBlock,
) -> Option<ProjectileHit> {
    let entity = projectile.get_entity();
    let world = entity.world.load();
    let delta = entity.velocity.load();
    let from = entity.pos.load();
    let mut to = from + delta;
    let normal = delta.normalize() * -1.0;

    let (block_hit, world_border) =
        world.clip_including_border(&ClipContext::new(from, to, clip_block, ClipFluid::None));
    let mut hit = block_hit.is_hit().then(|| {
        to = block_hit.location;
        ProjectileHit::Block {
            pos: block_hit.block_pos,
            face: block_hit.direction,
            hit_pos: block_hit.location,
            normal,
            world_border,
        }
    });

    let search_box = entity
        .bounding_box
        .load()
        .expand_towards(delta.x, delta.y, delta.z)
        .expand_all(1.0);
    if let Some((entity, hit_pos)) =
        nearest_entity_hit(entity, &world, &search_box, from, to, can_hit_entity)
    {
        hit = Some(ProjectileHit::Entity {
            entity,
            hit_pos,
            normal,
        });
    }
    hit
}

/// Every entity and player whose box touches `search_box`.
fn entities_in(world: &World, search_box: &BoundingBox) -> Vec<Arc<dyn EntityBase>> {
    let mut candidates = world.get_entities_at_box(search_box);
    candidates.extend(
        world
            .get_players_at_box(search_box)
            .into_iter()
            .map(|player| player as Arc<dyn EntityBase>),
    );
    candidates
}

/// The entity in `search_box` the segment enters first, its box grown by the projectile's age.
fn nearest_entity_hit(
    projectile: &Entity,
    world: &World,
    search_box: &BoundingBox,
    from: Vector3<f64>,
    to: Vector3<f64>,
    can_hit: impl Fn(&dyn EntityBase) -> bool,
) -> Option<(Arc<dyn EntityBase>, Vector3<f64>)> {
    let margin = compute_margin(projectile);
    let mut nearest = None;
    let mut nearest_distance = f64::MAX;
    for candidate in entities_in(world, search_box) {
        if candidate.get_entity().entity_id == projectile.entity_id || !can_hit(candidate.as_ref())
        {
            continue;
        }
        let bounding_box = candidate
            .get_entity()
            .bounding_box
            .load()
            .expand_all(margin);
        if let Some(location) = bounding_box.clip(from, to) {
            let distance = from.squared_distance_to_vec(&location);
            if distance < nearest_distance {
                nearest_distance = distance;
                nearest = Some((candidate, location));
            }
        }
    }
    nearest
}

/// Turns a fifth of the way towards the movement. `reversed` faces backwards, for a returning
/// trident.
pub fn update_rotation(entity: &Entity, reversed: bool) {
    let velocity = entity.velocity.load();
    let pitch = velocity.y.atan2(velocity.horizontal_length()).to_degrees() as f32;
    let yaw = if reversed {
        (-velocity.x).atan2(-velocity.z)
    } else {
        velocity.x.atan2(velocity.z)
    }
    .to_degrees() as f32;
    entity.set_rotation(
        lerp_rotation(entity.yaw.load(), yaw),
        lerp_rotation(entity.pitch.load(), pitch),
    );
}

fn lerp_rotation(previous: f32, rotation: f32) -> f32 {
    let previous = wrap_towards(previous, rotation);
    0.2f32.mul_add(rotation - previous, previous)
}

pub enum ProjectileHit {
    Block {
        pos: BlockPos,
        face: BlockDirection,
        hit_pos: Vector3<f64>,
        normal: Vector3<f64>,
        /// The ray stopped at the world border, not at a block.
        world_border: bool,
    },
    Entity {
        entity: Arc<dyn EntityBase>,
        hit_pos: Vector3<f64>,
        normal: Vector3<f64>,
    },
}

impl ProjectileHit {
    /// Returns the exact impact coordinates regardless of what was hit.
    #[must_use]
    pub const fn hit_pos(&self) -> Vector3<f64> {
        match self {
            Self::Block { hit_pos, .. } | Self::Entity { hit_pos, .. } => *hit_pos,
        }
    }

    /// Returns the surface normal of the impact.
    #[must_use]
    pub const fn normal(&self) -> Vector3<f64> {
        match self {
            Self::Block { normal, .. } | Self::Entity { normal, .. } => *normal,
        }
    }

    /// Safely returns the face hit if it was a block, otherwise None.
    #[must_use]
    pub const fn face(&self) -> Option<BlockDirection> {
        match self {
            Self::Block { face, .. } => Some(*face),
            Self::Entity { .. } => None,
        }
    }
}
