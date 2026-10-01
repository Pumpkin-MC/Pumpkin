use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::sound::Sound;

use crate::entity::mob::cube_mob::{CubeMob, CubeMobData};
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, custom_sound::CustomSound};

pub struct MagmaCubeEntity {
    pub mob_entity: MobEntity,
    pub cube_mob_data: CubeMobData,
}

impl MagmaCubeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let cube = Arc::new(Self {
            cube_mob_data: CubeMobData::new(&entity),
            mob_entity: MobEntity::new(entity),
        });
        let weak: Weak<dyn CubeMob> = {
            let mob: Arc<dyn CubeMob> = cube.clone();
            Arc::downgrade(&mob)
        };
        cube.register_common_goals(weak.clone());
        cube.add_attack_goal(weak);
        cube.randomize_size();
        cube
    }

    fn randomize_size(&self) {
        let mut size_scale = rand::random_range(0..3);
        if size_scale < 2 && rand::random_range(0.0..1.0) < 0.5 {
            size_scale += 1;
        }
        self.set_size(1 << size_scale, true);
    }
}

impl CubeMob for MagmaCubeEntity {
    fn get_cube_mob_data(&self) -> &CubeMobData {
        &self.cube_mob_data
    }
    fn jump_sound(&self) -> Sound {
        Sound::EntityMagmaCubeJump
    }
    fn squish_sound(&self) -> Sound {
        if self.is_tiny() {
            Sound::EntityMagmaCubeSquishSmall
        } else {
            Sound::EntityMagmaCubeSquish
        }
    }
    fn set_size(&self, size: i32, update_health: bool) {
        self.set_cube_mob_size(size, update_health);
        let actual_size = self.get_size();
        let mut attributes = self
            .mob_entity
            .living_entity
            .attributes
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(damage) = attributes.get_mut(&Attributes::ATTACK_DAMAGE.id) {
            damage.base_value = (actual_size + 2) as f64;
            damage.dirty.store(true, Ordering::Relaxed);
        }
        if let Some(armor) = attributes.get_mut(&Attributes::ARMOR.id) {
            armor.base_value = (actual_size * 3) as f64;
            armor.dirty.store(true, Ordering::Relaxed);
        }
        if let Some(speed) = attributes.get_mut(&Attributes::MOVEMENT_SPEED.id) {
            speed.base_value = 0.2;
            speed.dirty.store(true, Ordering::Relaxed);
        }
    }
    fn jump_delay(&self) -> i32 {
        rand::random_range(10..30) * 4
    }
    fn can_deal_damage(&self) -> bool {
        true
    }
    fn create_split_child(&self, entity: Entity, size: i32) -> Arc<dyn EntityBase> {
        let cube = Self::new(entity);
        cube.set_size(size, true);
        cube
    }
}

impl CustomSound for MagmaCubeEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(if self.is_tiny() {
            Sound::EntityMagmaCubeDeathSmall
        } else {
            Sound::EntityMagmaCubeDeath
        })
    }
    fn hurt_sound(&self) -> Option<Sound> {
        Some(if self.is_tiny() {
            Sound::EntityMagmaCubeHurtSmall
        } else {
            Sound::EntityMagmaCubeHurt
        })
    }
}

impl Mob for MagmaCubeEntity {
    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }
    fn mob_write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        self.write_cube_mob_nbt(nbt);
    }
    fn mob_read_nbt(&self, nbt: &pumpkin_nbt::compound::NbtCompound) {
        self.read_cube_mob_nbt(nbt);
    }
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.cube_mob_tick();
    }
    fn post_tick(&self) {
        self.cube_mob_post_tick();
    }
    fn mob_player_collision(&self, player: &Arc<Player>) {
        self.mob_entity.try_attack(self, &**player);
    }
}
