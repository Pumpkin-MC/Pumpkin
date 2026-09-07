use super::BlockEntity;
use crate::entity::EntityBase;
use crate::world::World;
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::MobCategory;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// `ConduitBlockEntity`.
///
/// Every 2 seconds the conduit re-scans its frame. With at least 16 frame blocks
/// (prismarine, prismarine bricks, dark prismarine or sea lantern) on the edges of
/// the 5x5x5 shell around it, and water in the full 3x3x3 core, it activates and
/// grants Conduit Power to wet players in range. With 42 or more frame blocks it
/// also attacks one hostile mob within 8 blocks.
pub struct ConduitBlockEntity {
    pub position: BlockPos,
    pub active: AtomicBool,
    /// `destroyTarget`: the hostile mob currently being zapped, synced to clients
    /// as the `Target` NBT tag so they can render the beam.
    pub target: Mutex<Option<Uuid>>,
    /// `effectBlocks`: frame blocks found by the last shape scan.
    pub effect_blocks: Mutex<Vec<BlockPos>>,
}

impl BlockEntity for ConduitBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    /// Reads `Active` and `Target`.
    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let active = nbt.get_bool("Active").unwrap_or(false);
        let target = nbt.get_uuid("Target");
        Self {
            position,
            active: AtomicBool::new(active),
            target: Mutex::new(target),
            effect_blocks: Mutex::new(Vec::new()),
        }
    }

    /// Writes `Active` and `Target`.
    fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("Active", self.active.load(Ordering::Relaxed));
        if let Some(target) = *self
            .target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        {
            nbt.put_uuid("Target", target);
        }
    }

    /// `ConduitBlockEntity.serverTick`.
    fn tick(&self, world: &Arc<World>) {
        let game_time = world.get_world_age();

        if game_time % Self::SHAPE_UPDATE_INTERVAL == 0 {
            let active = self.update_shape(world);
            if self.active.swap(active, Ordering::Relaxed) != active {
                let sound = if active {
                    Sound::BlockConduitActivate
                } else {
                    Sound::BlockConduitDeactivate
                };
                world.play_sound(sound, SoundCategory::Blocks, &self.center());
            }
        }

        if !self.active.load(Ordering::Relaxed) {
            return;
        }

        if game_time % Self::EFFECT_INTERVAL == 0 {
            self.apply_effects(world);
            world.play_sound(
                Sound::BlockConduitAmbient,
                SoundCategory::Blocks,
                &self.center(),
            );
        }

        if game_time % Self::ATTACK_INTERVAL == 0 {
            self.update_destroy_target(world);
        }
    }

    /// Same as [`Self::write_nbt`].
    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.write_nbt(&mut nbt);
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ConduitBlockEntity {
    pub const ID: &'static str = "minecraft:conduit";

    const SHAPE_UPDATE_INTERVAL: i64 = 40;
    const EFFECT_INTERVAL: i64 = 80;
    const ATTACK_INTERVAL: i64 = 40;
    const MIN_ACTIVE_SIZE: usize = 16;
    const MIN_HUNTING_SIZE: usize = 42;
    const BLOCKS_PER_RANGE_STEP: i32 = 7;
    const RANGE_PER_STEP: i32 = 16;
    const DESTROY_RANGE: f64 = 8.0;
    const DESTROY_DAMAGE: f32 = 4.0;
    const EFFECT_DURATION: i32 = 260;

    /// Inactive until the first scan.
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            active: AtomicBool::new(false),
            target: Mutex::new(None),
            effect_blocks: Mutex::new(Vec::new()),
        }
    }

    /// Result of the last frame scan.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    /// `isHunting`.
    #[must_use]
    pub fn is_hunting(&self) -> bool {
        self.effect_block_count() >= Self::MIN_HUNTING_SIZE
    }

    /// Frame blocks from the last scan.
    fn effect_block_count(&self) -> usize {
        self.effect_blocks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// Block centre.
    fn center(&self) -> pumpkin_util::math::vector3::Vector3<f64> {
        self.position.to_centered_f64()
    }

    /// `ConduitBlockEntity.VALID_BLOCKS`.
    fn is_frame_block(block: &Block) -> bool {
        block.id == BlockId::PRISMARINE
            || block.id == BlockId::PRISMARINE_BRICKS
            || block.id == BlockId::DARK_PRISMARINE
            || block.id == BlockId::SEA_LANTERN
    }

    /// `updateShape`.
    fn update_shape(&self, world: &World) -> bool {
        let mut blocks = Vec::new();
        let origin = self.position.0;

        let mut core_flooded = true;
        'core: for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let pos = BlockPos::new(origin.x + dx, origin.y + dy, origin.z + dz);
                    if !world.get_fluid(&pos).has_tag(&tag::Fluid::MINECRAFT_WATER) {
                        core_flooded = false;
                        break 'core;
                    }
                }
            }
        }

        if core_flooded {
            for dx in -2i32..=2 {
                for dy in -2i32..=2 {
                    for dz in -2i32..=2 {
                        let (ax, ay, az) = (dx.abs(), dy.abs(), dz.abs());
                        let on_shell = ax > 1 || ay > 1 || az > 1;
                        let on_edge_line = (dx == 0 && (ay == 2 || az == 2))
                            || (dy == 0 && (ax == 2 || az == 2))
                            || (dz == 0 && (ax == 2 || ay == 2));
                        if !(on_shell && on_edge_line) {
                            continue;
                        }
                        let pos = BlockPos::new(origin.x + dx, origin.y + dy, origin.z + dz);
                        if Self::is_frame_block(world.get_block(&pos)) {
                            blocks.push(pos);
                        }
                    }
                }
            }
        }

        let active = core_flooded && blocks.len() >= Self::MIN_ACTIVE_SIZE;
        *self
            .effect_blocks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = blocks;
        active
    }

    /// `applyEffects`.
    fn apply_effects(&self, world: &World) {
        let count = self.effect_block_count() as i32;
        let range = count / Self::BLOCKS_PER_RANGE_STEP * Self::RANGE_PER_STEP;
        if range <= 0 {
            return;
        }
        let range_squared = range * range;

        for player in world.players.load().iter() {
            let entity = &player.living_entity.entity;
            if self.position.squared_distance(&entity.block_pos.load()) >= range_squared {
                continue;
            }
            if !entity.is_in_water_rain_or_bubble() {
                continue;
            }
            player.add_effect(Effect {
                effect_type: &StatusEffect::CONDUIT_POWER,
                duration: Self::EFFECT_DURATION,
                amplifier: 0,
                ambient: true,
                show_particles: true,
                show_icon: true,
                blend: false,
            });
        }
    }

    /// `updateDestroyTarget`.
    fn update_destroy_target(&self, world: &Arc<World>) {
        let previous = *self
            .target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let target = if self.is_hunting() {
            let current = previous
                .and_then(|uuid| world.get_entity_by_uuid(uuid))
                .filter(|entity| self.is_valid_target(entity, false));
            current.or_else(|| self.find_destroy_target(world))
        } else {
            None
        };

        if let Some(target) = &target {
            world.play_sound(
                Sound::BlockConduitAttackTarget,
                SoundCategory::Blocks,
                &self.center(),
            );
            target.damage(target.as_ref(), Self::DESTROY_DAMAGE, DamageType::MAGIC);
        }

        let target_uuid = target.map(|entity| entity.get_entity().entity_uuid);
        if target_uuid != previous {
            *self
                .target
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = target_uuid;
            if let Some(block_entity) = world.get_block_entity(&self.position) {
                world.update_block_entity(&block_entity);
            }
        }
    }

    /// Hostile, alive, within range; new targets must be in water or rain.
    fn is_valid_target(&self, entity: &Arc<dyn EntityBase>, require_wet: bool) -> bool {
        let base = entity.get_entity();
        if !base.is_alive() || entity.get_living_entity().is_none() {
            return false;
        }
        if base.entity_type.category != &MobCategory::MONSTER {
            return false;
        }
        if require_wet && !base.is_in_water_or_rain() {
            return false;
        }
        let range = Self::DESTROY_RANGE as i32;
        self.position.squared_distance(&base.block_pos.load()) < range * range
    }

    /// Random valid target in the destroy range.
    fn find_destroy_target(&self, world: &World) -> Option<Arc<dyn EntityBase>> {
        let area = BoundingBox::from_block(&self.position).expand(
            Self::DESTROY_RANGE,
            Self::DESTROY_RANGE,
            Self::DESTROY_RANGE,
        );
        let candidates: Vec<Arc<dyn EntityBase>> = world
            .get_nearby_entities(self.center(), Self::DESTROY_RANGE * 2.0)
            .into_values()
            .filter(|entity| {
                entity.get_entity().bounding_box.load().intersects(&area)
                    && self.is_valid_target(entity, true)
            })
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let index = rand::random_range(0..candidates.len());
        candidates.into_iter().nth(index)
    }
}
