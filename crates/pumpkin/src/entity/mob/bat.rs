use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};

use pumpkin_data::damage::DamageType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::ChunkHeightmapType;
use rand::RngExt;

use crate::entity::mob::{Mob, MobEntity};
use crate::entity::{Entity, EntityBase};
use crate::world::World;

const ROOSTING_FLAG: u8 = 1;
const CLOSE_PLAYER_DISTANCE: f64 = 4.0;
/// Vanilla: `getMinAmbientSoundDelay()` returns 80 for most mobs
const MIN_AMBIENT_SOUND_DELAY: i32 = 80;

pub struct BatEntity {
    pub mob_entity: MobEntity,
    hanging_position: Mutex<Option<BlockPos>>,
    roosting: AtomicBool,
    ambient_sound_chance: AtomicI32,
}

impl BatEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let bat = Self {
            mob_entity,
            hanging_position: Mutex::new(None),
            roosting: AtomicBool::new(true),
            ambient_sound_chance: AtomicI32::new(MIN_AMBIENT_SOUND_DELAY),
        };
        Arc::new(bat)
    }

    #[must_use]
    pub fn check_bat_spawn_rules(world: &World, pos: &BlockPos) -> bool {
        if pos.0.y >= world.get_heightmap_height(ChunkHeightmapType::WorldSurface, pos.0.x, pos.0.z)
        {
            return false;
        }
        if rand::random_bool(0.5) {
            return false;
        }
        if world.get_max_local_raw_brightness(pos) > rand::random_range(0..4) {
            return false;
        }
        let below_pos = BlockPos::new(pos.0.x, pos.0.y - 1, pos.0.z);
        if !world
            .get_block(&below_pos)
            .has_tag(&tag::Block::MINECRAFT_BATS_SPAWNABLE_ON)
        {
            return false;
        }
        true
    }

    #[must_use]
    pub fn is_roosting(&self) -> bool {
        self.roosting.load(Relaxed)
    }

    pub fn set_roosting(&self, roosting: bool) {
        self.roosting.store(roosting, Relaxed);
        let flags: u8 = if roosting { ROOSTING_FLAG } else { 0 };
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(tracked_data::bat::DATA_ID_FLAGS, flags)],
            None,
        );
    }

    fn tick_ambient_sound(&self, world: &World, pos: &Vector3<f64>) {
        let chance = self.ambient_sound_chance.fetch_sub(1, Relaxed);
        if chance <= 0 {
            self.ambient_sound_chance
                .store(MIN_AMBIENT_SOUND_DELAY, Relaxed);
            if !self.is_roosting() || rand::random_range(0..4) == 0 {
                world.play_sound_fine(
                    Sound::EntityBatAmbient,
                    SoundCategory::Ambient,
                    pos,
                    0.1,
                    0.95,
                );
            }
        }
    }

    fn tick_roosting(&self, world: &World, above_pos: &BlockPos, pos: &Vector3<f64>) {
        let entity = &self.mob_entity.living_entity.entity;
        let above_state = world.get_block_state(above_pos);
        if above_state.is_solid_block() {
            let rotate_head = {
                let mut rng = rand::rng();
                (rng.random_range(0u32..200) == 0).then(|| rng.random_range(0i32..360) as f32)
            };
            if let Some(head_yaw) = rotate_head {
                entity.head_yaw.store(head_yaw);
            }

            if world
                .get_closest_player(*pos, CLOSE_PLAYER_DISTANCE)
                .is_some()
            {
                self.set_roosting(false);
                world.play_sound_fine(
                    Sound::EntityBatTakeoff,
                    SoundCategory::Ambient,
                    pos,
                    0.1,
                    0.95,
                );
            }
        } else {
            self.set_roosting(false);
            world.play_sound_fine(
                Sound::EntityBatTakeoff,
                SoundCategory::Ambient,
                pos,
                0.1,
                0.95,
            );
        }
    }

    fn tick_flying(&self, world: &World, above_pos: &BlockPos, pos: &Vector3<f64>) {
        let entity = &self.mob_entity.living_entity.entity;
        let mut hanging_pos = self
            .hanging_position
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if let Some(hp) = *hanging_pos {
            let hp_state = world.get_block_state(&hp);
            if !hp_state.is_air() || hp.0.y <= world.dimension.min_y {
                *hanging_pos = None;
            }
        }

        let (should_pick_new, new_target, try_roost) = {
            let mut rng = rand::rng();
            let should_pick = hanging_pos.is_none()
                || rng.random_range(0u32..30) == 0
                || hanging_pos.is_some_and(|hp| {
                    let dx = f64::from(hp.0.x) + 0.5 - pos.x;
                    let dy = f64::from(hp.0.y) + 0.1 - pos.y;
                    let dz = f64::from(hp.0.z) + 0.5 - pos.z;
                    dx * dx + dy * dy + dz * dz < 4.0
                });
            let new_target = should_pick.then(|| {
                BlockPos::new(
                    pos.x as i32 + rng.random_range(0i32..7) - rng.random_range(0i32..7),
                    (pos.y + f64::from(rng.random_range(0i32..6)) - 2.0) as i32,
                    pos.z as i32 + rng.random_range(0i32..7) - rng.random_range(0i32..7),
                )
            });
            let try_roost = rng.random_range(0u32..100) == 0;
            (should_pick, new_target, try_roost)
        };

        if should_pick_new {
            if let Some(target) = new_target {
                let target_state = world.get_block_state(&target);
                if target_state.is_air() && target.0.y > world.dimension.min_y {
                    *hanging_pos = Some(target);
                } else {
                    *hanging_pos = None;
                }
            } else {
                *hanging_pos = None;
            }
        }

        if let Some(target) = *hanging_pos {
            let d = f64::from(target.0.x) + 0.5 - pos.x;
            let e = f64::from(target.0.y) + 0.1 - pos.y;
            let f = f64::from(target.0.z) + 0.5 - pos.z;

            let velo = entity.velocity.load();
            let new_velo = Vector3::new(
                velo.x + (d.signum() * 0.5 - velo.x) * 0.1,
                velo.y + (e.signum() * 0.7 - velo.y) * 0.1,
                velo.z + (f.signum() * 0.5 - velo.z) * 0.1,
            );
            entity.velocity.store(new_velo);

            let yaw = (new_velo.z.atan2(new_velo.x) as f32).to_degrees() - 90.0;
            let yaw_diff = pumpkin_util::math::wrap_degrees(yaw - entity.yaw.load());
            entity.yaw.store(entity.yaw.load() + yaw_diff);
        }
        drop(hanging_pos);

        if try_roost {
            let above_state = world.get_block_state(above_pos);
            if above_state.is_solid_block() {
                self.set_roosting(true);
            }
        }
    }
}

