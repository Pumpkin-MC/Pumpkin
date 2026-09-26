//! Beacon block entity, mirroring vanilla `BeaconBlockEntity`.
//!
//! Each tick scans up to ten blocks of the column above the beacon; once the scan reaches
//! the world surface the result becomes the active beam state. Every 80 game ticks an
//! active beacon re-measures its pyramid and applies its effects to players in range.

use std::any::Any;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex};

use pumpkin_data::Block;
use pumpkin_data::advancement::Advancement;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_inventory::beacon_screen_handler::{
    DATA_LEVELS, DATA_PRIMARY, DATA_SECONDARY, NUM_DATA_VALUES, decode_effect, encode_effect,
    filter_effect,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::chunk::ChunkHeightmapType;

use crate::block::entities::{BlockEntity, PropertyDelegate};
use crate::world::World;

const MAX_LEVELS: i32 = 4;
const BLOCKS_CHECK_PER_TICK: i32 = 10;

pub struct BeaconBlockEntity {
    pub position: BlockPos,
    /// Encoded via [`encode_effect`]: `0` = none, otherwise `effect id + 1`.
    primary_effect: AtomicI32,
    secondary_effect: AtomicI32,
    levels: AtomicI32,
    dirty: AtomicBool,
    /// Result of the last completed column scan: `true` when nothing opaque blocks the beam.
    beam_active: AtomicBool,
    /// Whether the scan currently in progress has hit an opaque block.
    checking_beam_blocked: AtomicBool,
    last_check_y: AtomicI32,
    /// Set when the primary effect changes while the beam is up; consumed by `tick`.
    play_select_sound: AtomicBool,
    custom_name: Mutex<Option<String>>,
    lock_key: Mutex<Option<String>>,
}

/// Blocks the beam passes through and takes its colour from (`BeaconBeamBlock` in vanilla).
fn is_beam_block(block: &Block) -> bool {
    block == &Block::BEACON
        || block.name.ends_with("_stained_glass")
        || block.name.ends_with("_stained_glass_pane")
}

impl BeaconBlockEntity {
    pub const ID: &'static str = "minecraft:beacon";

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            primary_effect: AtomicI32::new(0),
            secondary_effect: AtomicI32::new(0),
            levels: AtomicI32::new(0),
            dirty: AtomicBool::new(false),
            beam_active: AtomicBool::new(false),
            checking_beam_blocked: AtomicBool::new(false),
            last_check_y: AtomicI32::new(position.0.y - 1),
            play_select_sound: AtomicBool::new(false),
            custom_name: Mutex::new(None),
            lock_key: Mutex::new(None),
        }
    }

    #[must_use]
    pub fn levels(&self) -> i32 {
        self.levels.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn primary_effect(&self) -> Option<&'static StatusEffect> {
        decode_effect(self.primary_effect.load(Ordering::Relaxed))
    }

    #[must_use]
    pub fn secondary_effect(&self) -> Option<&'static StatusEffect> {
        decode_effect(self.secondary_effect.load(Ordering::Relaxed))
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn play_sound(world: &World, position: BlockPos, sound: Sound) {
        world.play_block_sound(sound, SoundCategory::Blocks, position);
    }

    /// Mirrors `BeaconBlockEntity.updateBase`: the number of complete pyramid layers below.
    fn update_base(&self, world: &World) -> i32 {
        let BlockPos(pos) = self.position;
        let mut levels = 0;
        for step in 1..=MAX_LEVELS {
            let layer_y = pos.y - step;
            if layer_y < world.min_y {
                break;
            }
            let layer_ok = (pos.x - step..=pos.x + step).all(|x| {
                (pos.z - step..=pos.z + step).all(|z| {
                    world
                        .get_block(&BlockPos::new(x, layer_y, z))
                        .has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEACON_BASE_BLOCKS)
                })
            });
            if !layer_ok {
                break;
            }
            levels = step;
        }
        levels
    }

    /// Mirrors `BeaconBlockEntity.applyEffects`.
    fn apply_effects(&self, world: &World, levels: i32) {
        let Some(primary) = self.primary_effect() else {
            return;
        };
        let secondary = self.secondary_effect();
        let same = secondary.is_some_and(|s| s.id == primary.id);

        let range = f64::from(levels * 10 + 10);
        let base_amp = u8::from(levels >= MAX_LEVELS && same);
        let duration = (9 + levels * 2) * 20;
        let pos = self.position.0.to_f64();
        let bounds = BoundingBox::new_array(
            [pos.x - range, pos.y - range, pos.z - range],
            [
                pos.x + 1.0 + range,
                pos.y + 1.0 + range + f64::from(world.dimension.height),
                pos.z + 1.0 + range,
            ],
        );
        let effect = |effect_type, amplifier| Effect {
            effect_type,
            duration,
            amplifier,
            ambient: true,
            show_particles: true,
            show_icon: true,
            blend: false,
        };

        for player in world.players.load().iter() {
            if !bounds.intersects(&player.living_entity.entity.bounding_box.load()) {
                continue;
            }
            player.add_effect(effect(primary, base_amp));
            if levels >= MAX_LEVELS
                && !same
                && let Some(secondary) = secondary
            {
                player.add_effect(effect(secondary, 0));
            }
        }
    }

    /// Fires the `construct_beacon` criterion for players near a freshly activated beacon.
    fn trigger_construct_beacon(&self, world: &World, levels: i32) {
        let pos = self.position.0.to_f64();
        let bounds = BoundingBox::new_array(
            [pos.x - 10.0, pos.y - 4.0 - 5.0, pos.z - 10.0],
            [pos.x + 10.0, pos.y + 5.0, pos.z + 10.0],
        );
        for player in world.players.load().iter() {
            if !bounds.intersects(&player.living_entity.entity.bounding_box.load()) {
                continue;
            }
            player.trigger_advancement_criterion(Advancement::NETHER_CREATE_BEACON, "beacon");
            if levels >= MAX_LEVELS {
                player.trigger_advancement_criterion(
                    Advancement::NETHER_CREATE_FULL_BEACON,
                    "beacon",
                );
            }
        }
    }

    fn effect_from_nbt(nbt: &NbtCompound, key: &str) -> i32 {
        encode_effect(filter_effect(
            nbt.get_string(key)
                .and_then(StatusEffect::from_minecraft_name),
        ))
    }

    fn effect_to_nbt(nbt: &mut NbtCompound, key: &str, encoded: i32) {
        if let Some(effect) = decode_effect(encoded) {
            nbt.put_string(key, effect.minecraft_name.to_string());
        }
    }
}

