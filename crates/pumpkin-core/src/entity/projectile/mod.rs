use super::{Entity, EntityBase, living::LivingEntity};
use crate::world::World;
use pumpkin_data::BlockDirection;
use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::Taggable;
use pumpkin_protocol::java::client::play::CEntityVelocity;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use std::{
    sync::Arc,
    sync::atomic::{AtomicBool, Ordering},
};
pub mod arrow;
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

#[must_use]
pub fn is_projectile(entity_type: &EntityType) -> bool {
    *entity_type == EntityType::ARROW
        || *entity_type == EntityType::TRIDENT
        || *entity_type == EntityType::EGG
        || *entity_type == EntityType::EXPERIENCE_BOTTLE
        || *entity_type == EntityType::SNOWBALL
        || *entity_type == EntityType::FIREWORK_ROCKET
        || *entity_type == EntityType::WIND_CHARGE
        || *entity_type == EntityType::SPLASH_POTION
        || *entity_type == EntityType::LINGERING_POTION
        || *entity_type == EntityType::ENDER_PEARL
        || *entity_type == EntityType::SHULKER_BULLET
        || *entity_type == EntityType::FIREBALL
        || *entity_type == EntityType::SMALL_FIREBALL
        || *entity_type == EntityType::FISHING_BOBBER
        || *entity_type == EntityType::WITHER_SKULL
        || *entity_type == EntityType::LLAMA_SPIT
}

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
    pub collides_with_projectiles: bool,
    pub has_hit: AtomicBool,
    pub gravity: f64,
}

