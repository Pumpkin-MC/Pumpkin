use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering::Relaxed},
};

use pumpkin_data::{
    particle::Particle,
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::compound::NbtCompound;
use rand::RngExt;

use super::{
    ageable_water_creature::AgeableWaterCreature,
    squid::{Squid, SquidData, impl_squid},
};
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    mob::{Mob, MobEntity},
};

const DEFAULT_DARK_TICKS_REMAINING: i32 = 0;
const DARK_TICKS_AFTER_DAMAGE: i32 = 100;

pub struct GlowSquidEntity {
    pub mob_entity: MobEntity,
    ageable_data: AgeableData,
    squid_data: Arc<SquidData>,
    dark_ticks_remaining: AtomicI32,
}

impl GlowSquidEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let squid_data = SquidData::new(&mob_entity);
        Arc::new(Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            squid_data,
            dark_ticks_remaining: AtomicI32::new(DEFAULT_DARK_TICKS_REMAINING),
        })
    }

    pub fn get_dark_ticks_remaining(&self) -> i32 {
        self.dark_ticks_remaining.load(Relaxed)
    }

    fn set_dark_ticks(&self, ticks: i32) {
        self.dark_ticks_remaining.store(ticks, Relaxed);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::glow_squid::DATA_DARK_TICKS_REMAINING,
            ticks,
        );
    }
}

impl Mob for GlowSquidEntity {
    crate::squid_mob_hooks!();

    fn mob_ai_step(&self) {
        self.squid_ai_step();
        let dark_ticks = self.get_dark_ticks_remaining();
        if dark_ticks > 0 {
            self.set_dark_ticks(dark_ticks - 1);
        }
        // The server discards GLOW particles but still draws their coordinates.
        let mut rng = self.get_random();
        let _ = (
            rng.random::<f64>(),
            rng.random::<f64>(),
            rng.random::<f64>(),
        );
    }

    fn mob_init_data_tracker(&self) {
        self.init_squid_data();
        self.set_dark_ticks(self.get_dark_ticks_remaining());
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("DarkTicksRemaining", self.get_dark_ticks_remaining());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_dark_ticks(
            nbt.get_int("DarkTicksRemaining")
                .unwrap_or(DEFAULT_DARK_TICKS_REMAINING),
        );
    }

    fn on_damage_with_context(
        &self,
        _damage_type: pumpkin_data::damage::DamageType,
        _source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        let hurt = self.squid_hurt();
        if hurt {
            self.set_dark_ticks(DARK_TICKS_AFTER_DAMAGE);
        }
        hurt
    }
}

impl_squid!(GlowSquidEntity, ambient: Sound::EntityGlowSquidAmbient, squirt: Sound::EntityGlowSquidSquirt, ink: Particle::GlowSquidInk);