impl PropertyDelegate for BeaconBlockEntity {
    fn get_property(&self, index: i32) -> i32 {
        match index {
            DATA_LEVELS => self.levels.load(Ordering::Relaxed),
            DATA_PRIMARY => self.primary_effect.load(Ordering::Relaxed),
            DATA_SECONDARY => self.secondary_effect.load(Ordering::Relaxed),
            _ => 0,
        }
    }

    fn set_property(&self, index: i32, value: i32) {
        match index {
            DATA_LEVELS => self.levels.store(value, Ordering::Relaxed),
            DATA_PRIMARY => {
                if self.beam_active.load(Ordering::Relaxed) {
                    self.play_select_sound.store(true, Ordering::Relaxed);
                }
                self.primary_effect.store(
                    encode_effect(filter_effect(decode_effect(value))),
                    Ordering::Relaxed,
                );
            }
            DATA_SECONDARY => self.secondary_effect.store(
                encode_effect(filter_effect(decode_effect(value))),
                Ordering::Relaxed,
            ),
            _ => return,
        }
        self.mark_dirty();
    }

    fn get_properties_size(&self) -> i32 {
        NUM_DATA_VALUES
    }
}

impl BlockEntity for BeaconBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self {
        let entity = Self::new(position);
        entity.primary_effect.store(
            Self::effect_from_nbt(nbt, "primary_effect"),
            Ordering::Relaxed,
        );
        entity.secondary_effect.store(
            Self::effect_from_nbt(nbt, "secondary_effect"),
            Ordering::Relaxed,
        );
        entity
            .levels
            .store(nbt.get_int("Levels").unwrap_or(0), Ordering::Relaxed);
        *entity
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            nbt.get_string("CustomName").map(str::to_string);
        *entity
            .lock_key
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            nbt.get_string("Lock").map(str::to_string);
        entity
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        Self::effect_to_nbt(
            nbt,
            "primary_effect",
            self.primary_effect.load(Ordering::Relaxed),
        );
        Self::effect_to_nbt(
            nbt,
            "secondary_effect",
            self.secondary_effect.load(Ordering::Relaxed),
        );
        nbt.put_int("Levels", self.levels.load(Ordering::Relaxed));
        if let Some(name) = self
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            nbt.put_string("CustomName", name.clone());
        }
        if let Some(lock) = self
            .lock_key
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            nbt.put_string("Lock", lock.clone());
        }
    }

    /// Mirrors `BeaconBlockEntity.tick`.
    fn tick(&self, world: &Arc<World>) {
        let BlockPos(pos) = self.position;
        let mut last_check_y = self.last_check_y.load(Ordering::Relaxed);
        if last_check_y < pos.y {
            self.checking_beam_blocked.store(false, Ordering::Relaxed);
            last_check_y = pos.y - 1;
        }
        let surface_y = world.get_heightmap_height(ChunkHeightmapType::WorldSurface, pos.x, pos.z);

        let mut checked = 0;
        while checked < BLOCKS_CHECK_PER_TICK && last_check_y < surface_y {
            let check_pos = BlockPos::new(pos.x, last_check_y + 1, pos.z);
            let block = world.get_block(&check_pos);
            if !is_beam_block(block) {
                let opaque = world.get_block_state(&check_pos).opacity >= 15;
                // The first block must be the beacon itself; anything opaque above it (bedrock
                // excepted) cuts the beam.
                if check_pos.0.y == pos.y || (opaque && block != &Block::BEDROCK) {
                    self.checking_beam_blocked.store(true, Ordering::Relaxed);
                    last_check_y = surface_y;
                    break;
                }
            }
            last_check_y += 1;
            checked += 1;
        }

        let previous_levels = self.levels.load(Ordering::Relaxed);
        let beam_active = self.beam_active.load(Ordering::Relaxed);
        if world.get_world_age() % 80 == 0 {
            if beam_active {
                let levels = self.update_base(world);
                if levels != previous_levels {
                    self.levels.store(levels, Ordering::Relaxed);
                    self.mark_dirty();
                }
            }
            let levels = self.levels.load(Ordering::Relaxed);
            if levels > 0 && beam_active {
                self.apply_effects(world, levels);
                Self::play_sound(world, self.position, Sound::BlockBeaconAmbient);
            }
        }

        if last_check_y >= surface_y {
            last_check_y = world.min_y - 1;
            self.beam_active.store(
                !self.checking_beam_blocked.load(Ordering::Relaxed),
                Ordering::Relaxed,
            );
            let was_active = previous_levels > 0;
            let levels = self.levels.load(Ordering::Relaxed);
            let is_active = levels > 0;
            if !was_active && is_active {
                Self::play_sound(world, self.position, Sound::BlockBeaconActivate);
                self.trigger_construct_beacon(world, levels);
            } else if was_active && !is_active {
                Self::play_sound(world, self.position, Sound::BlockBeaconDeactivate);
            }
        }
        self.last_check_y.store(last_check_y, Ordering::Relaxed);

        if self.play_select_sound.swap(false, Ordering::Relaxed) {
            Self::play_sound(world, self.position, Sound::BlockBeaconPowerSelect);
        }
    }

    fn on_block_replaced(self: Arc<Self>, world: &Arc<World>, position: &BlockPos) {
        // Vanilla `setRemoved` plays this regardless of whether the beacon was lit.
        Self::play_sound(world, *position, Sound::BlockBeaconDeactivate);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.write_nbt(&mut nbt);
        Some(nbt)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn to_property_delegate(self: Arc<Self>) -> Option<Arc<dyn PropertyDelegate>> {
        Some(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
