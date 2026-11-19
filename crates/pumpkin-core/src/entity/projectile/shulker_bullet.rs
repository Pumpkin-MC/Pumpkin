use crate::entity::projectile::owner::{EntityReference, ProjectileOwner};
use pumpkin_data::BlockDirection;
use pumpkin_nbt::compound::NbtCompound;
use std::sync::atomic::{AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::particle::Particle;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::projectile::{
    ProjectileHit, get_hit_result_on_move_vector, hit_target_or_deflect_self,
    rotate_towards_movement,
};
use crate::entity::{Entity, EntityBase};
use crate::server::Server;
use crate::world::game_event::GameEventContext;
use pumpkin_data::block_properties::Axis;
use pumpkin_data::game_event::GameEvent;
use pumpkin_util::Difficulty;
use pumpkin_world::clip::ClipBlock;

// Direction ordinal constants
const DIR_DOWN: u8 = BlockDirection::Down as u8;
const DIR_UP: u8 = BlockDirection::Up as u8;
const DIR_NORTH: u8 = BlockDirection::North as u8;
const DIR_SOUTH: u8 = BlockDirection::South as u8;
const DIR_WEST: u8 = BlockDirection::West as u8;
const DIR_EAST: u8 = BlockDirection::East as u8;
const DIR_NONE: u8 = 255;

const SPEED: f64 = 0.15;
/// How fast a bullet without a target falls.
const GRAVITY: f64 = 0.04;

pub struct ShulkerBulletEntity {
    pub entity: Entity,
    pub owner: ProjectileOwner,
    target: EntityReference,
    /// Current movement direction (direction ordinal, or `DIR_NONE`)
    current_dir: AtomicU8,
    /// How many ticks remain on the current axis segment
    flight_steps: AtomicI32,
    /// Steering velocity components (updated each tick)
    target_delta_x: AtomicCell<f64>,
    target_delta_y: AtomicCell<f64>,
    target_delta_z: AtomicCell<f64>,
}

use std::sync::atomic::AtomicU8;

impl ShulkerBulletEntity {
    pub fn new(owner: &Entity, target: &Entity, invalid_axis: Axis) -> Self {
        let world = owner.world.load();

        let origin = owner.bounding_box.load().get_center();

        let entity = Entity::from_uuid(
            Uuid::new_v4(),
            (*world).clone(),
            origin,
            &EntityType::SHULKER_BULLET,
        );

        entity.no_physics.store(true, Ordering::Relaxed);
        let bullet = Self {
            entity,
            owner: ProjectileOwner::new(Some(owner)),
            target: EntityReference::new(Some(target)),
            current_dir: AtomicU8::new(DIR_UP),
            flight_steps: AtomicI32::new(0),
            target_delta_x: AtomicCell::new(0.0),
            target_delta_y: AtomicCell::new(0.0),
            target_delta_z: AtomicCell::new(0.0),
        };

        // Prime target_delta and flight_steps so movement begins on tick 1.
        let avoid_axis = Some(invalid_axis);
        bullet.select_next_dir(avoid_axis, Some(target));

        bullet
    }

    #[allow(clippy::similar_names)]
    fn select_next_dir(&self, avoid_axis: Option<Axis>, target: Option<&Entity>) {
        let y_offset = target.map_or(0.5, |target| f64::from(target.height()) * 0.5);

        let pos = self.entity.pos.load();
        let cur_bp = self.entity.block_pos.load();
        let world = self.entity.world.load();

        // Compute the target block position
        let tbp = target.map_or_else(
            || BlockPos::new(cur_bp.0.x, cur_bp.0.y - 1, cur_bp.0.z), // Default: below
            |target| {
                let tp = target.pos.load();
                BlockPos::new(
                    // Target provided
                    tp.x.floor() as i32,
                    (tp.y + y_offset).floor() as i32,
                    tp.z.floor() as i32,
                )
            },
        );

        // ... and the "steering" target point.
        let target_x = f64::from(tbp.0.x) + 0.5;
        let target_y = f64::from(tbp.0.y) + y_offset;
        let target_z = f64::from(tbp.0.z) + 0.5;

        let target_is_far = tbp.to_centered_f64().squared_distance_to_vec(&pos) >= 4.0;

        let (chosen_tx, chosen_ty, chosen_tz, selected_dir) = if target_is_far {
            // Pick an axis-aligned direction toward the target
            let mut options = [DIR_NONE; 3];
            let mut option_count = 0;

            if avoid_axis != Some(Axis::X) {
                if cur_bp.0.x < tbp.0.x && is_empty_block(&world, &cur_bp, DIR_EAST) {
                    options[option_count] = DIR_EAST;
                    option_count += 1;
                } else if cur_bp.0.x > tbp.0.x && is_empty_block(&world, &cur_bp, DIR_WEST) {
                    options[option_count] = DIR_WEST;
                    option_count += 1;
                }
            }
            if avoid_axis != Some(Axis::Y) {
                if cur_bp.0.y < tbp.0.y && is_empty_block(&world, &cur_bp, DIR_UP) {
                    options[option_count] = DIR_UP;
                    option_count += 1;
                } else if cur_bp.0.y > tbp.0.y && is_empty_block(&world, &cur_bp, DIR_DOWN) {
                    options[option_count] = DIR_DOWN;
                    option_count += 1;
                }
            }
            if avoid_axis != Some(Axis::Z) {
                if cur_bp.0.z < tbp.0.z && is_empty_block(&world, &cur_bp, DIR_SOUTH) {
                    options[option_count] = DIR_SOUTH;
                    option_count += 1;
                } else if cur_bp.0.z > tbp.0.z && is_empty_block(&world, &cur_bp, DIR_NORTH) {
                    options[option_count] = DIR_NORTH;
                    option_count += 1;
                }
            }

            // If options empty, pick a random direction (up to 5 tries for an
            // unobstructed one); ignore avoidAxis in this fallback.
            let mut r = Self::random_dir();
            let sel = if option_count == 0 {
                for _ in 0..5 {
                    if is_empty_block(&world, &cur_bp, r) {
                        break;
                    }
                    r = Self::random_dir();
                }
                r
            } else {
                options[rand::rng().random_range(0..option_count)]
            };

            let step = BlockDirection::from_index(sel)
                .map_or(Vector3::default(), |direction| direction.to_offset());
            (
                pos.x + f64::from(step.x),
                pos.y + f64::from(step.y),
                pos.z + f64::from(step.z),
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

        self.entity.velocity_dirty.store(true, Ordering::Relaxed);
        self.current_dir.store(selected_dir, Ordering::Relaxed);
        let steps = 10 + rand::rng().random_range(0..5) * 10;
        self.flight_steps.store(steps, Ordering::Relaxed);
    }

    pub(crate) fn target(&self) -> Option<std::sync::Arc<dyn EntityBase>> {
        self.target.get(&self.entity.world.load())
    }

    fn random_dir() -> u8 {
        rand::rng().random_range(0u8..6)
    }
}

fn dir_axis(dir: u8) -> Option<Axis> {
    BlockDirection::from_index(dir).map(|direction| direction.to_axis())
}

fn adjacent(from: &BlockPos, dir: u8) -> BlockPos {
    BlockDirection::try_from(i32::from(dir))
        .map_or(*from, |direction| from.offset(direction.to_offset()))
}

fn is_empty_block(world: &crate::world::World, from: &BlockPos, dir: u8) -> bool {
    world.get_block_state(&adjacent(from, dir)).is_air()
}

impl ShulkerBulletEntity {
    /// Removes the bullet and lets vibration listeners know.
    fn destroy(&self) {
        let entity = &self.entity;
        entity.remove();
        entity.world.load().emit_game_event_with_context(
            GameEvent::EntityDamage.name(),
            entity.pos.load(),
            &GameEventContext {
                source_entity: Some(self),
            },
        );
    }

    /// Create a `ShulkerBulletEntity` from a pre-existing Entity (e.g. loaded from
    /// disk or spawned via the /summon command with no context).
    pub fn orphan(entity: Entity) -> Self {
        entity.no_physics.store(true, Ordering::Relaxed);
        Self {
            entity,
            owner: ProjectileOwner::none(),
            target: EntityReference::none(),
            current_dir: AtomicU8::new(DIR_NONE),
            flight_steps: AtomicI32::new(0),
            target_delta_x: AtomicCell::new(0.0),
            target_delta_y: AtomicCell::new(0.0),
            target_delta_z: AtomicCell::new(0.0),
        }
    }
}

impl EntityBase for ShulkerBulletEntity {
    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        self.target.write_nbt(nbt, "Target");
        let direction = self.current_dir.load(Ordering::Relaxed);
        if direction != DIR_NONE {
            nbt.put_byte("Dir", direction as i8);
        }
        nbt.put_int("Steps", self.flight_steps.load(Ordering::Relaxed));
        nbt.put_double("TXD", self.target_delta_x.load());
        nbt.put_double("TYD", self.target_delta_y.load());
        nbt.put_double("TZD", self.target_delta_z.load());
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        self.target.set_uuid(nbt.get_uuid("Target"));
        self.current_dir.store(
            nbt.get_byte("Dir")
                .and_then(
                    |value| BlockDirection::by_index(i32::from(value).unsigned_abs() as usize),
                )
                .map_or(DIR_NONE, |direction| direction as u8),
            Ordering::Relaxed,
        );
        self.flight_steps
            .store(nbt.get_int("Steps").unwrap_or(0), Ordering::Relaxed);
        self.target_delta_x
            .store(nbt.get_double("TXD").unwrap_or(0.0));
        self.target_delta_y
            .store(nbt.get_double("TYD").unwrap_or(0.0));
        self.target_delta_z
            .store(nbt.get_double("TZD").unwrap_or(0.0));
    }

    fn get_gravity(&self) -> f64 {
        GRAVITY
    }

    fn on_redirect_projectile(&self, projectile: &dyn EntityBase) {
        crate::entity::projectile_deflection::deflect(
            projectile,
            crate::entity::projectile_deflection::ProjectileDeflection::MomentumDeflect,
            Some(self),
            self.owner.uuid(),
            true,
            Vector3::new(1.0, 1.0, 1.0),
        );
    }

    fn can_hit(&self) -> bool {
        true
    }

    fn get_projectile_owner(&self) -> Option<&ProjectileOwner> {
        Some(&self.owner)
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    /// Any damage destroys the bullet.
    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        let entity = &self.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();
        self.play_entity_sound(Sound::EntityShulkerBulletHurt, 1.0, 1.0);
        world.spawn_particle(pos, Vector3::new(0.2, 0.2, 0.2), 0.0, 15, Particle::Crit);
        self.destroy();
        true
    }

    fn on_hit(&self, hit: ProjectileHit) {
        let entity = &self.entity;
        let world = entity.world.load();
        match hit {
            ProjectileHit::Entity {
                entity: ref target, ..
            } => {
                let owner = self
                    .owner
                    .get(&world)
                    .filter(|owner| owner.get_living_entity().is_some());
                let hurt = target.damage_with_context(
                    target.as_ref(),
                    4.0,
                    DamageType::MOB_PROJECTILE,
                    None,
                    Some(self),
                    owner.as_deref(),
                );
                if hurt && let Some(living) = target.get_living_entity() {
                    living.add_effect(Effect {
                        effect_type: &StatusEffect::LEVITATION,
                        duration: 200,
                        amplifier: 0,
                        ambient: false,
                        show_particles: true,
                        show_icon: true,
                        blend: false,
                    });
                }
            }
            ProjectileHit::Block { .. } => {
                let pos = entity.pos.load();
                world.spawn_particle(
                    pos,
                    Vector3::new(0.2, 0.2, 0.2),
                    0.0,
                    2,
                    Particle::Explosion,
                );
                self.play_entity_sound(Sound::EntityShulkerBulletHit, 1.0, 1.0);
            }
        }
        self.destroy();
    }

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        let entity = &self.entity;
        let world = entity.world.load();
        // Bullets vanish on peaceful before they move.
        if world.level_info.load().difficulty == Difficulty::Peaceful {
            entity.remove();
            return;
        }
        entity.tick(caller, server);
        if entity.is_removed() {
            return;
        }
        self.owner.tick(entity);

        let target_opt = self.target.get(&world);
        if target_opt.is_none() {
            self.target.clear();
        }

        let target_alive = target_opt.as_ref().is_some_and(|target| {
            target.get_living_entity().map_or_else(
                || target.get_entity().is_alive(),
                crate::entity::living::LivingEntity::is_alive,
            ) && !(target.get_player().is_some() && target.is_spectator())
        });
        let mut vel = entity.velocity.load();
        if target_alive {
            let tdx = (self.target_delta_x.load() * 1.025).clamp(-1.0, 1.0);
            let tdy = (self.target_delta_y.load() * 1.025).clamp(-1.0, 1.0);
            let tdz = (self.target_delta_z.load() * 1.025).clamp(-1.0, 1.0);
            self.target_delta_x.store(tdx);
            self.target_delta_y.store(tdy);
            self.target_delta_z.store(tdz);
            vel.x += (tdx - vel.x) * 0.2;
            vel.y += (tdy - vel.y) * 0.2;
            vel.z += (tdz - vel.z) * 0.2;
        } else if !entity.has_no_gravity.load(Ordering::Relaxed) {
            vel.y -= GRAVITY;
        }
        entity.velocity.store(vel);

        let hit = get_hit_result_on_move_vector(
            caller,
            |other| {
                other.can_be_hit_by_projectile()
                    && self.owner.can_hit(&world, other.get_entity())
                    && !other.get_entity().no_physics.load(Ordering::Relaxed)
            },
            ClipBlock::Collider,
        );
        entity.set_pos(entity.pos.load() + vel);
        entity.tick_block_collisions(caller);
        let entered_portal = entity
            .portal_manager
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|portal| portal.inside_portal_this_tick);
        if entered_portal {
            entity.tick_portal(caller);
        }

        if let Some(hit) = hit
            && entity.is_alive()
        {
            hit_target_or_deflect_self(caller, hit, server);
        }
        rotate_towards_movement(entity, 0.5);

        // Steering only follows a target that still resolves, alive or not.
        let Some(target) = target_opt else {
            return;
        };
        if entity.is_removed() {
            return;
        }
        let target_entity = Some(target.get_entity());

        let steps = self.flight_steps.load(Ordering::Relaxed);
        if steps > 0 {
            self.flight_steps.store(steps - 1, Ordering::Relaxed);
            if steps == 1 {
                let dir = self.current_dir.load(Ordering::Relaxed);
                self.select_next_dir(dir_axis(dir), target_entity);
            }
        }

        // Check the block immediately ahead.
        // If it is solid we must re-select; if we've aligned axes with the target we also re-select.
        let dir = self.current_dir.load(Ordering::Relaxed);
        if dir != DIR_NONE {
            let cur_bp = entity.block_pos.load();
            if world.loaded_and_entity_can_stand_on_face(
                &adjacent(&cur_bp, dir),
                self,
                BlockDirection::Up,
            ) {
                // Solid obstacle -> navigate around it
                self.select_next_dir(dir_axis(dir), target_entity);
            } else {
                let axis = dir_axis(dir);
                let target_block_pos = target.get_entity().block_pos.load();
                let reached = match axis {
                    Some(Axis::X) => cur_bp.0.x == target_block_pos.0.x,
                    Some(Axis::Y) => cur_bp.0.y == target_block_pos.0.y,
                    _ => cur_bp.0.z == target_block_pos.0.z,
                };
                if reached {
                    // Aligned on this axis -> switch to next best axis
                    self.select_next_dir(axis, target_entity);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use crate::entity::passive::cow::CowEntity;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn vanilla_bullet_nbt_preserves_the_target_and_flight_segment() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let bullet = ShulkerBulletEntity::orphan(Entity::new(
            world.clone(),
            Vector3::default(),
            &EntityType::SHULKER_BULLET,
        ));
        let target_uuid = Uuid::new_v4();
        let mut nbt = NbtCompound::new();
        nbt.put_uuid("Target", target_uuid);
        nbt.put_byte("Dir", 4);
        nbt.put_int("Steps", 17);
        nbt.put_double("TXD", -0.15);
        nbt.put_double("TYD", 0.25);
        nbt.put_double("TZD", 0.5);
        bullet.read_custom_nbt(&nbt);
        let mut saved = NbtCompound::new();
        bullet.write_custom_nbt(&mut saved);
        assert_eq!(saved.get_uuid("Target"), Some(target_uuid));
        assert_eq!(saved.get_byte("Dir"), Some(4));
        assert_eq!(saved.get_int("Steps"), Some(17));
        assert_eq!(saved.get_double("TXD"), Some(-0.15));
        assert_eq!(saved.get_double("TYD"), Some(0.25));
        assert_eq!(saved.get_double("TZD"), Some(0.5));
        nbt.put_string("Dir", "invalid".into());
        bullet.read_custom_nbt(&nbt);
        assert_eq!(bullet.current_dir.load(Ordering::Relaxed), DIR_NONE);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn steering_uses_target_height_and_reports_the_retained_target() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let position = Vector3::new(0.5, 64.0, 0.5);
        let owner = Entity::new(world.clone(), position, &EntityType::SHULKER);
        let target = CowEntity::new(Entity::new(world.clone(), position, &EntityType::COW));
        world.spawn_entity(target.clone());
        let bullet = ShulkerBulletEntity::new(&owner, target.get_entity(), Axis::Y);
        assert_eq!(bullet.current_dir.load(Ordering::Relaxed), DIR_NONE);
        assert_eq!(bullet.target_delta_y.load(), 0.15);
        assert_eq!(bullet.target_delta_x.load(), 0.0);
        assert!(
            crate::world::entity_islands::links_of(&bullet)
                .contains(&target.get_entity().entity_id)
        );
        target.get_entity().set_pos(Vector3::new(10.5, 64.0, 10.5));
        let shulker = crate::entity::mob::shulker::ShulkerEntity::new(owner);
        shulker.set_attach_face(BlockDirection::East);
        let bullet = ShulkerBulletEntity::new(
            shulker.get_entity(),
            target.get_entity(),
            shulker.get_attach_face().to_axis(),
        );
        assert_eq!(bullet.current_dir.load(Ordering::Relaxed), DIR_SOUTH);
        assert_eq!(
            bullet.target().map(|entity| entity.get_entity().entity_id),
            Some(target.get_entity().entity_id),
        );
        world.level.shutdown().await;
        Ok(())
    }
}
