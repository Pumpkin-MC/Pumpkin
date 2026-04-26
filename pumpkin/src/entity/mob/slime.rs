use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::sound::Sound;

use crate::entity::{
    Entity, NBTStorage, NbtFuture,
    mob::{Mob, MobEntity},
};

pub struct SlimeEntity {
    entity: Arc<MobEntity>,
}

impl SlimeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        Arc::new(Self {
            entity: Arc::new(MobEntity::new(entity)),
        })
    }

    pub(crate) const fn hurt_sound_for_size(size: i32) -> Sound {
        if size == 1 {
            Sound::EntitySlimeHurtSmall
        } else {
            Sound::EntitySlimeHurt
        }
    }
}

use pumpkin_nbt::pnbt::PNbtCompound;

impl NBTStorage for SlimeEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut PNbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.living_entity.entity.write_nbt(nbt).await;
            nbt.put_int(self.entity.living_entity.entity.data.load(Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a mut PNbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.living_entity.entity.read_nbt_non_mut(nbt).await;
            self.entity
                .living_entity
                .entity
                .data
                .store(nbt.get_int().unwrap_or(0), Relaxed);
        })
    }
}

impl Mob for SlimeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_small_hurt_sound_only_for_smallest_slimes() {
        assert_eq!(
            SlimeEntity::hurt_sound_for_size(1),
            Sound::EntitySlimeHurtSmall
        );
        assert_eq!(SlimeEntity::hurt_sound_for_size(0), Sound::EntitySlimeHurt);
        assert_eq!(SlimeEntity::hurt_sound_for_size(2), Sound::EntitySlimeHurt);
    }
}