impl ThrownItemEntity {
    pub fn new(entity: Entity, owner: &Entity, gravity: f64) -> Self {
        let mut owner_pos = owner.pos.load();
        owner_pos.y += owner.get_eye_height() - f64::from(0.1f32);
        entity.set_pos(owner_pos);
        Self {
            entity,
            owner: owner::ProjectileOwner::new(Some(owner)),
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
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
    /// Process a tick for projectile movement and collisions
    pub fn process_tick(&self, caller: &dyn EntityBase) {
        let entity = self.get_entity();
        let world = entity.world.load();

        self.owner.check_left_owner(entity);
        entity.update_last_pos();

        self.handle_first_tick_bubble_column(caller);

        // Apply gravity and inertia
        let mut velocity = entity.velocity.load();
        if !entity.has_no_gravity() {
            velocity.y -= self.get_gravity();
        }

        let inertia = if entity.touching_water.load(Ordering::Relaxed) {
            f64::from(0.8f32)
        } else {
            f64::from(0.99f32)
        };
        velocity = velocity.multiply(inertia, inertia, inertia);

        // Store velocity
        entity.velocity.store(velocity);

        let start_pos = entity.pos.load();
        let delta = velocity;

        let new_pos = start_pos.add(&delta);

        // Calculate search box for collisions
        let search_box = entity
            .bounding_box
            .load()
            .expand_towards(delta.x, delta.y, delta.z)
            .expand_all(1.0);
        let margin = f64::from(
            ((entity.tick_count.load(Ordering::Relaxed) as f32 - 2.0) / 20.0).clamp(0.0, 0.3),
        );

        let mut closest_t = 1.0f64;
        let mut hit = None;

        // Block collisions
        let (block_cols, block_positions) = world.get_block_collisions(search_box, caller);
        for (idx, bb) in block_cols.iter().enumerate() {
            if let Some(t) = calculate_ray_intersection(&start_pos, &delta, bb)
                && t < closest_t
            {
                closest_t = t;
                // Map back to block pos
                for (len, pos) in &block_positions {
                    if idx < *len {
                        let hit_pos = start_pos.add(&delta.multiply(t, t, t));
                        hit = Some(ProjectileHit::Block {
                            pos: *pos,
                            face: get_hit_face(hit_pos, *pos),
                            hit_pos,
                            normal: delta.normalize().multiply(-1.0, -1.0, -1.0),
                        });
                        break;
                    }
                }
            }
        }

        // Entity collisions
        let mut candidates = world.get_entities_at_box(&search_box);
        candidates.extend(
            world
                .get_players_at_box(&search_box)
                .into_iter()
                .map(|player| player as Arc<dyn EntityBase>),
        );
        for cand in candidates {
            if self.should_skip_collision(&world, entity, &cand) {
                continue;
            }

            let ebb = cand.get_entity().bounding_box.load().expand_all(margin);
            if skips_hit_from_inside(cand.as_ref(), &ebb, &start_pos) {
                continue;
            }
            if let Some(t) = calculate_ray_intersection(&start_pos, &delta, &ebb)
                && t < closest_t
            {
                closest_t = t;
                let hit_pos = start_pos.add(&delta.multiply(t, t, t));
                hit = Some(ProjectileHit::Entity {
                    entity: cand.clone(),
                    hit_pos,
                    normal: delta.normalize().multiply(-1.0, -1.0, -1.0),
                });
            }
        }

        entity.set_pos(hit.as_ref().map_or(new_pos, ProjectileHit::hit_pos));
        self.update_rotation();
        entity.tick_block_collisions(caller);
        self.owner.emit_shoot_event(entity);
        entity.base_tick(caller);

        // Handle hit or continue
        if !entity.is_removed()
            && let Some(h) = hit
        {
            self.hit_target_or_deflect_self(caller, h);
        }
        if !entity.is_removed() {
            let packet = CEntityVelocity::new(entity.entity_id.into(), entity.velocity.load());
            world.broadcast_to_chunk(entity.chunk_pos.load(), &packet);
        }
    }

    fn update_rotation(&self) {
        use pumpkin_util::math::{atan2, rot_lerp};
        let entity = self.get_entity();
        let velocity = entity.velocity.load();
        let yaw = (atan2(velocity.x, velocity.z) * 180.0 / f64::from(std::f32::consts::PI)) as f32;
        let pitch = (atan2(velocity.y, velocity.horizontal_length()) * 180.0
            / f64::from(std::f32::consts::PI)) as f32;
        entity.set_rotation(
            rot_lerp(0.2, entity.yaw.load(), yaw),
            rot_lerp(0.2, entity.pitch.load(), pitch),
        );
    }

    fn handle_first_tick_bubble_column(&self, caller: &dyn EntityBase) {
        let entity = self.get_entity();
        let world = entity.world.load();
        if entity.first_tick.load(Ordering::Relaxed)
            && let Some(server) = world.server.upgrade()
        {
            let bounds = entity.bounding_box.load();
            for pos in BlockPos::iterate(bounds.min_block_pos(), bounds.max_block_pos()) {
                let (block, state) = world.get_block_and_state(&pos);
                if block == &pumpkin_data::Block::BUBBLE_COLUMN {
                    world
                        .block_registry
                        .on_entity_collision(block, &world, caller, &pos, state, &server);
                }
            }
        }
    }

    fn hit_target_or_deflect_self(&self, caller: &dyn EntityBase, hit: ProjectileHit) {
        if self.try_deflect(caller, &hit) {
            return;
        }
        let world = self.entity.world.load();
        if let ProjectileHit::Block { pos, .. } = &hit
            && world.get_block_state(pos).is_air()
        {
            return;
        }
        if self.has_hit.swap(true, Ordering::SeqCst) {
            return;
        }
        if let ProjectileHit::Block { pos, hit_pos, .. } = &hit {
            let block = world.get_block(pos);
            let state = world.get_block_state(pos);
            if let Some(server) = world.server.upgrade() {
                world
                    .block_registry
                    .on_projectile_hit(block, &world, caller, pos, state, hit_pos, &server);
            }
        }
        if let ProjectileHit::Entity { entity: target, .. } = &hit
            && target
                .get_entity()
                .entity_type
                .has_tag(&pumpkin_data::tag::EntityType::MINECRAFT_REDIRECTABLE_PROJECTILE)
        {
            let owner = self.owner.get(&world);
            let deflection =
                crate::entity::projectile_deflection::ProjectileDeflectionType::Redirected;
            let deflected = target
                .cast_any()
                .downcast_ref::<wind_charge::WindChargeEntity>()
                .map_or_else(
                    || {
                        deflection.deflect_entity(target.as_ref(), owner.as_deref());
                        if let Some(fireball) =
                            target.cast_any().downcast_ref::<fireball::FireballEntity>()
                        {
                            fireball.on_deflection(true);
                        }
                        true
                    },
                    |charge| charge.deflect(&deflection, owner.as_deref()),
                );
            if deflected && let Some(target_owner) = target.get_projectile_owner() {
                target_owner.copy_owner_from(&self.owner);
            }
        }
        caller.on_hit(hit);
        self.entity.remove();
    }

    fn try_deflect(&self, caller: &dyn EntityBase, hit: &ProjectileHit) -> bool {
        let ProjectileHit::Entity { entity: target, .. } = hit else {
            return false;
        };
        let target_entity = target.get_entity();
        if !target_entity
            .entity_type
            .has_tag(&pumpkin_data::tag::EntityType::MINECRAFT_DEFLECTS_PROJECTILES)
            || (target_entity.entity_type == &EntityType::BREEZE
                && (self.entity.entity_type == &EntityType::WIND_CHARGE
                    || self.entity.entity_type == &EntityType::BREEZE_WIND_CHARGE))
        {
            return false;
        }
        if self.owner.mark_deflected_by(target_entity) {
            crate::entity::projectile_deflection::ProjectileDeflectionType::Simple
                .deflect_entity(caller, Some(target.as_ref()));
            if let Some(fireball) = caller.cast_any().downcast_ref::<fireball::FireballEntity>() {
                fireball.on_deflection(false);
            }
            if target_entity.entity_type == &EntityType::BREEZE {
                target_entity.world.load().play_sound(
                    pumpkin_data::sound::Sound::EntityBreezeDeflect,
                    target.sound_source(),
                    &target_entity.pos.load(),
                );
            }
        }
        true
    }

    /// Returns if collision should be skipped (e.g. owner or projectile vs projectile)
    fn should_skip_collision(
        &self,
        world: &World,
        self_ent: &Entity,
        other: &Arc<dyn EntityBase>,
    ) -> bool {
        let other_ent = other.get_entity();
        if other_ent.entity_id == self_ent.entity_id {
            return true;
        }

        if other.is_spectator() {
            return true;
        }

        if !self.owner.can_hit(world, other_ent) {
            return true;
        }

        // Not pickable in vanilla: projectiles pass through these
        // TODO: replace with `!can_hit()` once Player, TNT, falling blocks, end crystals, item
        // frames, shulker bullets and redirectable projectiles override it like vanilla `isPickable`
        if *other_ent.entity_type == EntityType::AREA_EFFECT_CLOUD
            || *other_ent.entity_type == EntityType::EXPERIENCE_ORB
            || *other_ent.entity_type == EntityType::ITEM
        {
            return true;
        }

        // Projectile vs projectile logic
        if !self.collides_with_projectiles
            && is_projectile(other_ent.entity_type)
            && !other_ent
                .entity_type
                .has_tag(&pumpkin_data::tag::EntityType::MINECRAFT_REDIRECTABLE_PROJECTILE)
        {
            return true;
        }

        false
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

/// Ray intersection algorithm for AABBs, returning a t value
/// Whether a ray starting inside `bb` skips this entity, for mobs that opt out of inside hits.
fn skips_hit_from_inside(entity: &dyn EntityBase, bb: &BoundingBox, start: &Vector3<f64>) -> bool {
    (bb.min.x..bb.max.x).contains(&start.x)
        && (bb.min.y..bb.max.y).contains(&start.y)
        && (bb.min.z..bb.max.z).contains(&start.z)
        && entity
            .get_mob()
            .is_some_and(|mob| !mob.can_be_picked_from_inside())
}

fn calculate_ray_intersection(
    start: &Vector3<f64>,
    dir: &Vector3<f64>,
    bb: &BoundingBox,
) -> Option<f64> {
    let mut t_min = 0.0f64;
    let mut t_max = 1.0f64;

    let b_min = [bb.min.x, bb.min.y, bb.min.z];
    let b_max = [bb.max.x, bb.max.y, bb.max.z];
    let s = [start.x, start.y, start.z];
    let d = [dir.x, dir.y, dir.z];

    for i in 0..3 {
        if d[i].abs() < 1e-9 {
            if s[i] < b_min[i] || s[i] > b_max[i] {
                return None;
            }
        } else {
            let t1 = (b_min[i] - s[i]) / d[i];
            let t2 = (b_max[i] - s[i]) / d[i];
            t_min = t_min.max(t1.min(t2));
            t_max = t_max.min(t1.max(t2));
        }
    }

    (t_min <= t_max && (0.0..=1.0).contains(&t_min)).then_some(t_min)
}

/// Get the face of the block that was hit
fn get_hit_face(hit_pos: Vector3<f64>, block_pos: BlockPos) -> BlockDirection {
    let local = hit_pos.sub(&block_pos.0.to_f64());
    let eps = 1.0e-4;

    if local.x <= eps {
        BlockDirection::West
    } else if local.x >= 1.0 - eps {
        BlockDirection::East
    } else if local.y <= eps {
        BlockDirection::Down
    } else if local.y >= 1.0 - eps {
        BlockDirection::Up
    } else if local.z <= eps {
        BlockDirection::North
    } else {
        BlockDirection::South
    }
}

pub enum ProjectileHit {
    Block {
        pos: BlockPos,
        face: BlockDirection,
        hit_pos: Vector3<f64>,
        normal: Vector3<f64>,
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