impl Mob for BatEntity {
    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let flags: u8 = if self.is_roosting() { ROOSTING_FLAG } else { 0 };
        entity.send_meta_data(
            &[Metadata::new(tracked_data::bat::DATA_ID_FLAGS, flags)],
            None,
        );
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        let flags: u8 = if self.is_roosting() { ROOSTING_FLAG } else { 0 };
        nbt.put_byte("BatFlags", i8::try_from(flags).unwrap_or(0));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        let flags = u8::try_from(nbt.get_byte("BatFlags").unwrap_or(0)).unwrap_or(0);
        let roosting = (flags & ROOSTING_FLAG) != 0;
        self.set_roosting(roosting);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        let block_pos = entity.block_pos.load();
        let above_pos = BlockPos::new(block_pos.0.x, block_pos.0.y + 1, block_pos.0.z);
        let world = entity.world.load();
        let pos = entity.pos.load();

        self.tick_ambient_sound(&world, &pos);

        if self.is_roosting() {
            self.tick_roosting(&world, &above_pos, &pos);
        } else {
            self.tick_flying(&world, &above_pos, &pos);
        }
    }

    fn post_tick(&self) {
        if self.is_roosting() {
            let entity = &self.mob_entity.living_entity.entity;
            entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
            let pos = entity.pos.load();
            let snapped_y = (pos.y.floor()) + 1.0 - f64::from(entity.height());
            entity.set_pos(Vector3::new(pos.x, snapped_y, pos.z));
        }
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }

    fn get_mob_y_velocity_drag(&self) -> Option<f64> {
        Some(0.6)
    }

    fn on_damage(&self, _damage_type: DamageType, _source: Option<&dyn EntityBase>) {
        if self.is_roosting() {
            self.set_roosting(false);
            let entity = &self.mob_entity.living_entity.entity;
            let pos = entity.pos.load();
            entity.world.load().play_sound_fine(
                Sound::EntityBatTakeoff,
                SoundCategory::Ambient,
                &pos,
                0.1,
                0.95,
            );
        }
    }
}
