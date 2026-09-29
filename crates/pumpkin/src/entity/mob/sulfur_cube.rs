use std::sync::{Arc, Weak};

use pumpkin_data::sound::{Sound, SoundCategory};

use crate::entity::mob::cube_mob::{AbstractCubeMob, CubeMobHooks};
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::{Entity, EntityBase, custom_sound::CustomSound};

pub const SPLIT_COUNT: i32 = 2;
pub const MAX_SIZE: i32 = 2;

pub struct SulfurCubeEntity {
    pub cube: AbstractCubeMob,
}

impl SulfurCubeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let cube = Arc::new(Self {
            cube: AbstractCubeMob::new(entity),
        });
        let weak: Weak<dyn CubeMobHooks> = {
            let mob: Arc<dyn CubeMobHooks> = cube.clone();
            Arc::downgrade(&mob)
        };
        cube.cube.register_common_goals(weak);
        cube.set_size(MAX_SIZE, true);
        cube
    }

    pub fn get_size(&self) -> i32 {
        self.cube.get_size()
    }
    pub fn is_tiny(&self) -> bool {
        self.cube.is_tiny()
    }
}

impl CubeMobHooks for SulfurCubeEntity {
    fn cube_mob(&self) -> &AbstractCubeMob {
        &self.cube
    }
    fn jump_sound(&self) -> Sound {
        if self.is_tiny() {
            Sound::EntitySmallSulfurCubeJump
        } else {
            Sound::EntitySulfurCubeJump
        }
    }
    fn squish_sound(&self) -> Sound {
        if self.is_tiny() {
            Sound::EntitySmallSulfurCubeSquish
        } else {
            Sound::EntitySulfurCubeSquish
        }
    }
    fn sound_source(&self) -> SoundCategory {
        SoundCategory::Neutral
    }
    fn set_size(&self, size: i32, update_health: bool) {
        self.cube.set_size(self, size, update_health);
    }
    fn cube_mob_health(&self, actual_size: i32) -> f64 {
        f64::from(4 * actual_size)
    }
    fn split_count(&self) -> i32 {
        SPLIT_COUNT
    }
    fn create_split_child(&self, entity: Entity, size: i32) -> Arc<dyn EntityBase> {
        let cube = Self::new(entity);
        cube.set_size(size, true);
        cube
    }
}

impl CustomSound for SulfurCubeEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(if self.is_tiny() {
            Sound::EntitySmallSulfurCubeDeath
        } else {
            Sound::EntitySulfurCubeDeath
        })
    }
    fn hurt_sound(&self) -> Option<Sound> {
        Some(if self.is_tiny() {
            Sound::EntitySmallSulfurCubeHurt
        } else {
            Sound::EntitySulfurCubeHurt
        })
    }
}

impl Mob for SulfurCubeEntity {
    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }
    fn mob_write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        self.cube.mob_write_nbt(nbt);
    }
    fn mob_read_nbt(&self, nbt: &pumpkin_nbt::compound::NbtCompound) {
        self.cube.mob_read_nbt(self, nbt);
    }
    fn get_mob_entity(&self) -> &MobEntity {
        &self.cube.mob_entity
    }
    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.cube.mob_tick(self);
    }
    fn post_tick(&self) {
        self.cube.post_tick(self);
    }
}
