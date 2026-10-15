use crate::block::entities::BlockEntity;
use crate::world::World;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::any::Any;
use std::sync::Arc;

pub struct BellBlockEntity {
    pub position: BlockPos,
    pub last_side_hit: AtomicCell<Option<HorizontalFacing>>,
    pub ring_ticks: AtomicCell<i32>,
    pub ringing: AtomicCell<bool>,
    resonating: AtomicCell<bool>,
    resonate_time: AtomicCell<i32>,
}

impl BellBlockEntity {
    pub const ID: &'static str = "minecraft:bell";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            last_side_hit: AtomicCell::new(None),
            ring_ticks: AtomicCell::new(0),
            resonate_time: AtomicCell::new(0),
            resonating: AtomicCell::new(false),
            ringing: AtomicCell::new(false),
        }
    }

    pub fn activate(&self, direction: HorizontalFacing, world: &World) {
        self.last_side_hit.store(Some(direction));
        if self.ringing.load() {
            self.ring_ticks.store(0);
        } else {
            self.ringing.store(true);
        }

        let center = self.position.to_centered_f64();
        let entities = world.entities.load();
        for entity_base in entities.iter() {
            let entity = entity_base.get_entity();
            if entity.is_alive() && entity.pos.load().squared_distance_to_vec(&center) <= 32.0 * 32.0 {
                if let Some(mob) = entity_base.get_mob()
                    && let Some(villager) = mob.as_villager()
                {
                    villager.on_bell_heard();
                }
            }
        }
    }

    pub fn raiders_hear_bell(&self, world: &World) -> bool {
        let center = self.position.to_centered_f64();
        let entities = world.entities.load();
        for entity_base in entities.iter() {
            let entity = entity_base.get_entity();
            if entity.is_alive()
                && entity.pos.load().squared_distance_to_vec(&center) <= 32.0 * 32.0
                && entity
                    .entity_type
                    .has_tag(&tag::EntityType::MINECRAFT_RAIDERS)
            {
                return true;
            }
        }
        false
    }

    pub fn make_raiders_glow(&self, world: &World) {
        let center = self.position.to_centered_f64();
        let entities = world.entities.load();
        for entity_base in entities.iter() {
            let entity = entity_base.get_entity();
            if entity.is_alive()
                && entity.pos.load().squared_distance_to_vec(&center) <= 48.0 * 48.0
                && entity
                    .entity_type
                    .has_tag(&tag::EntityType::MINECRAFT_RAIDERS)
            {
                if let Some(living) = entity_base.get_living_entity() {
                    living.add_effect(Effect {
                        effect_type: &StatusEffect::GLOWING,
                        duration: 60,
                        amplifier: 0,
                        ambient: false,
                        show_particles: true,
                        show_icon: true,
                        blend: false,
                    });
                }
            }
        }
    }
}

impl BlockEntity for BellBlockEntity {
    fn write_nbt(&self, _nbt: &mut NbtCompound) {}

    fn from_nbt(_nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        Self::new(position)
    }

    fn tick(&self, world: &Arc<World>) {
        if self.ringing.load() {
            self.ring_ticks.fetch_add(1);
        }
        if self.ring_ticks.load() >= 50 {
            self.ringing.store(false);
            self.ring_ticks.store(0);
        }
        if self.ring_ticks.load() >= 5
            && self.resonate_time.load() == 0
            && self.raiders_hear_bell(world)
        {
            self.resonating.store(true);
            world.play_sound_fine(
                Sound::BlockBellResonate,
                SoundCategory::Blocks,
                &self.position.to_f64(),
                1.0,
                1.0,
            );
        }

        if self.resonating.load() {
            if self.resonate_time.load() < 40 {
                self.resonate_time.fetch_add(1);
            } else {
                self.make_raiders_glow(world);
                self.resonating.store(false);
                self.resonate_time.store(0);
            }
        }
    }

    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
